// src/vmx/init.rs

use super::memory::VMXON_REGION;
use core::arch::asm;

// Declare the Windows Kernel function signature
unsafe extern "system" {
    fn MmGetPhysicalAddress(BaseAddress: *mut core::ffi::c_void) -> u64;
}

/// Enables VMX operation in the CPU.
///
/// # Safety
///
/// Caller must ensure that the CPU is in a state capable of entering VMX mode,
/// and that `VMXON_REGION` is correctly initialized.
pub unsafe fn enable_vmx() -> Result<(), &'static str> {
    // 1. Enable VMX bit in CR4
    let mut cr4: u64;
    unsafe {
        asm!("mov {}, cr4", out(reg) cr4);
        cr4 |= 1 << 13; // Bit 13 is VMXE
        asm!("mov cr4, {}", in(reg) cr4);
    }

    // 2. Get the physical address of the VMXON region
    let virtual_ptr = VMXON_REGION.get() as *mut core::ffi::c_void;
    let physical_address = unsafe { MmGetPhysicalAddress(virtual_ptr) };

    // 3. Execute VMXON
    unsafe {
        if !vmxon(physical_address) {
            return Err("VMXON failed");
        }
    }

    Ok(())
}

/// Enters VMX Operation.
/// The `region` must be a 4KB-aligned physical address of the VMXON region.
///
/// # Safety
///
/// Caller must ensure `region` points to a valid, 4KB-aligned VMXON region.
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
///
/// # Safety
///
/// Caller must be in VMX operation mode.
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
///
/// # Safety
///
/// Caller must ensure `vmcs_pa` points to a valid, 4KB-aligned VMCS region.
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
