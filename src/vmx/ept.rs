// src/vmx/ept.rs

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
        // Bits 0-2: EPT memory type (Write-Back)
        // Bit 3: Walk length - 1 (for 4-level paging, this is 3)
        // Bits 12-51: Physical address of PML4
        Self {
            eptp: (pml4_pa & 0xFFFFFFFFFF000) | (3 << 3) | 6,
        }
    }
}
