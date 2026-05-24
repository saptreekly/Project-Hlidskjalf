// src/vmx/exit.rs

/// The VM Exit Handler entry point.
/// This function must save all registers, handle the exit, and then restore them.
#[unsafe(no_mangle)]
pub extern "system" fn vm_exit_handler() -> ! {
    // 1. Save guest state (all registers)
    // 2. Determine exit reason via VMREAD
    // 3. Dispatch to specific handler (e.g., handle_cpuid, handle_ept_violation)
    // 4. Restore guest state
    // 5. VMLAUNCH/VMRESUME
    
    // For now, infinite loop as a placeholder.
    loop {}
}
