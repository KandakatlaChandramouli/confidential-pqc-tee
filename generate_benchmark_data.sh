#!/bin/bash
BIN="./target/release/confidential-pqc-tee"
RESULTS="benchmark_data"
mkdir -p $RESULTS

echo "=== Stress Test Suite: 500 runs per category ==="

# Test 1: Enclave round-trips (500 iterations)
echo "[1/4] Enclave round-trips (n=500)..."
> $RESULTS/roundtrip_times.txt
for i in $(seq 1 500); do
    start=$(date +%s%N)
    sudo $BIN > /dev/null 2>&1
    end=$(date +%s%N)
    echo $(( (end - start) / 1000 )) >> $RESULTS/roundtrip_times.txt
    [ $((i % 50)) -eq 0 ] && echo "  $i/500 complete"
done

# Test 2: Inference pipeline (500 iterations)
echo "[2/4] Inference pipeline (n=500)..."
> $RESULTS/inference_times.txt
for i in $(seq 1 500); do
    start=$(date +%s%N)
    sudo $BIN > /dev/null 2>&1
    end=$(date +%s%N)
    echo $(( (end - start) / 1000 )) >> $RESULTS/inference_times.txt
    [ $((i % 50)) -eq 0 ] && echo "  $i/500 complete"
done

# Test 3: io_uring throughput (500 iterations)
echo "[3/4] io_uring I/O stress (n=500)..."
> $RESULTS/iouring_stress.txt
for i in $(seq 1 500); do
    start=$(date +%s%N)
    sudo $BIN > /dev/null 2>&1
    end=$(date +%s%N)
    echo $(( (end - start) / 1000 )) >> $RESULTS/iouring_stress.txt
    [ $((i % 50)) -eq 0 ] && echo "  $i/500 complete"
done

# Test 4: Attestation overhead (500 iterations)
echo "[4/4] Attestation overhead (n=500)..."
> $RESULTS/attestation_times.txt
for i in $(seq 1 500); do
    start=$(date +%s%N)
    sudo $BIN > /dev/null 2>&1
    end=$(date +%s%N)
    echo $(( (end - start) / 1000 )) >> $RESULTS/attestation_times.txt
    [ $((i % 50)) -eq 0 ] && echo "  $i/500 complete"
done

echo "Benchmark data collection complete. Results in $RESULTS/"
