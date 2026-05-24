#![no_std]
#![no_main]

// Project Hliðskjálf - Type-1.5 Thin Hypervisor
// Core Architecture: Intel VT-x
// Implementation: #![no_std] bare-metal Rust

pub mod vmx;

use core::arch::x86_64::__cpuid;
use core::panic::PanicInfo;

/// Panic handler for no_std
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
pub extern "system" fn DriverEntry(
    _driver_object: *mut DriverObject,
    _registry_path: *mut UnicodeString,
) -> i32 {
    // 0 is STATUS_SUCCESS in Windows NTSTATUS
    if check_vmx_support() {
        // VT-x supported, proceed to initialize hypervisor
        0
    } else {
        // STATUS_NOT_SUPPORTED
        0xC00000BBu32 as i32
    }
}
