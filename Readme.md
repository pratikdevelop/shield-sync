# 🛡️ Shield-Sync

**Automatic, encrypted file synchronization** - Built with Rust

Like Dropbox, but your files are encrypted before leaving your device.

## How is this different from file-crypt?

| Feature | file-crypt | shield-sync |
|---------|------------|-------------|
| **Purpose** | Encrypt/decrypt files manually | **Automatic** background sync |
| **Usage** | One command per file | Set once, forget it |
| **Storage** | Local only | **Cloud-synced** (S3, etc.) |
| **Multi-device** | No | **Yes** - laptop ↔️ desktop ↔️ phone |
| **File watching** | No | **Yes** - detects changes automatically |
| **Version history** | No | **Yes** - restore old versions |

Shield-Sync **uses** file-crypt's encryption engine under the hood!

## Quick Start
```bash
# Initialize a synced folder
shield-sync init ~/Documents

# Add cloud storage backend
shield-sync remote add my-s3 s3://my-bucket

# Start the sync daemon (runs in background)
shield-sync daemon start

# That's it! Now any changes to ~/Documents sync automatically
# Files are encrypted using file-crypt's engine before upload
```

## Architecture

Shield-Sync = file-crypt encryption + automatic sync + cloud storage

## Status: 🚧 Phase 1 Development

- [x] Project structure
- [ ] File system watcher
- [ ] Local database for tracking
- [ ] S3 backend integration
- [ ] Integrate file-crypt for encryption
- [ ] Multi-device sync protocol

## Support Development

Building this in public! [Sponsor on GitHub](https://github.com/sponsors/pratikdevelop)
EOF

git add README.md
git commit -m "docs: clarify shield-sync vs file-crypt differences"
Step 2: Initialize Rust Project with file-crypt Dependency
bashcargo init --name shield-sync

# Update Cargo.toml to USE your file-crypt as a library
cat > Cargo.toml << 'EOF'
[package]
name = "shield-sync"
version = "0.1.0"
edition = "2021"
authors = ["Pratik Raut <pratikraut770@gmail.com>"]
description = "Automatic encrypted file synchronization using file-crypt"
license = "MIT"
repository = "https://github.com/pratikdevelop/shield-sync"
keywords = ["sync", "encryption", "privacy", "backup", "cloud"]

[dependencies]
# Use YOUR file-crypt for encryption!
# Option 1: If file-crypt is published to crates.io
# file-crypt = "0.1"

# Option 2: Use it as a local dependency during development
# file-crypt = { path = "../file-crypt" }

# Option 3: Use it from GitHub
# file-crypt = { git = "https://github.com/pratikdevelop/file-crypt" }

# Async runtime
tokio = { version = "1.41", features = ["full"] }
async-trait = "0.1"

# File system watching (THE KEY DIFFERENCE!)
notify = "6.1"
notify-debouncer-full = "0.3"

# Cloud storage
object_store = "0.11"  # S3, Azure, GCS support
aws-config = "1.5"
aws-sdk-s3 = "1.55"

# Local database for sync state
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# CLI
clap = { version = "4.5", features = ["derive"] }
colored = "2.1"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Utilities
uuid = { version = "1.11", features = ["v4"] }
chrono = "0.4"
walkdir = "2.5"

[dev-dependencies]
tempfile = "3.13"
