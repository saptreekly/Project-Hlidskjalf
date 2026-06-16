#![cfg_attr(not(any(test, feature = "fuzzing", feature = "sim")), no_std)]
#![cfg_attr(not(test), no_main)]

// Project Hliðskjálf - Type-1.5 Thin Hypervisor
// Core Architecture: Intel VT-x
// Implementation: #![no_std] bare-metal Rust

pub mod vmx;

#[cfg(feature = "sim")]
pub use vmx::sim;

use core::arch::x86_64::__cpuid;
use vmx::config::setup_vmcs;
use vmx::init::enable_vmx;
use vmx::vmlaunch::{VmxLaunchError, vmlaunch};

#[cfg(not(any(test, feature = "fuzzing", feature = "sim")))]
use core::panic::PanicInfo;

pub const STATUS_SUCCESS: i32 = 0;
pub const STATUS_NOT_SUPPORTED: i32 = 0xC000_00BB_u32 as i32;
pub const STATUS_UNSUCCESSFUL: i32 = 0xC000_0001_u32 as i32;

#[cfg(not(any(test, feature = "fuzzing", feature = "sim")))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Returns whether CPUID leaf 1 reports VMX support in ECX bit 5.
pub fn check_vmx_support() -> bool {
    check_vmx_support_from_ecx(read_cpuid_ecx(1))
}

/// Testable VMX capability check from CPUID leaf 1 ECX bits.
pub fn check_vmx_support_from_ecx(ecx: u32) -> bool {
    ((ecx >> 5) & 1) == 1
}

#[inline]
fn read_cpuid_ecx(leaf: u32) -> u32 {
    #[cfg(feature = "sim")]
    if let Some(ecx) = vmx::sim::cpuid_ecx(leaf) {
        return ecx;
    }
    __cpuid(leaf).ecx
}

// Minimal Windows Driver Types
#[repr(C)]
pub struct DriverObject([u8; 0]);

#[repr(C)]
pub struct UnicodeString([u8; 0]);

/// Windows Kernel Driver Entry Point
#[unsafe(no_mangle)]
pub extern "system" fn DriverEntry(
    _driver_object: *mut DriverObject,
    _registry_path: *mut UnicodeString,
) -> i32 {
    if !check_vmx_support() {
        return STATUS_NOT_SUPPORTED;
    }

    unsafe {
        if enable_vmx().is_err() {
            return STATUS_NOT_SUPPORTED;
        }

        if setup_vmcs().is_err() {
            return STATUS_NOT_SUPPORTED;
        }

        if let Err(VmxLaunchError::VmlaunchFailed(_)) = vmlaunch() {
            return STATUS_UNSUCCESSFUL;
        }
    }

    STATUS_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vmx_bit_set_when_ecx_has_bit_5() {
        assert!(check_vmx_support_from_ecx(1 << 5));
    }

    #[test]
    fn vmx_bit_clear_when_ecx_is_zero() {
        assert!(!check_vmx_support_from_ecx(0));
    }
}
