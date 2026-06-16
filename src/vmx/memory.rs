// src/vmx/memory.rs
use core::cell::UnsafeCell;

pub const EPT_GB_COUNT: usize = 4;
pub const HOST_STACK_SIZE: usize = 8192;

#[repr(align(16))]
pub struct HostStack {
    bytes: [u8; HOST_STACK_SIZE],
}

impl Default for HostStack {
    fn default() -> Self {
        Self::new()
    }
}

impl HostStack {
    pub const fn new() -> Self {
        Self {
            bytes: [0; HOST_STACK_SIZE],
        }
    }

    pub fn top(&self) -> *mut u8 {
        self.bytes.as_ptr().wrapping_add(HOST_STACK_SIZE) as *mut u8
    }
}

#[repr(align(4096))]
#[derive(Clone, Copy)]
pub struct VmxRegion {
    bytes: [u8; 4096],
}

impl Default for VmxRegion {
    fn default() -> Self {
        Self::new()
    }
}

impl VmxRegion {
    pub const fn new() -> Self {
        Self { bytes: [0; 4096] }
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.bytes.as_mut_ptr()
    }

    pub fn revision_id(&self) -> u32 {
        u32::from_le_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]])
    }

    pub fn set_revision_id(&mut self, revision: u32) {
        let bytes = revision.to_le_bytes();
        self.bytes[0..4].copy_from_slice(&bytes);
    }
}

#[repr(C, align(4096))]
pub struct EptPdPages {
    pub pages: [VmxRegion; EPT_GB_COUNT],
}

impl Default for EptPdPages {
    fn default() -> Self {
        Self::new()
    }
}

impl EptPdPages {
    pub const fn new() -> Self {
        Self {
            pages: [VmxRegion::new(); EPT_GB_COUNT],
        }
    }
}

pub struct SyncWrapper<T>(UnsafeCell<T>);

unsafe impl<T> Sync for SyncWrapper<T> {}

impl<T> SyncWrapper<T> {
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    pub fn get(&self) -> *mut T {
        self.0.get()
    }
}

pub static VMXON_REGION: SyncWrapper<VmxRegion> = SyncWrapper::new(VmxRegion::new());
pub static VMCS_REGION: SyncWrapper<VmxRegion> = SyncWrapper::new(VmxRegion::new());
pub static EPT_PML4: SyncWrapper<VmxRegion> = SyncWrapper::new(VmxRegion::new());
pub static EPT_PDPT: SyncWrapper<VmxRegion> = SyncWrapper::new(VmxRegion::new());
pub static EPT_PD_PAGES: SyncWrapper<EptPdPages> = SyncWrapper::new(EptPdPages::new());
pub static HOST_STACK: SyncWrapper<HostStack> = SyncWrapper::new(HostStack::new());

#[cfg(not(any(test, feature = "fuzzing")))]
unsafe extern "system" {
    fn MmGetPhysicalAddress(BaseAddress: *mut core::ffi::c_void) -> u64;
}

/// Resolve a kernel virtual address to its physical address.
///
/// # Safety
///
/// `ptr` must reference valid mapped memory.
pub unsafe fn physical_address<T>(ptr: *mut T) -> u64 {
    #[cfg(any(test, feature = "fuzzing"))]
    {
        ptr as u64
    }
    #[cfg(not(any(test, feature = "fuzzing")))]
    {
        unsafe { MmGetPhysicalAddress(ptr.cast()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vmx_region_is_4k_aligned() {
        assert_eq!(core::mem::align_of::<VmxRegion>(), 4096);
    }

    #[test]
    fn hypervisor_static_allocation_is_lightweight() {
        let bytes = core::mem::size_of::<VmxRegion>() * 4
            + core::mem::size_of::<EptPdPages>()
            + core::mem::size_of::<HostStack>();
        assert!(bytes <= 40 * 1024);
    }
}
