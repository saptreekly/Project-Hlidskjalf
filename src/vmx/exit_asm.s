// src/vmx/exit_asm.s

.global _vm_exit_wrapper

# This function is called by the CPU on VM Exit.
# The CPU has already switched to the HOST RSP/RIP defined in the VMCS.
_vm_exit_wrapper:
    # 1. Save all registers on the stack (GuestContext structure)
    push rax
    push rcx
    push rdx
    push rbx
    # RSP will be handled separately if needed
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    # 2. Prepare for calling Rust handler
    # Pass pointer to saved registers as the first argument (RDI)
    mov rdi, rsp
    
    # Call the Rust VM Exit handler
    call vm_exit_handler_rust

    # 3. Restore registers
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rbx
    pop rdx
    pop rcx
    pop rax

    # 4. Resume Guest
    vmresume
    # If VMRESUME fails, we'd handle that here
    1: jmp 1b # Should not be reached
