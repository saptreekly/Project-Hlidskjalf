// src/vmx/state/capture.rs

use super::{CpuState, SegmentState, flat_data_segment, segment_access_rights};
use crate::vmx::msr::{IA32_EFER, rdmsr};
use core::arch::asm;

#[repr(C, packed)]
struct DescriptorTable {
    limit: u16,
    base: u64,
}

macro_rules! read_segment {
    ($seg:tt) => {{
        let selector: u16;
        unsafe {
            asm!(
                concat!("mov {0:x}, ", stringify!($seg)),
                out(reg) selector,
                options(nomem, nostack, preserves_flags),
            );
        }
        selector
    }};
}

fn read_tr() -> u16 {
    let selector: u16;
    unsafe {
        asm!(
            "str {0:x}",
            out(reg) selector,
            options(nomem, nostack, preserves_flags),
        );
    }
    selector
}

fn read_ldtr() -> u16 {
    let selector: u16;
    unsafe {
        asm!(
            "sldt {0:x}",
            out(reg) selector,
            options(nomem, nostack, preserves_flags),
        );
    }
    selector
}

fn read_fs_base() -> u64 {
    unsafe { rdmsr(0xC0000100) }
}

fn read_gs_base() -> u64 {
    unsafe { rdmsr(0xC0000101) }
}

/// Capture the current CPU state for use as both guest and host VMCS fields.
///
/// # Safety
///
/// Must be called while running on the CPU that will be virtualized.
pub unsafe fn capture_cpu_state() -> CpuState {
    let mut cr0: u64;
    let mut cr3: u64;
    let mut cr4: u64;
    let mut dr7: u64;
    let mut rsp: u64;
    let mut rflags: u64;
    let rip: u64;

    unsafe {
        asm!("mov {}, cr0", out(reg) cr0);
        asm!("mov {}, cr3", out(reg) cr3);
        asm!("mov {}, cr4", out(reg) cr4);
        asm!("mov {}, dr7", out(reg) dr7);
        asm!("mov {}, rsp", out(reg) rsp);
        asm!("pushfq");
        asm!("pop {}", out(reg) rflags);
        asm!(
            "lea {}, [rip + 2]",
            "2:",
            out(reg) rip,
        );
    }

    let cs = SegmentState {
        selector: read_segment!(cs),
        base: 0,
        limit: 0xFFFF_FFFF,
        access_rights: 0xA09B,
    };
    let ss = flat_data_segment(read_segment!(ss));
    let ds = flat_data_segment(read_segment!(ds));
    let es = flat_data_segment(read_segment!(es));
    let fs = SegmentState {
        selector: read_segment!(fs),
        base: read_fs_base(),
        limit: 0xFFFF_FFFF,
        access_rights: segment_access_rights(read_segment!(fs)),
    };
    let gs = SegmentState {
        selector: read_segment!(gs),
        base: read_gs_base(),
        limit: 0xFFFF_FFFF,
        access_rights: segment_access_rights(read_segment!(gs)),
    };
    let tr = SegmentState {
        selector: read_tr(),
        base: 0,
        limit: 0x67,
        access_rights: 0x008B,
    };
    let ldtr = SegmentState {
        selector: read_ldtr(),
        base: 0,
        limit: 0,
        access_rights: 1 << 16,
    };

    let mut gdtr = DescriptorTable { limit: 0, base: 0 };
    let mut idtr = DescriptorTable { limit: 0, base: 0 };
    unsafe {
        asm!("sgdt [{}]", in(reg) &raw mut gdtr, options(nostack));
        asm!("sidt [{}]", in(reg) &raw mut idtr, options(nostack));
    }

    CpuState {
        cr0,
        cr3,
        cr4,
        dr7,
        rip,
        rsp,
        rflags,
        cs,
        ss,
        ds,
        es,
        fs,
        gs,
        tr,
        ldtr,
        gdtr_base: gdtr.base,
        gdtr_limit: gdtr.limit as u32,
        idtr_base: idtr.base,
        idtr_limit: idtr.limit as u32,
        efer: unsafe { rdmsr(IA32_EFER) },
    }
}
