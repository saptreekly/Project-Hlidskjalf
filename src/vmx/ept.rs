// src/vmx/ept.rs

use super::memory::{EPT_PD, EPT_PDPT, EPT_PML4, EPT_PT};

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct EptTable {
    pub entries: [u64; 512],
}

impl EptTable {
    pub const fn new() -> Self {
        Self { entries: [0; 512] }
    }
}

/// EPT Pointer (EPTP) structure
#[repr(C)]
#[derive(Clone, Copy)]
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

pub unsafe fn identity_map_ept() {
    let pml4 = unsafe { &mut *(EPT_PML4.get() as *mut EptTable) };
    let pdpt = unsafe { &mut *(EPT_PDPT.get() as *mut EptTable) };
    let pd = unsafe { &mut *(EPT_PD.get() as *mut EptTable) };
    let pt = unsafe { &mut *(EPT_PT.get() as *mut EptTable) };

    // Identity map: Read + Write + Execute (Bits 0-2 = 7)
    // Entry points to physical address of next table/page

    pml4.entries[0] = (EPT_PDPT.get() as u64) | 0x7;
    pdpt.entries[0] = (EPT_PD.get() as u64) | 0x7;
    pd.entries[0] = (EPT_PT.get() as u64) | 0x7;

    for i in 0..512 {
        pt.entries[i] = ((i as u64) * 0x1000) | 0x7; // 4KB pages
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
