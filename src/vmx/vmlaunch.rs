// src/vmx/vmlaunch.rs

use core::arch::asm;

/// Executes the VMLAUNCH instruction to run the guest.
/// Should only be called after VMCS is loaded and fully configured.
pub unsafe fn vmlaunch() -> ! {
    unsafe {
        asm!(
            "vmlaunch",
            "jmp .", // If VMLAUNCH succeeds, this line is not reached (VM entry occurs)
            options(noreturn)
        );
    }
}
