#![cfg_attr(not(any(test, feature = "fuzzing")), no_std)]
#![cfg_attr(not(test), no_main)]

// Project Hliðskjálf - Type-1.5 Thin Hypervisor
// Core Architecture: Intel VT-x
// Implementation: #![no_std] bare-metal Rust

pub mod vmx;

use core::arch::x86_64::__cpuid;
use vmx::config::setup_vmcs;
use vmx::init::enable_vmx;
use vmx::vmlaunch::vmlaunch;

#[cfg(not(any(test, feature = "fuzzing")))]
use core::panic::PanicInfo;

#[cfg(not(any(test, feature = "fuzzing")))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Verify if the CPU supports Intel VT-x (VMX)
pub fn check_vmx_support() -> bool {
    // CPUID leaf 1: Feature Information
    let cpuid = __cpuid(1);

    // VMX is bit 5 of ECX
    let vmx_bit = (cpuid.ecx >> 5) & 1;

    vmx_bit == 1
}

// Minimal Windows Driver Types
#[repr(C)]
pub struct DriverObject([u8; 0]);

#[repr(C)]
pub struct UnicodeString([u8; 0]);

/// Windows Kernel Driver Entry Point
#[unsafe(no_mangle)]
#[allow(unreachable_code)]
pub extern "system" fn DriverEntry(
    _driver_object: *mut DriverObject,
    _registry_path: *mut UnicodeString,
) -> i32 {
    unsafe {
        // 1. Enable VMX
        if enable_vmx().is_err() {
            return 0xC00000BBu32 as i32; // STATUS_NOT_SUPPORTED
        }

        // 2. Configure VMCS
        if setup_vmcs().is_err() {
            return 0xC00000BBu32 as i32; // STATUS_NOT_SUPPORTED
        }

        // 3. Launch VM
        vmlaunch();
    }

    0 // STATUS_SUCCESS
}
