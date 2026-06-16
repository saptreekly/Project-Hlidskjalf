//! Linux simulation integration tests for the full hypervisor bring-up path.

use hlidskjalf::vmx::sim::SimEvent;
use hlidskjalf::vmx::vmcs::encoding;
use hlidskjalf::{
    DriverEntry, DriverObject, STATUS_NOT_SUPPORTED, STATUS_SUCCESS, STATUS_UNSUCCESSFUL,
    UnicodeString, sim,
};

#[test]
fn driver_entry_completes_full_simulated_bringup() {
    sim::reset();

    let status = DriverEntry(core::ptr::null_mut(), core::ptr::null_mut());
    assert_eq!(status, STATUS_SUCCESS);

    let events = sim::events();
    assert!(events.iter().any(|e| matches!(e, SimEvent::Vmxon { .. })));
    assert!(events.iter().any(|e| matches!(e, SimEvent::Vmptrld { .. })));
    assert!(events.iter().any(|e| matches!(e, SimEvent::Vmlaunch)));
    assert_ne!(sim::vmcs_field(encoding::EPT_POINTER), 0);
    assert_eq!(sim::vmcs_field(encoding::HOST_RIP), sim::HOST_EXIT_STUB);
}

#[test]
fn driver_entry_rejects_missing_vmx_cpuid() {
    sim::reset();
    sim::set_cpuid_ecx(1, 0);

    let status = DriverEntry(core::ptr::null_mut(), core::ptr::null_mut());
    assert_eq!(status, STATUS_NOT_SUPPORTED);
}

#[test]
fn driver_entry_rejects_failed_vmxon() {
    sim::reset();
    sim::set_vmxon_fail(true);

    let status = DriverEntry(core::ptr::null_mut(), core::ptr::null_mut());
    assert_eq!(status, STATUS_NOT_SUPPORTED);
}

#[test]
fn driver_entry_rejects_failed_vmlaunch() {
    sim::reset();
    sim::set_vmlaunch_fail(true);

    let status = DriverEntry(core::ptr::null_mut(), core::ptr::null_mut());
    assert_eq!(status, STATUS_UNSUCCESSFUL);
}

#[test]
fn vmxon_region_gets_revision_id_in_sim() {
    sim::reset();
    let _ = DriverEntry(
        core::ptr::null_mut::<DriverObject>(),
        core::ptr::null_mut::<UnicodeString>(),
    );
    let revision = unsafe { (*hlidskjalf::vmx::memory::VMXON_REGION.get()).revision_id() };
    assert_eq!(revision, 0x1A);
}
