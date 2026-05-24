// src/vmx/init.rs

use core::arch::asm;

/// Enters VMX Operation.
/// The `region` must be a 4KB-aligned physical address of the VMXON region.
pub unsafe fn vmxon(region: u64) -> bool {
    let success: u64;
    unsafe {
        asm!(
            "vmxon [{0}]",
            "setz {1:l}", // Set success based on ZF
            in(reg) &region,
            out(reg) success,
        );
    }
    success != 0
}

/// Leaves VMX Operation.
pub unsafe fn vmxoff() -> bool {
    let success: u64;
    unsafe {
        asm!(
            "vmxoff",
            "setz {0:l}",
            out(reg) success,
        );
    }
    success != 0
}

/// Makes the VMCS at the given physical address current.
/// The `vmcs_pa` must be 4KB-aligned.
pub unsafe fn vmptrld(vmcs_pa: u64) -> bool {
    let success: u64;
    unsafe {
        asm!(
            "vmptrld [{0}]",
            "setz {1:l}",
            in(reg) &vmcs_pa,
            out(reg) success,
        );
    }
    success != 0
}
