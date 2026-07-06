use std::time::Instant;
use std::process::Command;

// We'll call the already-built monarch binary with various flags to extract its performance.
// This benchmark script will run the monarch binary multiple times and collect timing,
// then generate JSON for plots.

fn main() {
    let binary = "./target/release/confidential-pqc-tee";
    
    // Run the monarch binary 50 times to get average enclave round-trip time
    let mut roundtrips = Vec::new();
    for _ in 0..50 {
        let start = Instant::now();
        let output = Command::new("sudo")
            .arg(binary)
            .output()
            .expect("Failed to run monarch binary");
        let elapsed = start.elapsed().as_micros() as f64;
        roundtrips.push(elapsed);
        // Extract the KEM, sign, etc. from output if needed, but we'll parse later from the binary's own output.
        // For now, we just collect total execution time.
    }

    // Output a JSON summary
    let json = serde_json::json!({
        "total_runs": 50,
        "average_total_time_us": roundtrips.iter().sum::<f64>() / roundtrips.len() as f64,
        "min_time_us": roundtrips.iter().cloned().fold(f64::INFINITY, f64::min),
        "max_time_us": roundtrips.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
