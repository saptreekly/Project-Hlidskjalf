// src/vmx/vmcs.rs

use core::arch::asm;

pub mod encoding {
    pub const VM_INSTRUCTION_ERROR: u32 = 0x0000_4400;
    pub const VM_EXIT_REASON: u32 = 0x0000_4402;
    pub const VM_EXIT_QUALIFICATION: u32 = 0x0000_6400;
    pub const GUEST_RIP: u32 = 0x0000_681E;
    pub const GUEST_RSP: u32 = 0x0000_681C;
    pub const GUEST_RFLAGS: u32 = 0x0000_6820;
    pub const GUEST_CR0: u32 = 0x0000_6800;
    pub const GUEST_CR3: u32 = 0x0000_6802;
    pub const GUEST_CR4: u32 = 0x0000_6804;
    pub const GUEST_DR7: u32 = 0x0000_681A;
    pub const GUEST_CS_SELECTOR: u32 = 0x0000_0802;
    pub const GUEST_SS_SELECTOR: u32 = 0x0000_0804;
    pub const GUEST_DS_SELECTOR: u32 = 0x0000_0806;
    pub const GUEST_ES_SELECTOR: u32 = 0x0000_0800;
    pub const GUEST_FS_SELECTOR: u32 = 0x0000_0808;
    pub const GUEST_GS_SELECTOR: u32 = 0x0000_080A;
    pub const GUEST_TR_SELECTOR: u32 = 0x0000_080E;
    pub const GUEST_LDTR_SELECTOR: u32 = 0x0000_080C;
    pub const GUEST_CS_BASE: u32 = 0x0000_6808;
    pub const GUEST_SS_BASE: u32 = 0x0000_680A;
    pub const GUEST_DS_BASE: u32 = 0x0000_680C;
    pub const GUEST_ES_BASE: u32 = 0x0000_6806;
    pub const GUEST_FS_BASE: u32 = 0x0000_680E;
    pub const GUEST_GS_BASE: u32 = 0x0000_6810;
    pub const GUEST_TR_BASE: u32 = 0x0000_6814;
    pub const GUEST_LDTR_BASE: u32 = 0x0000_6812;
    pub const GUEST_CS_LIMIT: u32 = 0x0000_4802;
    pub const GUEST_SS_LIMIT: u32 = 0x0000_4804;
    pub const GUEST_DS_LIMIT: u32 = 0x0000_4806;
    pub const GUEST_ES_LIMIT: u32 = 0x0000_4800;
    pub const GUEST_FS_LIMIT: u32 = 0x0000_4808;
    pub const GUEST_GS_LIMIT: u32 = 0x0000_480A;
    pub const GUEST_TR_LIMIT: u32 = 0x0000_480E;
    pub const GUEST_LDTR_LIMIT: u32 = 0x0000_480C;
    pub const GUEST_CS_ACCESS_RIGHTS: u32 = 0x0000_4816;
    pub const GUEST_SS_ACCESS_RIGHTS: u32 = 0x0000_4818;
    pub const GUEST_DS_ACCESS_RIGHTS: u32 = 0x0000_481A;
    pub const GUEST_ES_ACCESS_RIGHTS: u32 = 0x0000_4814;
    pub const GUEST_FS_ACCESS_RIGHTS: u32 = 0x0000_481C;
    pub const GUEST_GS_ACCESS_RIGHTS: u32 = 0x0000_481E;
    pub const GUEST_TR_ACCESS_RIGHTS: u32 = 0x0000_4822;
    pub const GUEST_LDTR_ACCESS_RIGHTS: u32 = 0x0000_4820;
    pub const GUEST_GDTR_BASE: u32 = 0x0000_6816;
    pub const GUEST_IDTR_BASE: u32 = 0x0000_6818;
    pub const GUEST_GDTR_LIMIT: u32 = 0x0000_4810;
    pub const GUEST_IDTR_LIMIT: u32 = 0x0000_480E;
    pub const GUEST_ACTIVITY_STATE: u32 = 0x0000_4826;
    pub const GUEST_INTERRUPTIBILITY: u32 = 0x0000_4824;
    pub const GUEST_PENDING_DEBUG: u32 = 0x0000_6822;
    pub const HOST_CR0: u32 = 0x0000_6C00;
    pub const HOST_CR3: u32 = 0x0000_6C02;
    pub const HOST_CR4: u32 = 0x0000_6C04;
    pub const HOST_CS_SELECTOR: u32 = 0x0000_0C02;
    pub const HOST_SS_SELECTOR: u32 = 0x0000_0C00;
    pub const HOST_DS_SELECTOR: u32 = 0x0000_0C04;
    pub const HOST_ES_SELECTOR: u32 = 0x0000_0C06;
    pub const HOST_FS_SELECTOR: u32 = 0x0000_0C08;
    pub const HOST_GS_SELECTOR: u32 = 0x0000_0C0A;
    pub const HOST_TR_SELECTOR: u32 = 0x0000_0C0C;
    pub const HOST_FS_BASE: u32 = 0x0000_6C06;
    pub const HOST_GS_BASE: u32 = 0x0000_6C08;
    pub const HOST_TR_BASE: u32 = 0x0000_6C0A;
    pub const HOST_GDTR_BASE: u32 = 0x0000_6C0C;
    pub const HOST_IDTR_BASE: u32 = 0x0000_6C0E;
    pub const HOST_RIP: u32 = 0x0000_6C16;
    pub const HOST_RSP: u32 = 0x0000_6C14;
    pub const PIN_BASED_VM_EXEC_CONTROL: u32 = 0x0000_4000;
    pub const CPU_BASED_VM_EXEC_CONTROL: u32 = 0x0000_4002;
    pub const SECONDARY_VM_EXEC_CONTROL: u32 = 0x0000_401E;
    pub const VM_EXIT_CONTROLS: u32 = 0x0000_400C;
    pub const VM_ENTRY_CONTROLS: u32 = 0x0000_4012;
    pub const EPT_POINTER: u32 = 0x0000_201A;
    pub const VMCS_LINK_POINTER: u32 = 0x0000_2800;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmcsError {
    VmreadFailed,
    VmwriteFailed,
}

/// Read a field from the current VMCS.
#[inline]
pub fn vmread(field: u32) -> Result<u64, VmcsError> {
    let value: u64;
    let failed: u64;
    unsafe {
        asm!(
            "vmread {1}, {0}",
            "setc {2:l}",
            in(reg) field as u64,
            out(reg) value,
            out(reg) failed,
            options(nostack, preserves_flags),
        );
    }
    if failed != 0 {
        Err(VmcsError::VmreadFailed)
    } else {
        Ok(value)
    }
}

/// Write a field to the current VMCS.
#[inline]
pub fn vmwrite(field: u32, value: u64) -> Result<(), VmcsError> {
    let failed: u64;
    unsafe {
        asm!(
            "vmwrite {1}, {0}",
            "setc {2:l}",
            in(reg) field as u64,
            in(reg) value,
            out(reg) failed,
            options(nostack, preserves_flags),
        );
    }
    if failed != 0 {
        Err(VmcsError::VmwriteFailed)
    } else {
        Ok(())
    }
}

/// Read the VM-instruction error field after a failed VMX instruction.
#[inline]
pub fn vm_instruction_error() -> u32 {
    vmread(encoding::VM_INSTRUCTION_ERROR).unwrap_or(0) as u32
}

#[cfg(test)]
mod tests {
    use super::encoding;

    #[test]
    fn encodings_are_unique_for_core_fields() {
        assert_ne!(encoding::GUEST_RIP, encoding::HOST_RIP);
        assert_ne!(encoding::GUEST_RSP, encoding::HOST_RSP);
        assert_ne!(encoding::VM_EXIT_REASON, encoding::VM_INSTRUCTION_ERROR);
    }
}
