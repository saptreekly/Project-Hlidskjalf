#![no_std]
#![no_main]

// Project Hliðskjálf - Type-1.5 Thin Hypervisor
// Core Architecture: Intel VT-x
// Implementation: #![no_std] bare-metal Rust

pub mod vmx;

use core::panic::PanicInfo;
use vmx::config::setup_vmcs;
use vmx::init::enable_vmx;
use vmx::vmlaunch::vmlaunch;

/// Panic handler for no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
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
    unsafe {
        // 1. Enable VMX
        if let Err(_) = enable_vmx() {
            return 0xC00000BBu32 as i32; // STATUS_NOT_SUPPORTED
        }

        // 2. Configure VMCS
        if let Err(_) = setup_vmcs() {
            return 0xC00000BBu32 as i32; // STATUS_NOT_SUPPORTED
        }

        // 3. Launch VM
        vmlaunch();
    }

    0 // STATUS_SUCCESS
}
