// src/vmx/exit.rs

use super::context::GuestContext;

/// The Rust VM Exit handler, called by the assembly wrapper.
#[no_mangle]
pub extern "system" fn vm_exit_handler_rust(context: &mut GuestContext) {
    // 1. Determine exit reason via VMREAD
    // 2. Dispatch to specific handler (e.g., handle_cpuid, handle_ept_violation)
    
    // For now, placeholder.
}
