import json, sys, matplotlib.pyplot as plt, numpy as np

def main():
    with open(sys.argv[1]) as f:
        data = json.load(f)

    # Figure 1: Boxplot of all operations
    all_ops = [
        ("KEM Encaps", data["kem_encaps_us"]),
        ("KEM Decaps", data["kem_decaps_us"]),
        ("Sign (Falcon-512)", data["sig_sign_us"]),
        ("Verify (Falcon-512)", data["sig_verify_us"]),
        ("ChaCha20 Encrypt 16B", data["chacha_encrypt_us"]),
        ("ChaCha20 Decrypt 16B", data["chacha_decrypt_us"]),
        ("Nano-AI Inference", data["nano_ai_us"]),
        ("SHA-512 64KB", data["attestation_hash_us"]),
    ]
    labels = [x[0] for x in all_ops]
    values = [x[1] for x in all_ops]

    fig, ax = plt.subplots(figsize=(14, 6))
    box = ax.boxplot(values, patch_artist=True, showfliers=False)
    for patch in box['boxes']:
        patch.set_facecolor('#87CEEB')
    ax.set_xticklabels(labels, rotation=45, ha='right')
    ax.set_ylabel("Latency (µs)")
    ax.set_title(f"Post-Quantum Cryptographic Operation Latencies (n={len(values[0])})")
    ax.grid(axis='y', linestyle='--', alpha=0.5)
    plt.tight_layout()
    plt.savefig("fig1_boxplot_latencies.pdf", dpi=300)
    print("Saved fig1_boxplot_latencies.pdf")

    # Figure 2: Enclave round-trip breakdown (pie chart)
    median_kem = np.median(data['kem_encaps_us']) + np.median(data['kem_decaps_us'])
    median_enc = np.median(data['chacha_encrypt_us'])
    median_ai  = np.median(data['nano_ai_us'])
    median_sig = np.median(data['sig_sign_us'])
    roundtrip  = data['enclave_roundtrip_us']
    overhead   = roundtrip - (median_kem + median_enc + median_ai + median_sig)
    if overhead < 0: overhead = 0

    sizes = [median_kem, median_enc, median_ai, median_sig, overhead]
    labels = ['KEM Handshake', 'Encrypt Tensor', 'AI Inference', 'Sign Result', 'Overhead']
    colors = ['#ff9999', '#66b3ff', '#99ff99', '#ffcc99', '#c2c2f0']
    fig, ax = plt.subplots()
    ax.pie(sizes, labels=labels, autopct='%1.1f%%', startangle=90, colors=colors)
    ax.set_title(f'Enclave Round-Trip Breakdown (Total: {roundtrip:.0f} µs)')
    plt.tight_layout()
    plt.savefig("fig2_roundtrip_pie.pdf", dpi=300)
    print("Saved fig2_roundtrip_pie.pdf")

    # Figure 3: CDF of KEM latencies
    fig, ax = plt.subplots()
    for label, vec in [('KEM Encaps', data['kem_encaps_us']), ('KEM Decaps', data['kem_decaps_us'])]:
        sorted_vec = np.sort(vec)
        cdf = np.arange(1, len(sorted_vec)+1) / len(sorted_vec)
        ax.plot(sorted_vec, cdf, label=label, linewidth=2)
    ax.set_xlabel('Latency (µs)')
    ax.set_ylabel('Cumulative Probability')
    ax.set_title('CDF of ML-KEM-768 Encapsulation/Decapsulation')
    ax.legend()
    ax.grid(linestyle='--', alpha=0.5)
    plt.tight_layout()
    plt.savefig("fig3_kem_cdf.pdf", dpi=300)
    print("Saved fig3_kem_cdf.pdf")
    
if __name__ == "__main__":
    main()
