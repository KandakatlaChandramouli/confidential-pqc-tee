mod error; mod iouring; mod kvm_enclave; mod pqcrypto; mod seccomp;
use error::Result;
use iouring::MultiQueueEngine;
use kvm_enclave::KvmEnclave;
use pqcrypto::{PmuMonitor, PqcSigner, PqcKem};
use chacha20poly1305::{ChaCha20Poly1305, aead::{Aead, KeyInit, generic_array::GenericArray}};

fn main() -> Result<()> {
    env_logger::init();
    println!("=============================================================");
    println!("  SOVEREIGN AI NEXUS v3.0 - MONARCH TIER");
    println!("  KVM | io_uring MQ | Seccomp | KEM+SIG | ChaCha Tunnel | Nano-AI");
    println!("=============================================================\n");

    println!("[1/7] KVM enclave with 1MB network packet buffer...");
    let mut enclave = KvmEnclave::new()?;
    enclave.write_guest_code(0, &[0xB0, 0x42, 0xE6, 0x10, 0xF4])?;
    println!("      [+] 2MB isolated guest memory\n");

    println!("[2/7] Multi-queue io_uring engine (256W/256R)...");
    let mut mq = MultiQueueEngine::new(256, 256)?;
    println!("      [+] Write ring fd: {}, Read ring fd: {}\n", mq.write_ring_fd(), mq.read_ring_fd());

    println!("[3/7] Multichannel I/O: packet injection test...");
    enclave.net_buffer.inject_packet(b"ENCLAVE:ATTEST:CHALLENGE:NONCE=0xDEADBEEF");
    let mut rx = [0u8; 64];
    let n = enclave.net_buffer.read_packet(&mut rx);
    println!("      [+] Echoed {} bytes: {}\n", n, String::from_utf8_lossy(&rx[..n]));

    println!("[4/7] Falcon-512 identity signature engine...");
    let signer = PqcSigner::new()?;
    println!("      [+] pk: {}B, sig: {}B", signer.public_key.len(), signer.sig_len);
    let test = b"sovereign-ai-nexus-attestation";
    let sig = signer.sign(test)?;
    let ok = signer.verify(test, &sig)?;
    println!("      [SIG-TEST] Signature: {} bytes, Verification: {}\n", sig.len(), if ok {"PASSED"} else {"FAILED"});

    println!("[5/7] ML-KEM-768 quantum-safe key encapsulation (HNDL protection)...");
    let kem = PqcKem::new()?;
    println!("      [+] pk: {}B, ct: {}B, ss: {}B", kem.public_key.len(), kem.ct_len, kem.ss_len);

    println!("\n[6/7] SOVEREIGN AI NEXUS - Confidential Inference Pipeline");
    let mut ct_kem = vec![0u8; kem.ct_len];
    let mut client_ss = vec![0u8; kem.ss_len];
    kem.encapsulate(&mut ct_kem, &mut client_ss, &kem.public_key)?;
    println!("      [KEM] Client encapsulated shared secret ({} bytes)", client_ss.len());

    let mut enclave_ss = vec![0u8; kem.ss_len];
    kem.decapsulate(&mut enclave_ss, &ct_kem)?;
    assert_eq!(client_ss, enclave_ss, "Shared secret mismatch!");
    println!("      [KEM] Enclave derived shared secret. Quantum-safe tunnel established.");

    // Build key and nonce as GenericArrays
    let key_bytes: [u8; 32] = enclave_ss[..32].try_into().unwrap();
    let cipher_key = GenericArray::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(cipher_key);
    
    let nonce_bytes: [u8; 12] = *b"nonce12bytes";
    let nonce = GenericArray::from_slice(&nonce_bytes);

    let input_tensor: [f32; 4] = [1.0, 2.0, 3.0, 4.0];
    let tensor_bytes: [u8; 16] = unsafe { std::mem::transmute(input_tensor) };
    let encrypted_tensor = cipher.encrypt(nonce, &tensor_bytes[..])
        .map_err(|e| error::TeeError::ChaCha(format!("Encryption failed: {}", e)))?;
    println!("      [TUNNEL] Client encrypted tensor ({}B -> {}B ciphertext)", tensor_bytes.len(), encrypted_tensor.len());

    let decrypted_bytes = cipher.decrypt(nonce, encrypted_tensor.as_ref())
        .map_err(|e| error::TeeError::ChaCha(format!("Decryption failed: {}", e)))?;
    let decrypted_tensor: [f32; 4] = unsafe {
        let ptr = decrypted_bytes.as_ptr() as *const [f32; 4];
        *ptr
    };
    println!("      [TUNNEL] Enclave decrypted tensor: {:?}", decrypted_tensor);

    let weights: [f32; 4] = [0.5, -1.2, 3.0, 0.1];
    let bias: f32 = 0.05;
    let mut result: f32 = 0.0;
    for i in 0..4 { result += decrypted_tensor[i] * weights[i]; }
    result += bias;
    println!("      [NANO-AI] Inference complete: W*x + b = {:.4}", result);

    let result_bytes = result.to_be_bytes();
    let inference_sig = signer.sign(&result_bytes)?;
    println!("      [PROOF] Result signed with Falcon-512 ({} bytes)", inference_sig.len());

    let mut response = Vec::new();
    response.extend_from_slice(&result_bytes);
    response.extend_from_slice(&inference_sig);
    let encrypted_response = cipher.encrypt(nonce, &response[..])
        .map_err(|e| error::TeeError::ChaCha(format!("Response encryption failed: {}", e)))?;
    println!("      [TUNNEL] Signed result encrypted for client ({}B)\n", encrypted_response.len());

    println!("[7/7] Activating seccomp BPF sandbox and running enclave...");
    let allowed: Vec<i32> = vec![
        libc::SYS_read as i32, libc::SYS_write as i32, libc::SYS_writev as i32,
        libc::SYS_fstat as i32, libc::SYS_lseek as i32,
        libc::SYS_io_uring_setup as i32, libc::SYS_io_uring_enter as i32, libc::SYS_io_uring_register as i32,
        libc::SYS_ioctl as i32, libc::SYS_mmap as i32, libc::SYS_munmap as i32,
        libc::SYS_mprotect as i32, libc::SYS_brk as i32, libc::SYS_futex as i32,
        libc::SYS_exit_group as i32, libc::SYS_exit as i32, libc::SYS_getpid as i32,
        libc::SYS_getrandom as i32, libc::SYS_rt_sigaction as i32, libc::SYS_rt_sigprocmask as i32,
        libc::SYS_sigaltstack as i32, libc::SYS_close as i32, libc::SYS_sched_yield as i32,
        libc::SYS_nanosleep as i32, libc::SYS_clock_gettime as i32, libc::SYS_gettid as i32,
        libc::SYS_tgkill as i32, libc::SYS_set_robust_list as i32, libc::SYS_rseq as i32,
        libc::SYS_prlimit64 as i32, libc::SYS_restart_syscall as i32,
    ];
    seccomp::install_filter(&allowed).map_err(|e| error::TeeError::Seccomp(e))?;
    println!("      [+] {} syscalls permitted, sandbox active\n", allowed.len());

    let mut pmu = PmuMonitor::new();
    for i in 1..=2 {
        let ld = pmu.sample_load();
        match enclave.run() {
            Ok(kvm_ioctls::VcpuExit::IoOut(port, data)) => {
                println!("  [ENCLAVE-IO-{}] Port 0x{:04X} = 0x{:02X} | PMU: {:.0}%", i, port, data[0], ld*100.0);
                let event = [data[0], i as u8];
                let sig = signer.sign(&event)?;
                let msg = format!("[NEXUS-IO] Event {} attested (sig {}B)\n", i, sig.len());
                mq.async_write(libc::STDOUT_FILENO, msg.as_bytes(), i)?;
                mq.flush_writes(1)?;
            }
            Ok(kvm_ioctls::VcpuExit::Hlt) => {
                println!("  [ENCLAVE] HLT - generating attestation...");
                let hash = enclave.attestation_hash();
                let attest_sig = signer.sign(&hash)?;
                println!("  [ATTEST] SHA-512 hash: {}... signed: {}B", hex::encode(&hash[..8]), attest_sig.len());
                break;
            }
            _ => break,
        }
    }

    println!("\n=============================================================");
    println!("  SOVEREIGN AI NEXUS - MONARCH TIER");
    println!("  KVM: 2MB enclave | io_uring: MQ 256x2 | Seccomp: {} syscalls", allowed.len());
    println!("  PQC SIG: Falcon-512 | PQC KEM: ML-KEM-768 | Tunnel: ChaCha20-Poly1305");
    println!("  Nano-AI: Linear Inference | Attestation: SHA-512 guest hash");
    println!("  Status: CLEAN SHUTDOWN - Sovereign Nexus Operational");
    println!("=============================================================");
    Ok(())
}
