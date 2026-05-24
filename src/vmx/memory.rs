// src/vmx/memory.rs
use core::cell::UnsafeCell;

#[repr(align(4096))]
#[allow(dead_code)]
pub struct VmxRegion([u8; 4096]);

impl VmxRegion {
    pub const fn new() -> Self {
        Self([0; 4096])
    }
}

// Wrapper to manually implement Sync for static access
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
