use crate::error::{Result, TeeError};
use std::ffi::CString;

extern "C" {
    fn OQS_SIG_new(method_name: *const libc::c_char) -> *mut libc::c_void;
    fn OQS_SIG_free(sig: *mut libc::c_void);
    fn OQS_SIG_keypair(sig: *mut libc::c_void, pk: *mut u8, sk: *mut u8) -> libc::c_int;
    fn OQS_SIG_sign(sig: *mut libc::c_void, sig_out: *mut u8, sig_len: *mut libc::size_t, msg: *const u8, msg_len: libc::size_t, sk: *const u8) -> libc::c_int;
    fn OQS_SIG_verify(sig: *mut libc::c_void, msg: *const u8, msg_len: libc::size_t, sig_in: *const u8, sig_len: libc::size_t, pk: *const u8) -> libc::c_int;
    fn OQS_SIG_alg_is_enabled(alg_name: *const libc::c_char) -> libc::c_int;
    fn OQS_KEM_new(method_name: *const libc::c_char) -> *mut libc::c_void;
    fn OQS_KEM_free(kem: *mut libc::c_void);
    fn OQS_KEM_keypair(kem: *mut libc::c_void, pk: *mut u8, sk: *mut u8) -> libc::c_int;
    fn OQS_KEM_encaps(kem: *mut libc::c_void, ct: *mut u8, ss: *mut u8, pk: *const u8) -> libc::c_int;
    fn OQS_KEM_decaps(kem: *mut libc::c_void, ss: *mut u8, ct: *const u8, sk: *const u8) -> libc::c_int;
    fn OQS_KEM_alg_is_enabled(alg_name: *const libc::c_char) -> libc::c_int;
}

pub struct PqcSigner {
    handle: *mut libc::c_void,
    pub public_key: Vec<u8>,
    secret_key: Vec<u8>,
    pub sig_len: usize,
}
impl PqcSigner {
    pub fn new() -> Result<Self> {
        let alg = CString::new("Falcon-512").unwrap();
        unsafe {
            if OQS_SIG_alg_is_enabled(alg.as_ptr()) == 0 { return Err(TeeError::Pqc("Falcon-512 not enabled".into())); }
            let h = OQS_SIG_new(alg.as_ptr());
            if h.is_null() { return Err(TeeError::Pqc("OQS_SIG_new failed".into())); }
            let mut pk = vec![0u8; 897]; let mut sk = vec![0u8; 1281];
            if OQS_SIG_keypair(h, pk.as_mut_ptr(), sk.as_mut_ptr()) != 0 { OQS_SIG_free(h); return Err(TeeError::Pqc("keygen failed".into())); }
            Ok(PqcSigner { handle: h, public_key: pk, secret_key: sk, sig_len: 690 })
        }
    }
    pub fn sign(&self, msg: &[u8]) -> Result<Vec<u8>> {
        unsafe {
            let mut sig = vec![0u8; self.sig_len]; let mut actual = self.sig_len;
            if OQS_SIG_sign(self.handle, sig.as_mut_ptr(), &mut actual, msg.as_ptr(), msg.len(), self.secret_key.as_ptr()) != 0 { return Err(TeeError::Pqc("sign failed".into())); }
            sig.truncate(actual); Ok(sig)
        }
    }
    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> Result<bool> {
        unsafe { Ok(OQS_SIG_verify(self.handle, msg.as_ptr(), msg.len(), sig.as_ptr(), sig.len(), self.public_key.as_ptr()) == 0) }
    }
    pub fn public_key_der(&self) -> Vec<u8> {
        let mut der = Vec::new();
        der.extend_from_slice(&[0x30, 0x81, (self.public_key.len() + 20) as u8]);
        der.extend_from_slice(&[0x30, 0x0d, 0x06, 0x09, 0x2b, 0x06, 0x01, 0x04, 0x01, 0x02, 0x82, 0x0b, 0x01, 0x00, 0x08]);
        der.extend_from_slice(&[0x03, (self.public_key.len() + 1) as u8, 0x00]);
        der.extend_from_slice(&self.public_key);
        der
    }
}
impl Drop for PqcSigner { fn drop(&mut self) { if !self.handle.is_null() { unsafe { OQS_SIG_free(self.handle); } } } }
unsafe impl Send for PqcSigner {}
unsafe impl Sync for PqcSigner {}

