// src/vmx/context.rs

/// Guest GPR save area pushed by `exit_asm.s`.
///
/// Layout must match the push order in `exit_asm.s` exactly.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GuestContext {
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

pub const GUEST_CONTEXT_GPR_COUNT: usize = 15;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_matches_assembly_push_count() {
        assert_eq!(
            core::mem::size_of::<GuestContext>(),
            GUEST_CONTEXT_GPR_COUNT * 8
        );
    }
}
