// src/vmx/exit.rs

use super::context::GuestContext;
use super::vmcs::{encoding, vmread};

const VM_EXIT_REASON_CPUID: u64 = 10;
const VM_EXIT_REASON_EPT_VIOLATION: u64 = 48;

fn handle_cpuid(_context: &mut GuestContext) {}

fn handle_ept_violation() {
    let _ = vmread(encoding::VM_EXIT_QUALIFICATION);
}

/// Rust VM-exit handler called from the assembly entry point.
#[unsafe(no_mangle)]
pub extern "system" fn vm_exit_handler_rust(context: &mut GuestContext) {
    if let Ok(reason) = vmread(encoding::VM_EXIT_REASON) {
        let basic_reason = reason & 0xFFFF;
        match basic_reason {
            VM_EXIT_REASON_CPUID => handle_cpuid(context),
            VM_EXIT_REASON_EPT_VIOLATION => handle_ept_violation(),
            _ => {}
        }
    }

    if let Ok(guest_rsp) = vmread(encoding::GUEST_RSP) {
        let _ = guest_rsp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_reason_constants_are_stable() {
        assert_eq!(VM_EXIT_REASON_CPUID, 10);
        assert_eq!(VM_EXIT_REASON_EPT_VIOLATION, 48);
    }
}
