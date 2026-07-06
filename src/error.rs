use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum TeeError {
    #[error("KVM: {0}")] Kvm(#[from] kvm_ioctls::Error),
    #[error("IO: {0}")] Io(#[from] io::Error),
    #[error("Seccomp: {0}")] Seccomp(String),
    #[error("Mmap: {0}")] Mmap(#[from] nix::Error),
    #[error("PQC: {0}")] Pqc(String),
    #[error("io_uring: {0}")] IoUring(String),
    #[error("KEM: {0}")] Kem(String),
    #[error("ChaCha: {0}")] ChaCha(String),
}
pub type Result<T> = std::result::Result<T, TeeError>;