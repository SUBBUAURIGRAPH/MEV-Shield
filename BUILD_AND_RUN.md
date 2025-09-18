# MEV Shield - Build and Run Instructions

## 🚀 Quick Start Guide

This document provides step-by-step instructions to build and run MEV Shield on your system.

## Prerequisites

### 1. Install Rust (REQUIRED)

MEV Shield is built with Rust. You must install Rust to compile the project.

```bash
# macOS/Linux - Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the on-screen instructions, then reload your shell configuration:
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

**Expected output:**
```
rustc 1.75.0 or higher
cargo 1.75.0 or higher
```

### 2. Install PostgreSQL (Optional for full functionality)

```bash
# macOS with Homebrew
brew install postgresql@14
brew services start postgresql@14

# Ubuntu/Debian
sudo apt-get update
sudo apt-get install postgresql-14 postgresql-client-14
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create MEV Shield database
createdb mevshield
```

### 3. Install Redis (Optional for caching)

```bash
# macOS with Homebrew
brew install redis
brew services start redis

# Ubuntu/Debian
sudo apt-get install redis-server
sudo systemctl start redis
sudo systemctl enable redis

# Test Redis connection
redis-cli ping
# Should return: PONG
```

## 📦 Building the Project

### Step 1: Navigate to Project Directory

```bash
cd "/Users/subbujois/subbuworkingdir/MEV Shield/MEV-Shield"
```

### Step 2: Build the Project

```bash
# Development build (faster compilation, with debug symbols)
cargo build

# Production build (optimized, takes longer)
cargo build --release
```

**First build may take 5-10 minutes** as it downloads and compiles all dependencies.

### Step 3: Run Tests (Optional)

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_encryption
```

## 🏃 Running MEV Shield

### Option 1: Run with Cargo (Development)

```bash
# Run with default configuration
cargo run

# Run with debug logging
cargo run -- --debug

# Run on custom port
cargo run -- --port 9000

# Run with custom config file
cargo run -- --config custom-config.toml
```

### Option 2: Run the Compiled Binary (Production)

```bash
# After building with --release
./target/release/mev-shield

# With options
./target/release/mev-shield --port 9000 --debug
```

### Option 3: Run the CLI Tool

```bash
# Build and run CLI
cargo run --bin mev-shield-cli -- --help

# Submit a transaction
cargo run --bin mev-shield-cli -- submit \
  --transaction '{"from":"0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb8","to":"0x5aAeb6053f3E94C9b9A09f33669435E7Ef1BeAed","value":"1000000000000000000","gas":21000,"gasPrice":"20000000000","nonce":0,"data":"0x"}' \
  --protection maximum
```

## 🐳 Docker Deployment (Alternative)

### Build and Run with Docker

```bash
# Build Docker image
docker build -t mev-shield:latest .

# Run with Docker Compose (includes PostgreSQL and Redis)
docker-compose up -d

# Check logs
docker-compose logs -f mev-shield

# Stop services
docker-compose down
```

## ⚙️ Configuration

### Default Configuration Location

The default configuration file is `config.toml` in the project root.

### Key Configuration Options

```toml
[api]
port = 8080                    # API server port
bind_address = "0.0.0.0"       # Listen address

[database]
postgres_url = "postgresql://mevshield:password@localhost/mevshield"

[cache]
redis_url = "redis://localhost:6379"

[blockchain]
ethereum_rpc = "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
```

### Environment Variables (Alternative)

```bash
# Set environment variables
export MEV_SHIELD_PORT=8080
export MEV_SHIELD_DB_URL="postgresql://localhost/mevshield"
export MEV_SHIELD_REDIS_URL="redis://localhost:6379"

# Run with environment variables
cargo run
```

## 🔍 Verifying the Installation

### 1. Check API Health

Once MEV Shield is running, verify the API is accessible:

```bash
# Check health endpoint
curl http://localhost:8080/api/v1/health

# Expected response:
# {"status":"healthy","version":"1.0.0"}
```

### 2. Check Metrics

```bash
# Get current metrics
curl http://localhost:8080/api/v1/metrics
```

### 3. Submit a Test Transaction

```bash
curl -X POST http://localhost:8080/api/v1/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "transaction": {
      "from": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb8",
      "to": "0x5aAeb6053f3E94C9b9A09f33669435E7Ef1BeAed",
      "value": "1000000000000000000",
      "gas": 21000,
      "gasPrice": "20000000000",
      "nonce": 0,
      "data": "0x"
    },
    "protection": {
      "level": "Standard",
      "private_pool": false
    },
    "chain_id": 1
  }'
```

## 🛠️ Troubleshooting

### Common Issues and Solutions

#### 1. Rust Not Found

**Error:** `cargo: command not found`

**Solution:**
```bash
# Reinstall Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 2. Build Fails with Dependency Errors

**Error:** `error: failed to compile mev-shield v1.0.0`

**Solution:**
```bash
# Clean and rebuild
cargo clean
cargo update
cargo build
```

#### 3. Database Connection Failed

**Error:** `Database connection failed: postgresql://localhost/mevshield`

**Solution:**
```bash
# Ensure PostgreSQL is running
brew services restart postgresql@14  # macOS
sudo systemctl restart postgresql    # Linux

# Create database if not exists
createdb mevshield

# Check connection
psql -d mevshield -c "SELECT 1;"
```

#### 4. Port Already in Use

**Error:** `Address already in use: 0.0.0.0:8080`

**Solution:**
```bash
# Find process using port
lsof -i :8080

# Kill the process or use different port
cargo run -- --port 9090
```

#### 5. Out of Memory During Build

**Solution:**
```bash
# Limit parallel jobs
cargo build -j 2
```

## 📊 Performance Tuning

### Optimize Compilation

```bash
# Use LLD linker (faster linking)
cargo install -f cargo-binutils
cargo build --release

# Enable incremental compilation
export CARGO_INCREMENTAL=1
cargo build
```

### Production Optimizations

Add to `Cargo.toml`:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

## 🔐 Security Considerations

1. **Never commit secrets** - Keep API keys and passwords in environment variables
2. **Use TLS in production** - Configure proper certificates
3. **Enable authentication** - Set `enable_auth = true` in config.toml
4. **Regular updates** - Keep dependencies updated with `cargo update`

## 📚 Additional Resources

- **API Documentation:** Run the server and visit http://localhost:8080/docs
- **Configuration Guide:** See `config.toml` for all available options
- **Development Guide:** See CONTRIBUTING.md for development workflow
- **Architecture:** See README.md for system architecture

## 🆘 Getting Help

If you encounter issues not covered here:

1. Check the logs: `RUST_LOG=debug cargo run`
2. Review error messages carefully
3. Check GitHub Issues: https://github.com/aurigraph/mev-shield/issues
4. Contact support: dev@aurigraph.io

---

## Quick Command Reference

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build --release

# Run
cargo run --release

# Test
cargo test

# Docker
docker-compose up -d

# Health check
curl http://localhost:8080/api/v1/health
```

**Last Updated:** January 2025
**Version:** 1.0.0