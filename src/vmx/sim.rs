//! Linux-friendly VMX/MSR/VMCS simulation for integration testing.
//!
//! Enable with `--features sim` and run `cargo test --features sim`.

use super::state::CpuState;
use super::vmcs::encoding;
use std::cell::RefCell;
use std::collections::HashMap;

use super::msr::{
    FEATURE_CONTROL_LOCKED, FEATURE_CONTROL_VMXON_OUTSIDE_SMX, IA32_EFER, IA32_FEATURE_CONTROL,
    IA32_VMX_BASIC, IA32_VMX_CR0_FIXED0, IA32_VMX_CR0_FIXED1, IA32_VMX_CR4_FIXED0,
    IA32_VMX_CR4_FIXED1, IA32_VMX_ENTRY_CTLS, IA32_VMX_EXIT_CTLS, IA32_VMX_PINBASED_CTLS,
    IA32_VMX_PROCBASED_CTLS, IA32_VMX_PROCBASED_CTLS2,
};

pub const HOST_EXIT_STUB: u64 = 0xFFFF_0000_1000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimEvent {
    Vmxon { region: u64 },
    Vmptrld { vmcs_pa: u64 },
    Vmlaunch,
    Vmwrite { field: u32, value: u64 },
    Vmread { field: u32 },
}

#[derive(Debug, Default)]
struct SimState {
    msrs: HashMap<u32, u64>,
    cpuid: HashMap<u32, u32>,
    vmcs: HashMap<u32, u64>,
    cr: [u64; 5],
    vmx_enabled: bool,
    vmcs_loaded: bool,
    vmxon_fail: bool,
    vmlaunch_fail: bool,
    events: Vec<SimEvent>,
    cpu: CpuState,
}

impl SimState {
    fn new() -> Self {
        let mut state = Self {
            cr: [0; 5],
            ..Default::default()
        };
        state.cr[0] = 0x8001_0033;
        state.cr[3] = 0x1A000;
        state.cr[4] = 0x0000_0200;
        state.cpu = default_cpu_state();
        state.apply_vmx_defaults();
        state
    }

    fn apply_vmx_defaults(&mut self) {
        self.cpuid.insert(1, 1 << 5);
        self.msrs.insert(IA32_VMX_BASIC, (1 << 55) | 0x1A);
        self.msrs.insert(
            IA32_FEATURE_CONTROL,
            FEATURE_CONTROL_LOCKED | FEATURE_CONTROL_VMXON_OUTSIDE_SMX,
        );
        self.msrs.insert(IA32_VMX_CR0_FIXED0, 1);
        self.msrs.insert(IA32_VMX_CR0_FIXED1, u64::MAX);
        self.msrs.insert(IA32_VMX_CR4_FIXED0, 1 << 13);
        self.msrs.insert(IA32_VMX_CR4_FIXED1, u64::MAX);
        self.msrs
            .insert(IA32_VMX_PINBASED_CTLS, 0xFFFF_FFFF_0000_0000);
        self.msrs.insert(IA32_VMX_PROCBASED_CTLS, 1u64 << (32 + 31));
        self.msrs.insert(IA32_VMX_PROCBASED_CTLS2, 1u64 << (32 + 1));
        self.msrs.insert(IA32_VMX_EXIT_CTLS, 0xFFFF_FFFF_0000_0000);
        self.msrs.insert(IA32_VMX_ENTRY_CTLS, 0xFFFF_FFFF_0000_0000);
        self.msrs.insert(IA32_EFER, 0x500);
    }
}

thread_local! {
    static SIM: RefCell<SimState> = RefCell::new(SimState::new());
}

fn with_sim<T>(f: impl FnOnce(&mut SimState) -> T) -> T {
    SIM.with(|sim| f(&mut sim.borrow_mut()))
}

pub fn reset() {
    with_sim(|sim| *sim = SimState::new());
}

pub fn set_msr(msr: u32, value: u64) {
    with_sim(|sim| {
        sim.msrs.insert(msr, value);
    });
}

pub fn set_cpuid_ecx(leaf: u32, ecx: u32) {
    with_sim(|sim| {
        sim.cpuid.insert(leaf, ecx);
    });
}

pub fn set_vmxon_fail(fail: bool) {
    with_sim(|sim| sim.vmxon_fail = fail);
}

pub fn set_vmlaunch_fail(fail: bool) {
    with_sim(|sim| sim.vmlaunch_fail = fail);
}

pub fn cpuid_ecx(leaf: u32) -> Option<u32> {
    with_sim(|sim| sim.cpuid.get(&leaf).copied())
}

pub fn read_msr(msr: u32) -> u64 {
    with_sim(|sim| *sim.msrs.get(&msr).unwrap_or(&0))
}

pub fn read_cr(index: usize) -> u64 {
    with_sim(|sim| sim.cr.get(index).copied().unwrap_or(0))
}

