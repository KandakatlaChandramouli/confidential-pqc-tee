use std::vec::Vec;

const BPF_LD_W_ABS: u16 = 0x20;
const BPF_JMP_JEQ_K: u16 = 0x15;
const BPF_RET_K: u16 = 0x06;
const SECCOMP_RET_KILL: u32 = 0x8000_0000;
const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;

fn simulate_bpf(allowed: &[i32], syscall_num: i32) -> u32 {
    let n = allowed.len();
    // Build the same filter as in src/seccomp.rs
    let mut insns: Vec<(u16, u8, u8, u32)> = vec![(BPF_LD_W_ABS, 0, 0, 0)]; // load syscall number from offset 0
    for i in 0..n {
        insns.push((BPF_JMP_JEQ_K, (n - i) as u8, 0, allowed[i] as u32));
    }
    insns.push((BPF_RET_K, 0, 0, SECCOMP_RET_KILL));
    insns.push((BPF_RET_K, 0, 0, SECCOMP_RET_ALLOW));

    // accumulator holds the syscall number
    let mut a = syscall_num as u32;
    let mut pc = 0;
    loop {
        if pc >= insns.len() {
            return SECCOMP_RET_KILL;
        }
        let (code, jt, jf, k) = insns[pc];
        match code {
            BPF_LD_W_ABS => {
                // offset 0 -> load syscall number into accumulator
                if k == 0 {
                    // a already contains syscall_num, no change needed
                }
                pc += 1;
            }
            BPF_JMP_JEQ_K => {
                if a == k {
                    pc += jt as usize + 1;
                } else {
                    pc += jf as usize + 1;
                }
            }
            BPF_RET_K => {
                return k;
            }
            _ => panic!("unknown BPF instruction"),
        }
    }
}

#[test]
fn formal_seccomp_correctness() {
    let allowed: Vec<i32> = vec![
        0,  // read
        1,  // write
        20, // writev
        5,  // fstat
        8,  // lseek
        425, // io_uring_setup
        426, // io_uring_enter
        427, // io_uring_register
        16, // ioctl
        9,  // mmap
        11, // munmap
        10, // mprotect
        12, // brk
        202, // futex
        231, // exit_group
        60, // exit
        39, // getpid
        318, // getrandom
        13, // rt_sigaction
        14, // rt_sigprocmask
        131, // sigaltstack
        3,  // close
        24, // sched_yield
        35, // nanosleep
        228, // clock_gettime
        186, // gettid
        234, // tgkill
        273, // set_robust_list
        334, // rseq
        261, // prlimit64
        219, // restart_syscall
    ];

    for syscall_num in 0..=500 {
        let result = simulate_bpf(&allowed, syscall_num);
        let expected = if allowed.contains(&syscall_num) {
            SECCOMP_RET_ALLOW
        } else {
            SECCOMP_RET_KILL
        };
        assert_eq!(result, expected, "Syscall {} failed", syscall_num);
    }
}
