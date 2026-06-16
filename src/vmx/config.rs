// src/vmx/config.rs

use super::ept::{EptPointer, identity_map_ept};
use super::init::{vm_exit_handler_address, vmptrld};
use super::memory::{EPT_PML4, HOST_STACK, VMCS_REGION, physical_address};
use super::msr::{
    IA32_VMX_ENTRY_CTLS, IA32_VMX_EXIT_CTLS, IA32_VMX_PINBASED_CTLS, IA32_VMX_PROCBASED_CTLS,
    IA32_VMX_PROCBASED_CTLS2, VMX_PROCBASED_USE_SECONDARY, VMX_PROCBASED2_ENABLE_EPT,
    adjust_vmx_control,
};
use super::state::{capture_cpu_state, write_guest_state, write_host_state};
use super::vmcs::{encoding, vmwrite};

/// Initializes the VMCS for a minimal in-place launch of the current CPU context.
///
/// # Safety
///
/// Caller must have entered VMX root operation and static VMX/EPT regions must be valid.
pub unsafe fn setup_vmcs() -> Result<(), &'static str> {
    let vmcs_pa = unsafe { physical_address(VMCS_REGION.get()) };
    unsafe {
        if !vmptrld(vmcs_pa) {
            return Err("VMPTRLD failed");
        }
    }

    vmwrite(encoding::VMCS_LINK_POINTER, 0xFFFF_FFFF_FFFF_FFFF)
        .map_err(|_| "VMCS link pointer write failed")?;

    unsafe { identity_map_ept() };
    let eptp = EptPointer::new(unsafe { physical_address(EPT_PML4.get()) });
    vmwrite(encoding::EPT_POINTER, eptp.eptp).map_err(|_| "EPT pointer write failed")?;

    let pin_controls = adjust_vmx_control(0, IA32_VMX_PINBASED_CTLS);
    let primary_controls = adjust_vmx_control(VMX_PROCBASED_USE_SECONDARY, IA32_VMX_PROCBASED_CTLS);
    let secondary_controls =
        adjust_vmx_control(VMX_PROCBASED2_ENABLE_EPT, IA32_VMX_PROCBASED_CTLS2);
    let exit_controls = adjust_vmx_control(0, IA32_VMX_EXIT_CTLS);
    let entry_controls = adjust_vmx_control(0, IA32_VMX_ENTRY_CTLS);

    vmwrite(encoding::PIN_BASED_VM_EXEC_CONTROL, pin_controls)
        .map_err(|_| "pin controls write failed")?;
    vmwrite(encoding::CPU_BASED_VM_EXEC_CONTROL, primary_controls)
        .map_err(|_| "primary controls write failed")?;
    vmwrite(encoding::SECONDARY_VM_EXEC_CONTROL, secondary_controls)
        .map_err(|_| "secondary controls write failed")?;
    vmwrite(encoding::VM_EXIT_CONTROLS, exit_controls).map_err(|_| "exit controls write failed")?;
    vmwrite(encoding::VM_ENTRY_CONTROLS, entry_controls)
        .map_err(|_| "entry controls write failed")?;

    let cpu_state = unsafe { capture_cpu_state() };
    write_guest_state(&cpu_state).map_err(|_| "guest state write failed")?;

    let host_stack_top = unsafe { (*HOST_STACK.get()).top() as u64 };
    write_host_state(&cpu_state, vm_exit_handler_address(), host_stack_top)
        .map_err(|_| "host state write failed")?;

    Ok(())
}
