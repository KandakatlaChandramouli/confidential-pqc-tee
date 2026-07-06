use crate::error::{Result, TeeError};
use kvm_ioctls::{Kvm, VcpuFd};
use sha2::{Sha512, Digest};
use std::os::unix::io::AsRawFd;
use std::ptr::NonNull;

pub const ENCLAVE_MEM_SIZE: usize = 0x200000;
const KVM_SET_USER_MEMORY_REGION: u64 = 0x4020_ae46;

#[repr(C)]
struct KvmUserspaceMemoryRegion { slot: u32, flags: u32, guest_phys_addr: u64, memory_size: u64, userspace_addr: u64 }

pub struct NetworkBuffer { pub base: *mut u8, pub size: usize, ro: usize, wo: usize }

impl NetworkBuffer {
    pub fn new(base: *mut u8, size: usize) -> Self { NetworkBuffer { base, size, ro:0, wo:0 } }
    pub fn inject_packet(&mut self, data: &[u8]) {
        let len = data.len().min(self.size - self.wo);
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), self.base.add(self.wo), len); }
        self.wo += len;
    }
    pub fn read_packet(&mut self, buf: &mut [u8]) -> usize {
        let avail = self.wo.saturating_sub(self.ro);
        let len = avail.min(buf.len());
        unsafe { std::ptr::copy_nonoverlapping(self.base.add(self.ro), buf.as_mut_ptr(), len); }
        self.ro += len;
        if self.ro >= self.wo { self.ro = 0; self.wo = 0; }
        len
    }
}

pub struct KvmEnclave { pub vcpu: VcpuFd, pub guest_mem: NonNull<u8>, _gp: *mut libc::c_void, pub net_buffer: NetworkBuffer }

impl KvmEnclave {
    pub fn new() -> Result<Self> {
        let kvm = Kvm::new().map_err(|e| TeeError::Kvm(e))?;
        let vm = kvm.create_vm().map_err(|e| TeeError::Kvm(e))?;
        let gp = unsafe { libc::mmap(std::ptr::null_mut(), ENCLAVE_MEM_SIZE, libc::PROT_READ|libc::PROT_WRITE|libc::PROT_EXEC, libc::MAP_ANONYMOUS|libc::MAP_PRIVATE, -1, 0) };
        if gp == libc::MAP_FAILED { return Err(TeeError::Io(std::io::Error::last_os_error())); }
        let gm = NonNull::new(gp as *mut u8).ok_or_else(|| TeeError::Kvm(kvm_ioctls::Error::new(libc::EINVAL)))?;
        let mr = KvmUserspaceMemoryRegion { slot:0, flags:0, guest_phys_addr:0x1000, memory_size:ENCLAVE_MEM_SIZE as u64, userspace_addr:gp as u64 };
        if unsafe { libc::ioctl(vm.as_raw_fd(), KVM_SET_USER_MEMORY_REGION as _, &mr) } != 0 {
            return Err(TeeError::Kvm(kvm_ioctls::Error::new(std::io::Error::last_os_error().raw_os_error().unwrap_or(libc::EINVAL))));
        }
        let vcpu = vm.create_vcpu(0).map_err(|e| TeeError::Kvm(e))?;
        let mut sregs = vcpu.get_sregs().map_err(|e| TeeError::Kvm(e))?;
        sregs.cs.base=0; sregs.cs.selector=0; sregs.cs.limit=0xFFFF; sregs.cs.db=0; sregs.cs.l=0;
        vcpu.set_sregs(&sregs).map_err(|e| TeeError::Kvm(e))?;
        let mut regs = vcpu.get_regs().map_err(|e| TeeError::Kvm(e))?;
        regs.rip=0x1000; regs.rflags=0x2;
        vcpu.set_regs(&regs).map_err(|e| TeeError::Kvm(e))?;
        let nb = NetworkBuffer::new(unsafe { (gp as *mut u8).add(0x100000) }, 0x100000);
        Ok(KvmEnclave { vcpu, guest_mem: gm, _gp: gp, net_buffer: nb })
    }
    pub fn write_guest_code(&self, offset: usize, code: &[u8]) -> Result<()> {
        if offset+code.len() > ENCLAVE_MEM_SIZE { return Err(TeeError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, "bounds"))); }
        unsafe { std::ptr::copy_nonoverlapping(code.as_ptr(), self.guest_mem.as_ptr().add(offset), code.len()); }
        Ok(())
    }
    pub fn attestation_hash(&self) -> Vec<u8> {
        let mut h = Sha512::new();
        h.update(unsafe { std::slice::from_raw_parts(self.guest_mem.as_ptr(), ENCLAVE_MEM_SIZE.min(65536)) });
        h.finalize().to_vec()
    }
    pub fn run(&self) -> std::result::Result<kvm_ioctls::VcpuExit, kvm_ioctls::Error> { self.vcpu.run() }
}

impl Drop for KvmEnclave { fn drop(&mut self) { unsafe { libc::munmap(self._gp, ENCLAVE_MEM_SIZE); } } }