// src/vmx/init.rs

use super::memory::VMXON_REGION;
use core::arch::asm;

/// Enables VMX operation in the CPU.
pub unsafe fn enable_vmx() -> Result<(), &'static str> {
    // 1. Enable VMX bit in CR4
    let mut cr4: u64;
    unsafe {
        asm!("mov {}, cr4", out(reg) cr4);
        cr4 |= 1 << 13; // Bit 13 is VMXE
        asm!("mov cr4, {}", in(reg) cr4);
    }

    // 2. Get the physical address of the VMXON region
    // In a real kernel, we would need to get the actual physical address
    // via a platform-specific API. For this prototype, we'll assume the
    // virtual address is identity-mapped or manageable as a physical address.
    let pa = VMXON_REGION.get() as u64;

    // 3. Execute VMXON
    unsafe {
        if !vmxon(pa) {
            return Err("VMXON failed");
        }
    }

    Ok(())
}

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