pub struct PqcKem {
    handle: *mut libc::c_void,
    pub public_key: Vec<u8>,
    secret_key: Vec<u8>,
    pub ct_len: usize,
    pub ss_len: usize,
}
impl PqcKem {
    pub fn new() -> Result<Self> {
        let alg = CString::new("ML-KEM-768").unwrap();
        unsafe {
            if OQS_KEM_alg_is_enabled(alg.as_ptr()) == 0 { return Err(TeeError::Kem("ML-KEM-768 not enabled".into())); }
            let h = OQS_KEM_new(alg.as_ptr());
            if h.is_null() { return Err(TeeError::Kem("OQS_KEM_new failed".into())); }
            let mut pk = vec![0u8; 1184]; let mut sk = vec![0u8; 2400];
            if OQS_KEM_keypair(h, pk.as_mut_ptr(), sk.as_mut_ptr()) != 0 { OQS_KEM_free(h); return Err(TeeError::Kem("keygen failed".into())); }
            Ok(PqcKem { handle: h, public_key: pk, secret_key: sk, ct_len: 1088, ss_len: 32 })
        }
    }
    pub fn encapsulate(&self, ct: &mut [u8], ss: &mut [u8], pk: &[u8]) -> Result<()> {
        unsafe { if OQS_KEM_encaps(self.handle, ct.as_mut_ptr(), ss.as_mut_ptr(), pk.as_ptr()) != 0 { return Err(TeeError::Kem("encaps failed".into())); } Ok(()) }
    }
    pub fn decapsulate(&self, ss: &mut [u8], ct: &[u8]) -> Result<()> {
        unsafe { if OQS_KEM_decaps(self.handle, ss.as_mut_ptr(), ct.as_ptr(), self.secret_key.as_ptr()) != 0 { return Err(TeeError::Kem("decaps failed".into())); } Ok(()) }
    }
}
impl Drop for PqcKem { fn drop(&mut self) { if !self.handle.is_null() { unsafe { OQS_KEM_free(self.handle); } } } }

pub struct PqcCertificateAuthority {
    pub root_signer: PqcSigner,
    pub root_cert_der: Vec<u8>,
}
impl PqcCertificateAuthority {
    pub fn new() -> Result<Self> {
        let root_signer = PqcSigner::new()?;
        let root_cert_der = root_signer.public_key_der();
        Ok(PqcCertificateAuthority { root_signer, root_cert_der })
    }
    pub fn issue_enclave_certificate(&self, enclave_pk: &[u8]) -> Result<Vec<u8>> {
        let mut cert = Vec::new();
        cert.extend_from_slice(&self.root_cert_der);
        cert.extend_from_slice(enclave_pk);
        let sig = self.root_signer.sign(&cert)?;
        cert.extend_from_slice(&sig);
        Ok(cert)
    }
    pub fn verify_certificate(&self, cert: &[u8]) -> Result<bool> {
        let sig_len = self.root_signer.sig_len;
        if cert.len() < self.root_cert_der.len() + sig_len { return Ok(false); }
        let tbs = &cert[..cert.len() - sig_len];
        let sig = &cert[cert.len() - sig_len..];
        self.root_signer.verify(tbs, sig)
    }
}

pub struct PmuMonitor { last_tsc: u64, idle: u64, busy: u64 }
impl PmuMonitor {
    pub fn new() -> Self { PmuMonitor { last_tsc: unsafe { std::arch::x86_64::_rdtsc() }, idle:0, busy:0 } }
    pub fn sample_load(&mut self) -> f64 {
        let cur = unsafe { std::arch::x86_64::_rdtsc() };
        let delta = cur.wrapping_sub(self.last_tsc);
        self.last_tsc = cur;
        if delta > 10_000_000 { self.busy += delta; } else { self.idle += delta; }
        let total = self.idle + self.busy;
        if total == 0 { 0.0 } else { self.busy as f64 / total as f64 }
    }
}