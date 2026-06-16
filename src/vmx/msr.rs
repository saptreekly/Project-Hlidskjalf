// src/vmx/msr.rs
//! MSR constants and helpers for VMX bring-up.

use core::arch::asm;

pub const IA32_FEATURE_CONTROL: u32 = 0x3A;
pub const IA32_VMX_BASIC: u32 = 0x480;
pub const IA32_VMX_CR0_FIXED0: u32 = 0x486;
pub const IA32_VMX_CR0_FIXED1: u32 = 0x487;
pub const IA32_VMX_CR4_FIXED0: u32 = 0x488;
pub const IA32_VMX_CR4_FIXED1: u32 = 0x489;
pub const IA32_VMX_PROCBASED_CTLS: u32 = 0x482;
pub const IA32_VMX_PROCBASED_CTLS2: u32 = 0x48B;
pub const IA32_VMX_PINBASED_CTLS: u32 = 0x481;
pub const IA32_VMX_EXIT_CTLS: u32 = 0x483;
pub const IA32_VMX_ENTRY_CTLS: u32 = 0x484;
pub const IA32_VMX_VMCS_ENUM: u32 = 0x48A;
pub const IA32_VMX_EPT_VPID_CAP: u32 = 0x48C;
pub const IA32_EFER: u32 = 0xC0000080;

pub const FEATURE_CONTROL_VMXON_OUTSIDE_SMX: u64 = 1 << 2;
pub const FEATURE_CONTROL_LOCKED: u64 = 1 << 0;

pub const VMX_PROCBASED_USE_SECONDARY: u64 = 1 << 31;
pub const VMX_PROCBASED2_ENABLE_EPT: u64 = 1 << 1;

/// Read a model-specific register.
#[inline]
pub fn rdmsr(msr: u32) -> u64 {
    #[cfg(feature = "sim")]
    {
        super::sim::read_msr(msr)
    }
    #[cfg(not(feature = "sim"))]
    {
        rdmsr_hw(msr)
    }
}

#[inline]
#[cfg_attr(feature = "sim", allow(dead_code))]
fn rdmsr_hw(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags),
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Write a model-specific register.
///
/// # Safety
///
/// The MSR index and value must be valid on the current CPU.
#[inline]
pub unsafe fn wrmsr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
            options(nomem, nostack, preserves_flags),
        );
    }
}

/// VMX revision ID from `IA32_VMX_BASIC` (bits 30:0).
#[inline]
pub fn vmx_revision_id(basic: u64) -> u32 {
    (basic & 0x7FFF_FFFF) as u32
}

/// Adjust a control field using the VMX capability MSR format.
#[inline]
pub fn adjust_vmx_control_values(current: u64, cap: u64) -> u64 {
    let allowed0 = cap & 0xFFFF_FFFF;
    let allowed1 = cap >> 32;
    (current & allowed1) | allowed0
}

/// Adjust a control field using the VMX "true capability" MSR format.
#[inline]
pub fn adjust_vmx_control(current: u64, msr: u32) -> u64 {
    let cap = rdmsr(msr);
    adjust_vmx_control_values(current, cap)
}

/// Apply Intel fixed CR0 bits required for VMX operation.
#[inline]
pub fn adjust_cr0_for_vmx(cr0: u64) -> u64 {
    let fixed0 = rdmsr(IA32_VMX_CR0_FIXED0);
    let fixed1 = rdmsr(IA32_VMX_CR0_FIXED1);
    (cr0 | fixed0) & fixed1
}

/// Apply Intel fixed CR4 bits required for VMX operation.
#[inline]
pub fn adjust_cr4_for_vmx(cr4: u64) -> u64 {
    let fixed0 = rdmsr(IA32_VMX_CR4_FIXED0);
    let fixed1 = rdmsr(IA32_VMX_CR4_FIXED1);
    (cr4 | fixed0) & fixed1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vmx_revision_id_masks_upper_bits() {
        let basic = 0xDA0400600_u64;
        assert_eq!(vmx_revision_id(basic), (basic & 0x7FFF_FFFF) as u32);
    }

    #[test]
    fn adjust_vmx_control_applies_allowed_bits() {
        let cap = 0xFFFF_FFFF_0000_0022_u64;
        let result = adjust_vmx_control_values(0, cap);
        assert_eq!(result, 0x22);
    }
}
