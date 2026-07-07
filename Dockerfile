# Requires host kernel >= 6.10 for Januscape (CVE-2026-53359) mitigation
# Requires host kernel >= 6.10 for Januscape (CVE-2026-53359) mitigation
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y build-essential cmake libssl-dev libelf-dev libbpf-dev cargo curl sudo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /app
COPY . .
RUN cargo build --release
CMD ["sudo", "./target/release/confidential-pqc-tee"]
