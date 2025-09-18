[package]
name = "mev-shield"
version = "1.0.0"
edition = "2021"
authors = ["MEV Shield Team"]
description = "Comprehensive MEV Protection Framework for Blockchain Networks"
license = "MIT"

[workspace]
members = [
    "core",
    "api",
    "cli",
    "detection",
    "encryption",
    "ordering",
    "redistribution"
]

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"] }
redis = { version = "0.23", features = ["tokio-comp"] }
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.4", features = ["cors", "trace"] }
clap = { version = "4.0", features = ["derive"] }
config = "0.13"
async-trait = "0.1"

# Cryptography
ring = "0.16"
sha3 = "0.10"
num-bigint = "0.4"
threshold_crypto = "0.4"
bls = "0.4"

# Blockchain integration
ethers = "2.0"
web3 = "0.19"

# Performance and monitoring
metrics = "0.21"
metrics-exporter-prometheus = "0.12"

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]