// src/vmx/vmlaunch.rs

#[cfg(not(feature = "sim"))]
use super::vmcs::vm_instruction_error;
#[cfg(not(feature = "sim"))]
use core::arch::asm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmxLaunchError {
    VmlaunchFailed(u32),
}

/// Execute `VMLAUNCH`. On success, guest execution begins and this does not return.
///
/// # Safety
///
/// The current VMCS must be fully configured and loaded.
pub unsafe fn vmlaunch() -> Result<(), VmxLaunchError> {
    #[cfg(feature = "sim")]
    {
        super::sim::vmlaunch()
    }
    #[cfg(not(feature = "sim"))]
    {
        let failed: u64;
        unsafe {
            asm!(
                "vmlaunch",
                "setc {0:l}",
                out(reg) failed,
            );
        }

        if failed != 0 {
            return Err(VmxLaunchError::VmlaunchFailed(vm_instruction_error()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_error_carries_instruction_error_code() {
        let err = VmxLaunchError::VmlaunchFailed(7);
        assert_eq!(err, VmxLaunchError::VmlaunchFailed(7));
    }
}
