// src/vmx/vmcs.rs

use core::arch::asm;

/// VMCS Field Encodings (simplified partial list)
pub mod encoding {
    pub const GUEST_RIP: u32 = 0x0000681E;
    pub const GUEST_RSP: u32 = 0x0000681C;
    pub const HOST_RIP:  u32 = 0x00006C16;
    pub const HOST_RSP:  u32 = 0x00006C14;
    pub const EPT_POINTER: u32 = 0x0000201A;
}

/// Read a field from the current VMCS
#[inline]
pub fn vmread(field: u32) -> u64 {
    let value: u64;
    unsafe {
        asm!(
            "vmread {0}, {1}",
            out(reg) value,
            in(reg) field as u64,
        );
    }
    value
}

/// Write a field to the current VMCS
#[inline]
pub fn vmwrite(field: u32, value: u64) {
    unsafe {
        asm!(
            "vmwrite {1}, {0}",
            in(reg) field as u64,
            in(reg) value,
        );
    }
}
