// src/vmx/config.rs

use super::memory::VMCS_REGION;
use super::init::vmptrld;
use super::vmcs::{vmwrite, encoding};

/// Initializes the VMCS for the guest.
pub unsafe fn setup_vmcs() -> Result<(), &'static str> {
    // 1. Load the VMCS
    let vmcs_pa = VMCS_REGION.get() as u64;
    unsafe {
        if !vmptrld(vmcs_pa) {
            return Err("VMPTRLD failed");
        }
    }

    // 2. Initialize critical guest state
    // Note: In a real hypervisor, these would be captured from the 
    // current context or the state we want the guest to boot in.
    vmwrite(encoding::GUEST_RIP, 0x00000000); // Placeholder
    vmwrite(encoding::GUEST_RSP, 0x00000000); // Placeholder

    // 3. Initialize critical host state
    // We need to point the CPU to our host entry point and stack when a VM exit occurs.
    // vmwrite(encoding::HOST_RIP, host_entry_point as u64);
    // vmwrite(encoding::HOST_RSP, host_stack_pointer as u64);

    Ok(())
}
