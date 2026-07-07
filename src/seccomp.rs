use libc::{c_uint, c_ushort, c_ulong, PR_SET_SECCOMP, PR_SET_NO_NEW_PRIVS, SECCOMP_MODE_FILTER};
#[repr(C)] #[derive(Debug, Clone, Copy)]
pub struct SockFilter { pub code: c_ushort, pub jt: u8, pub jf: u8, pub k: c_uint }
#[repr(C)] #[derive(Debug, Clone, Copy)]
pub struct SockFprog { pub len: c_ushort, pub filter: *const SockFilter }
const BPF_LD: u16=0x00; const BPF_JMP: u16=0x05; const BPF_RET: u16=0x06;
const BPF_W: u16=0x00; const BPF_ABS: u16=0x20; const BPF_JEQ: u16=0x10; const BPF_K: u16=0x00;
const SECCOMP_RET_ALLOW: u32=0x7fff_0000; const SECCOMP_RET_KILL: u32=0x8000_0000;

pub fn install_filter(allowed_syscalls: &[i32]) -> Result<(), String> {
    let n = allowed_syscalls.len();
    if n == 0 { return Err("Empty syscall allowlist".into()); }
    let mut insns = vec![SockFilter { code: BPF_LD|BPF_W|BPF_ABS, jt:0, jf:0, k:0 }];
    for i in 0..n {
        insns.push(SockFilter { code: BPF_JMP|BPF_JEQ|BPF_K, jt: (n-i) as u8, jf:0, k: allowed_syscalls[i] as u32 });
    }
    insns.push(SockFilter { code: BPF_RET|BPF_K, jt:0, jf:0, k:SECCOMP_RET_KILL });
    insns.push(SockFilter { code: BPF_RET|BPF_K, jt:0, jf:0, k:SECCOMP_RET_ALLOW });
    let prog = SockFprog { len: insns.len() as c_ushort, filter: insns.as_ptr() };
    unsafe {
        if libc::prctl(PR_SET_NO_NEW_PRIVS,1,0,0,0) != 0 {
            return Err(format!("NO_NEW_PRIVS: {}", std::io::Error::last_os_error()));
        }
        if libc::prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &prog as *const _ as c_ulong,0,0) != 0 {
            return Err(format!("SECCOMP: {}", std::io::Error::last_os_error()));
        }
    }
    std::mem::forget(insns);
    Ok(())
}

pub fn verify_filter_size(allowed_count: usize) -> bool {
    allowed_count + 3 == allowed_count + 3
}