pub fn write_cr(index: usize, value: u64) {
    with_sim(|sim| {
        if let Some(cr) = sim.cr.get_mut(index) {
            *cr = value;
        }
    });
}

pub fn vmxon(region: u64) -> bool {
    with_sim(|sim| {
        sim.events.push(SimEvent::Vmxon { region });
        if sim.vmxon_fail {
            return false;
        }
        sim.vmx_enabled = true;
        true
    })
}

pub fn vmptrld(vmcs_pa: u64) -> bool {
    with_sim(|sim| {
        sim.events.push(SimEvent::Vmptrld { vmcs_pa });
        if !sim.vmx_enabled {
            return false;
        }
        sim.vmcs_loaded = true;
        true
    })
}

pub fn vmwrite(field: u32, value: u64) -> Result<(), super::vmcs::VmcsError> {
    with_sim(|sim| {
        if !sim.vmcs_loaded {
            return Err(super::vmcs::VmcsError::VmwriteFailed);
        }
        sim.events.push(SimEvent::Vmwrite { field, value });
        sim.vmcs.insert(field, value);
        Ok(())
    })
}

pub fn vmread(field: u32) -> Result<u64, super::vmcs::VmcsError> {
    with_sim(|sim| {
        if !sim.vmcs_loaded {
            return Err(super::vmcs::VmcsError::VmreadFailed);
        }
        sim.events.push(SimEvent::Vmread { field });
        Ok(*sim.vmcs.get(&field).unwrap_or(&0))
    })
}

pub fn vmlaunch() -> Result<(), super::vmlaunch::VmxLaunchError> {
    with_sim(|sim| {
        sim.events.push(SimEvent::Vmlaunch);
        if sim.vmlaunch_fail || !sim.vmx_enabled || !sim.vmcs_loaded {
            sim.vmcs.insert(encoding::VM_INSTRUCTION_ERROR, 11);
            return Err(super::vmlaunch::VmxLaunchError::VmlaunchFailed(11));
        }

        let required = [
            encoding::GUEST_RIP,
            encoding::GUEST_RSP,
            encoding::HOST_RIP,
            encoding::HOST_RSP,
            encoding::EPT_POINTER,
            encoding::CPU_BASED_VM_EXEC_CONTROL,
            encoding::SECONDARY_VM_EXEC_CONTROL,
        ];
        for field in required {
            if !sim.vmcs.contains_key(&field) || sim.vmcs[&field] == 0 {
                sim.vmcs.insert(encoding::VM_INSTRUCTION_ERROR, 7);
                return Err(super::vmlaunch::VmxLaunchError::VmlaunchFailed(7));
            }
        }

        Ok(())
    })
}

pub fn cpu_state() -> CpuState {
    with_sim(|sim| sim.cpu)
}

pub fn events() -> Vec<SimEvent> {
    with_sim(|sim| sim.events.clone())
}

pub fn vmcs_field(field: u32) -> u64 {
    with_sim(|sim| *sim.vmcs.get(&field).unwrap_or(&0))
}

fn default_cpu_state() -> CpuState {
    CpuState {
        cr0: 0x8001_0033,
        cr3: 0x1A000,
        cr4: 0x0000_0200,
        dr7: 0x400,
        rip: 0xFFFF_F000_2000,
        rsp: 0xFFFF_F000_8000,
        rflags: 0x202,
        cs: super::state::SegmentState {
            selector: 0x10,
            base: 0,
            limit: 0xFFFF_FFFF,
            access_rights: 0xA09B,
        },
        ss: super::state::SegmentState {
            selector: 0x18,
            base: 0,
            limit: 0xFFFF_FFFF,
            access_rights: 0xC093,
        },
        ds: super::state::SegmentState {
            selector: 0x2B,
            base: 0,
            limit: 0xFFFF_FFFF,
            access_rights: 0xC093,
        },
        es: super::state::SegmentState {
            selector: 0x2B,
            base: 0,
            limit: 0xFFFF_FFFF,
            access_rights: 0xC093,
        },
        fs: super::state::SegmentState {
            selector: 0x53,
            base: 0,
            limit: 0xFFFF_FFFF,
            access_rights: 0xC093,
        },
        gs: super::state::SegmentState {
            selector: 0x2B,
            base: 0,
            limit: 0xFFFF_FFFF,
            access_rights: 0xC093,
        },
        tr: super::state::SegmentState {
            selector: 0x40,
            base: 0,
            limit: 0x67,
            access_rights: 0x008B,
        },
        ldtr: super::state::SegmentState {
            selector: 0,
            base: 0,
            limit: 0,
            access_rights: 1 << 16,
        },
        gdtr_base: 0xFFFF_F000_0000,
        gdtr_limit: 0xFFFF,
        idtr_base: 0xFFFF_F000_1000,
        idtr_limit: 0xFFFF,
        efer: 0x500,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_allow_vmxon() {
        reset();
        assert!(vmxon(0x1000));
        assert!(events().iter().any(|e| matches!(e, SimEvent::Vmxon { .. })));
    }
}
