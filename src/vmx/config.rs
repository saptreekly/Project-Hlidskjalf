// src/vmx/config.rs

use core::arch::asm;
use super::ept::{EptPointer, identity_map_ept};
use super::init::vmptrld;
use super::memory::{EPT_PML4, VMCS_REGION};
use super::vmcs::{encoding, vmwrite};

/// Initializes the VMCS for the guest.
///
/// # Safety
///
/// Caller must ensure that `VMCS_REGION` and `EPT` paging structures are initialized
/// and that the CPU is in a valid state to configure the VMCS.
pub unsafe fn setup_vmcs() -> Result<(), &'static str> {
    // 1. Load the VMCS
    let vmcs_pa = VMCS_REGION.get() as u64;
    unsafe {
        if !vmptrld(vmcs_pa) {
            return Err("VMPTRLD failed");
        }
    }

    // 2. Setup EPT Paging
    unsafe { identity_map_ept() };
    let eptp = EptPointer::new(EPT_PML4.get() as u64);
    vmwrite(encoding::EPT_POINTER, eptp.eptp);

    // 3. Initialize critical guest state
    // Note: In a real hypervisor, these would be captured from the
    // current context or the state we want the guest to boot in.
    vmwrite(encoding::GUEST_RIP, 0x00000000); // Placeholder
    vmwrite(encoding::GUEST_RSP, 0x00000000); // Placeholder

    // 4. Initialize critical host state
    // We need to point the CPU to our host entry point and stack when a VM exit occurs.
    let mut cr3: u64;
    unsafe { asm!("mov {}, cr3", out(reg) cr3) };
    vmwrite(encoding::HOST_CR3, cr3);

    // Placeholder: Need to capture current RIP and RSP to return to
    vmwrite(encoding::HOST_RIP, 0x00000000); // Should be a label in our assembly
    vmwrite(encoding::HOST_RSP, 0x00000000); // Should be the host stack

    Ok(())
}
