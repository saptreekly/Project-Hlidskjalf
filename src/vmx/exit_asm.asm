; src/vmx/exit_asm.asm — MASM/ML64 syntax for Windows kernel builds

EXTERN vm_exit_handler_rust:PROC

.code
PUBLIC vm_exit_wrapper

vm_exit_wrapper PROC
    push rax
    push rcx
    push rdx
    push rbx
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

    mov rcx, rsp
    sub rsp, 32
    call vm_exit_handler_rust
    add rsp, 32

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

    vmresume
    ud2
vm_exit_wrapper ENDP

END
