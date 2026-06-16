// src/vmx/init.rs

use super::memory::{VMCS_REGION, VMXON_REGION, physical_address};
use super::msr::{
    FEATURE_CONTROL_LOCKED, FEATURE_CONTROL_VMXON_OUTSIDE_SMX, IA32_FEATURE_CONTROL,
    IA32_VMX_BASIC, adjust_cr0_for_vmx, adjust_cr4_for_vmx, rdmsr, vmx_revision_id,
};
use core::arch::asm;

#[cfg(windows)]
extern "C" {
    fn vm_exit_wrapper();
}

#[cfg(windows)]
pub fn vm_exit_handler_address() -> u64 {
    vm_exit_wrapper as *const () as u64
}

#[cfg(not(windows))]
pub fn vm_exit_handler_address() -> u64 {
    0
}

/// Enables VMX operation on the current CPU.
///
/// # Safety
///
/// Caller must ensure the CPU supports VMX and that static VMX regions are initialized.
pub unsafe fn enable_vmx() -> Result<(), &'static str> {
    let vmx_basic = unsafe { rdmsr(IA32_VMX_BASIC) };
    if vmx_basic & (1 << 55) == 0 {
        return Err("64-bit VMX not supported");
    }

    unsafe { initialize_vmx_regions(vmx_basic)? };
    unsafe { ensure_feature_control()? };

    let mut cr0: u64;
    let mut cr4: u64;
    unsafe {
        asm!("mov {}, cr0", out(reg) cr0);
        asm!("mov {}, cr4", out(reg) cr4);
    }

    cr0 = adjust_cr0_for_vmx(cr0);
    cr4 = adjust_cr4_for_vmx(cr4 | (1 << 13));

    unsafe {
        asm!("mov cr0, {}", in(reg) cr0);
        asm!("mov cr4, {}", in(reg) cr4);
    }

    let vmxon_pa = unsafe { physical_address(VMXON_REGION.get()) };
    unsafe {
        if !vmxon(vmxon_pa) {
            return Err("VMXON failed");
        }
    }

    Ok(())
}

unsafe fn initialize_vmx_regions(vmx_basic: u64) -> Result<(), &'static str> {
    let revision = vmx_revision_id(vmx_basic);
    unsafe {
        (*VMXON_REGION.get()).set_revision_id(revision);
        (*VMCS_REGION.get()).set_revision_id(revision);
    }
    Ok(())
}

unsafe fn ensure_feature_control() -> Result<(), &'static str> {
    let feature_control = unsafe { rdmsr(IA32_FEATURE_CONTROL) };
    if feature_control & FEATURE_CONTROL_LOCKED == 0 {
        return Err("IA32_FEATURE_CONTROL is not locked");
    }
    if feature_control & FEATURE_CONTROL_VMXON_OUTSIDE_SMX == 0 {
        return Err("VMX disabled outside SMX");
    }
    Ok(())
}

/// Enters VMX operation.
///
/// # Safety
///
/// `region` must be the physical address of an initialized VMXON region.
pub unsafe fn vmxon(region: u64) -> bool {
    let failed: u64;
    unsafe {
        asm!(
            "vmxon [{0}]",
            "setc {1:l}",
            in(reg) &region,
            out(reg) failed,
        );
    }
    failed == 0
}

/// Leaves VMX operation.
///
/// # Safety
///
/// Caller must be in VMX operation mode.
pub unsafe fn vmxoff() -> bool {
    let failed: u64;
    unsafe {
        asm!(
            "vmxoff",
            "setc {0:l}",
            out(reg) failed,
        );
    }
    failed == 0
}

/// Makes the VMCS at the given physical address current.
///
/// # Safety
///
/// `vmcs_pa` must be the physical address of an initialized VMCS region.
pub unsafe fn vmptrld(vmcs_pa: u64) -> bool {
    let failed: u64;
    unsafe {
        asm!(
            "vmptrld [{0}]",
            "setc {1:l}",
            in(reg) &vmcs_pa,
            out(reg) failed,
        );
    }
    failed == 0
}

#[cfg(test)]
mod tests {
    use super::super::msr::vmx_revision_id;

    #[test]
    fn revision_id_uses_low_31_bits() {
        assert_eq!(vmx_revision_id(0x8000_0123), 0x123);
    }
}
