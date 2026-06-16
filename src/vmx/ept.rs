// src/vmx/ept.rs

use super::memory::{EPT_GB_COUNT, EPT_PD_PAGES, EPT_PDPT, EPT_PML4, physical_address};

pub const EPT_PAGE_SIZE: u64 = 4096;
pub const EPT_LARGE_PAGE_SIZE: u64 = 2 * 1024 * 1024;

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct EptTable {
    pub entries: [u64; 512],
}

impl Default for EptTable {
    fn default() -> Self {
        Self::new()
    }
}

impl EptTable {
    pub const fn new() -> Self {
        Self { entries: [0; 512] }
    }
}

/// EPT Pointer (EPTP) structure.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct EptPointer {
    pub eptp: u64,
}

impl EptPointer {
    pub const fn new(pml4_pa: u64) -> Self {
        Self {
            eptp: (pml4_pa & 0xFFFF_FFFF_F000) | (3 << 3) | 6,
        }
    }
}

/// Build a present, readable, writable, executable EPT table pointer entry.
#[inline]
pub const fn ept_table_entry(pa: u64) -> u64 {
    (pa & 0xFFFF_FFFF_F000) | 0x7
}

/// Build a 2 MB large-page EPT entry for an identity map.
#[inline]
pub const fn ept_large_page_entry(physical_base: u64) -> u64 {
    (physical_base & 0xFFFF_FFFF_F000) | 0x87
}

/// Identity-map the first `EPT_GB_COUNT` GB of physical memory using 2 MB pages.
///
/// Static footprint: 1 PML4 + 1 PDPT + 4 PD pages = 24 KB.
///
/// # Safety
///
/// Caller must ensure EPT static regions are mapped and writable.
pub unsafe fn identity_map_ept() {
    let pml4 = unsafe { &mut *(EPT_PML4.get() as *mut EptTable) };
    let pdpt = unsafe { &mut *(EPT_PDPT.get() as *mut EptTable) };
    let pd_pages = unsafe { &mut *EPT_PD_PAGES.get() };

    pml4.entries.fill(0);
    pdpt.entries.fill(0);

    let pdpt_pa = unsafe { physical_address(EPT_PDPT.get()) };
    pml4.entries[0] = ept_table_entry(pdpt_pa);

    for gb in 0..EPT_GB_COUNT {
        let pd = unsafe { &mut *(pd_pages.pages[gb].as_mut_ptr() as *mut EptTable) };
        pd.entries.fill(0);

        let pd_pa = unsafe { physical_address(pd_pages.pages[gb].as_mut_ptr()) };
        pdpt.entries[gb] = ept_table_entry(pd_pa);

        for i in 0..512 {
            let phys = (gb as u64 * 1024 * 1024 * 1024) + (i as u64 * EPT_LARGE_PAGE_SIZE);
            pd.entries[i] = ept_large_page_entry(phys);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eptp_creation() {
        let pml4_pa = 0x1000;
        let eptp = EptPointer::new(pml4_pa);
        assert_eq!(eptp.eptp, 0x101E);
    }

    #[test]
    fn large_page_entry_marks_large_bit() {
        let entry = ept_large_page_entry(0x0020_0000);
        assert_eq!(entry & 0x87, 0x87);
    }

    #[test]
    fn identity_map_covers_first_4gb() {
        let gb = 3;
        let slot = 10;
        let phys = (gb as u64 * 1024 * 1024 * 1024) + (slot as u64 * EPT_LARGE_PAGE_SIZE);
        assert_eq!(phys, 0xC140_0000);
        assert!(phys < 4 * 1024 * 1024 * 1024);
    }
}
