// src/vmx/ept.rs

use super::memory::{EPT_PD, EPT_PDPT, EPT_PML4};

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

/// EPT Pointer (EPTP) structure
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct EptPointer {
    pub eptp: u64,
}

impl EptPointer {
    pub const fn new(pml4_pa: u64) -> Self {
        Self {
            eptp: (pml4_pa & 0xFFFFFFFFFF000) | (3 << 3) | 6,
        }
    }
}

/// Identity maps EPT tables using 2MB Large Pages.
///
/// # Safety
///
/// Caller must ensure that the EPT tables (PML4, PDPT, PD)
/// are correctly initialized in memory and accessible.
pub unsafe fn identity_map_ept() {
    let pml4 = unsafe { &mut *(EPT_PML4.get() as *mut EptTable) };
    let pdpt = unsafe { &mut *(EPT_PDPT.get() as *mut EptTable) };
    let pd = unsafe { &mut *(EPT_PD.get() as *mut EptTable) };

    // Clear tables
    pml4.entries.fill(0);
    pdpt.entries.fill(0);
    pd.entries.fill(0);

    // Map the entire 4K physical address space (PML4->PDPT->PD)
    // 512 PDPT entries (512 * 512GB = 256TB total address space)
    for i in 0..512 {
        let pdpt_pa = (EPT_PDPT.get() as u64) + (i * 4096);
        pml4.entries[i as usize] = pdpt_pa | 0x7;

        for j in 0..512 {
            let pd_pa = (EPT_PD.get() as u64) + ((i * 512 + j) * 4096);
            pdpt.entries[j as usize] = pd_pa | 0x7;

            for k in 0..512 {
                // Large Page bit (bit 7) set for 2MB pages
                let physical_addr = (i * 512 * 512 + j * 512 + k) * 2 * 1024 * 1024;
                pd.entries[k as usize] = physical_addr | 0x87; // 0x80 (Large Page) | 0x7 (R/W/X)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eptp_creation() {
        // Test EPTP with a 4KB aligned physical address 0x1000
        // Expect (0x1000 & 0xFFFFFFFFFF000) | (3 << 3) | 6
        // = 0x1000 | 24 | 6 = 0x101E
        let pml4_pa = 0x1000;
        let eptp = EptPointer::new(pml4_pa);
        assert_eq!(eptp.eptp, 0x101E);
    }
}
