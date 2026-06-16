// src/vmx/state.rs

use super::vmcs::{encoding, vmwrite};

#[repr(C, packed)]
pub(super) struct DescriptorTable {
    limit: u16,
    base: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SegmentState {
    pub selector: u16,
    pub base: u64,
    pub limit: u32,
    pub access_rights: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CpuState {
    pub cr0: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub dr7: u64,
    pub rip: u64,
    pub rsp: u64,
    pub rflags: u64,
    pub cs: SegmentState,
    pub ss: SegmentState,
    pub ds: SegmentState,
    pub es: SegmentState,
    pub fs: SegmentState,
    pub gs: SegmentState,
    pub tr: SegmentState,
    pub ldtr: SegmentState,
    pub gdtr_base: u64,
    pub gdtr_limit: u32,
    pub idtr_base: u64,
    pub idtr_limit: u32,
    pub efer: u64,
}

#[cfg(windows)]
mod capture;

#[cfg(windows)]
pub use capture::capture_cpu_state;

#[cfg(not(windows))]
pub unsafe fn capture_cpu_state() -> CpuState {
    CpuState::default()
}

pub(super) fn segment_access_rights(selector: u16) -> u32 {
    if selector == 0 {
        return 1 << 16;
    }

    let mut rights: u32 = 0xA000;
    if selector & 0x4 != 0 {
        rights |= 1 << 16;
    }
    rights
}

pub(super) fn flat_data_segment(selector: u16) -> SegmentState {
    SegmentState {
        selector,
        base: 0,
        limit: 0xFFFF_FFFF,
        access_rights: segment_access_rights(selector),
    }
}

fn write_guest_segment(
    selector: u32,
    base: u32,
    limit: u32,
    access_rights: u32,
    segment: SegmentState,
) -> Result<(), super::vmcs::VmcsError> {
    vmwrite(selector, segment.selector as u64)?;
    vmwrite(base, segment.base)?;
    vmwrite(limit, segment.limit as u64)?;
    vmwrite(access_rights, segment.access_rights as u64)?;
    Ok(())
}

/// Write guest CPU state into the current VMCS.
pub fn write_guest_state(state: &CpuState) -> Result<(), super::vmcs::VmcsError> {
    vmwrite(encoding::GUEST_CR0, state.cr0)?;
    vmwrite(encoding::GUEST_CR3, state.cr3)?;
    vmwrite(encoding::GUEST_CR4, state.cr4)?;
    vmwrite(encoding::GUEST_DR7, state.dr7)?;
    vmwrite(encoding::GUEST_RIP, state.rip)?;
    vmwrite(encoding::GUEST_RSP, state.rsp)?;
    vmwrite(encoding::GUEST_RFLAGS, state.rflags)?;

    write_guest_segment(
        encoding::GUEST_ES_SELECTOR,
        encoding::GUEST_ES_BASE,
        encoding::GUEST_ES_LIMIT,
        encoding::GUEST_ES_ACCESS_RIGHTS,
        state.es,
    )?;
    write_guest_segment(
        encoding::GUEST_CS_SELECTOR,
        encoding::GUEST_CS_BASE,
        encoding::GUEST_CS_LIMIT,
        encoding::GUEST_CS_ACCESS_RIGHTS,
        state.cs,
    )?;
    write_guest_segment(
        encoding::GUEST_SS_SELECTOR,
        encoding::GUEST_SS_BASE,
        encoding::GUEST_SS_LIMIT,
        encoding::GUEST_SS_ACCESS_RIGHTS,
        state.ss,
    )?;
    write_guest_segment(
        encoding::GUEST_DS_SELECTOR,
        encoding::GUEST_DS_BASE,
        encoding::GUEST_DS_LIMIT,
        encoding::GUEST_DS_ACCESS_RIGHTS,
        state.ds,
    )?;
    write_guest_segment(
        encoding::GUEST_FS_SELECTOR,
        encoding::GUEST_FS_BASE,
        encoding::GUEST_FS_LIMIT,
        encoding::GUEST_FS_ACCESS_RIGHTS,
        state.fs,
    )?;
    write_guest_segment(
        encoding::GUEST_GS_SELECTOR,
        encoding::GUEST_GS_BASE,
        encoding::GUEST_GS_LIMIT,
        encoding::GUEST_GS_ACCESS_RIGHTS,
        state.gs,
    )?;
    write_guest_segment(
        encoding::GUEST_LDTR_SELECTOR,
        encoding::GUEST_LDTR_BASE,
        encoding::GUEST_LDTR_LIMIT,
        encoding::GUEST_LDTR_ACCESS_RIGHTS,
        state.ldtr,
    )?;
    write_guest_segment(
        encoding::GUEST_TR_SELECTOR,
        encoding::GUEST_TR_BASE,
        encoding::GUEST_TR_LIMIT,
        encoding::GUEST_TR_ACCESS_RIGHTS,
        state.tr,
    )?;

    vmwrite(encoding::GUEST_GDTR_BASE, state.gdtr_base)?;
    vmwrite(encoding::GUEST_GDTR_LIMIT, state.gdtr_limit as u64)?;
    vmwrite(encoding::GUEST_IDTR_BASE, state.idtr_base)?;
    vmwrite(encoding::GUEST_IDTR_LIMIT, state.idtr_limit as u64)?;
    vmwrite(encoding::GUEST_ACTIVITY_STATE, 0)?;
    vmwrite(encoding::GUEST_INTERRUPTIBILITY, 0)?;
    vmwrite(encoding::GUEST_PENDING_DEBUG, 0)?;
    Ok(())
}

/// Write host CPU state into the current VMCS.
pub fn write_host_state(
    state: &CpuState,
    host_rip: u64,
    host_rsp: u64,
) -> Result<(), super::vmcs::VmcsError> {
    vmwrite(encoding::HOST_CR0, state.cr0)?;
    vmwrite(encoding::HOST_CR3, state.cr3)?;
    vmwrite(encoding::HOST_CR4, state.cr4)?;
    vmwrite(encoding::HOST_CS_SELECTOR, state.cs.selector as u64)?;
    vmwrite(encoding::HOST_SS_SELECTOR, state.ss.selector as u64)?;
    vmwrite(encoding::HOST_DS_SELECTOR, state.ds.selector as u64)?;
    vmwrite(encoding::HOST_ES_SELECTOR, state.es.selector as u64)?;
    vmwrite(encoding::HOST_FS_SELECTOR, state.fs.selector as u64)?;
    vmwrite(encoding::HOST_GS_SELECTOR, state.gs.selector as u64)?;
    vmwrite(encoding::HOST_TR_SELECTOR, state.tr.selector as u64)?;
    vmwrite(encoding::HOST_FS_BASE, state.fs.base)?;
    vmwrite(encoding::HOST_GS_BASE, state.gs.base)?;
    vmwrite(encoding::HOST_TR_BASE, state.tr.base)?;
    vmwrite(encoding::HOST_GDTR_BASE, state.gdtr_base)?;
    vmwrite(encoding::HOST_IDTR_BASE, state.idtr_base)?;
    vmwrite(encoding::HOST_RIP, host_rip)?;
    vmwrite(encoding::HOST_RSP, host_rsp)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_selector_access_rights_are_unusable() {
        assert_eq!(segment_access_rights(0), 1 << 16);
    }

    #[test]
    fn flat_data_segment_has_full_limit() {
        let seg = flat_data_segment(0x2B);
        assert_eq!(seg.limit, 0xFFFF_FFFF);
        assert_eq!(seg.base, 0);
    }
}
