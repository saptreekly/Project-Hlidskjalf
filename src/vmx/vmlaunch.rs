// src/vmx/vmlaunch.rs

use core::arch::asm;

/// Executes the VMLAUNCH instruction to run the guest.
/// Should only be called after VMCS is loaded and fully configured.
///
/// # Safety
///
/// Caller must ensure that the VMCS is properly configured and loaded (via `vmptrld`).
pub unsafe fn vmlaunch() -> ! {
    unsafe {
        asm!(
            "vmlaunch",
            "1: jmp 1b", // If VMLAUNCH succeeds, this line is not reached (VM entry occurs)
            options(noreturn)
        );
    }
}
