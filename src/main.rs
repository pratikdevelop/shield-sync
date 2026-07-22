use clap::{Parser, Subcommand};
use shield_sync::crypto::{MasterKey, CryptoEngine};
use shield_sync::Result;
use std::path::PathBuf;
use anyhow::Context;

#[derive(Parser)]
#[command(
    name = "shield-sync",
    about = "End-to-end encrypted file synchronization",
    version,
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new encrypted sync vault
    Init {
        /// Folder to synchronize
        path: PathBuf,

        /// Master password (will be used to derive encryption keys)
        #[arg(short, long)]
        password: String,
    },

    /// Encrypt a single file (mainly for testing)
    Encrypt {
        /// Input plaintext file
        input: PathBuf,

        /// Output encrypted file
        output: PathBuf,

        /// Password
        #[arg(short, long)]
        password: String,
    },

    /// Decrypt a single file (mainly for testing)
    Decrypt {
        /// Input encrypted file
        input: PathBuf,

        /// Output plaintext file
        output: PathBuf,

        /// Password
        #[arg(short, long)]
        password: String,
    },

    /// Daemon control (background sync process)
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },

    /// Manage remote storage backends
    Remote {
        #[command(subcommand)]
        action: RemoteAction,
    },

    /// Show current synchronization status
    Status,

    /// Show recent sync history
    History,
}

#[derive(Subcommand)]
enum DaemonAction {
    Start,
    Stop,
    Restart,
    Status,
}

#[derive(Subcommand)]
enum RemoteAction {
    /// Add a new remote storage (e.g. S3, WebDAV, Dropbox, ...)
    Add {
        /// Friendly name for this remote
        name: String,
        /// Connection string / URL
        url: String,
    },
    List,
    Remove {
        name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path, password } => {
            println!("🛡️  Initializing Shield-Sync vault at: {}", path.display());

            if !path.exists() {
                tokio::fs::create_dir_all(&path)
                    .await
                    .context("Failed to create vault directory")?;
            }

            // In real app you would:
            // • Store salt + encrypted config / metadata
            // • Create local sqlite / sled / ... database
            // • Save encrypted master key material or key derivation parameters

            let salt = MasterKey::generate_salt();
            let master_key = MasterKey::derive_from_password(&password, &salt)?;

            // TODO: save salt + encrypted vault metadata / config
            println!("🔐 Master key derived (salt: {} bytes)", salt.len());
            println!("📁 Vault folder ready. Add remotes with: shield-sync remote add ...");

            println!("✅ Initialization complete!");
            Ok(())
        }

        Commands::Encrypt { input, output, password } => {
            println!("🔒 Encrypting {} → {}", input.display(), output.display());

            let salt = MasterKey::generate_salt();
            let master_key = MasterKey::derive_from_password(&password, &salt)?;
            let engine = CryptoEngine::new(master_key);

            let encrypted = engine.encrypt_file(&input).await?;

            let mut out_data = Vec::with_capacity(salt.len() + encrypted.len());
            out_data.extend_from_slice(&salt);
            out_data.extend_from_slice(&encrypted);

            tokio::fs::write(&output, &out_data).await?;

            println!("✅ Encrypted file saved");
            Ok(())
        }

        Commands::Decrypt { input, output, password } => {
            println!("🔓 Decrypting {} → {}", input.display(), output.display());

            let data = tokio::fs::read(&input).await?;
            if data.len() < 16 {
                anyhow::bail!("File too small — not a valid encrypted file");
            }

            let (salt, ciphertext) = data.split_at(16);

            let master_key = MasterKey::derive_from_password(&password, salt)?;
            let engine = CryptoEngine::new(master_key);

            engine.decrypt_file(ciphertext, &output).await?;

            println!("✅ Decrypted successfully");
            Ok(())
        }

        Commands::Daemon { action } => match action {
            DaemonAction::Start => {
                println!("🚀 Starting Shield-Sync daemon ...");
                println!("👁️  Watching filesystem → {}", "TODO: path from config");
                println!("🔄 Syncing with remotes → {}", "TODO: list of remotes");

                // TODO:
                // • Load config + encrypted master key
                // • Start file watcher (notify, inotify, recommended)
                // • Start periodic sync loop / event-driven sync
                // • Handle graceful shutdown

                tokio::signal::ctrl_c()
                    .await
                    .context("Failed to listen for Ctrl+C")?;

                println!("🛑 Received shutdown signal");
                // TODO: graceful shutdown
                Ok(())
            }
            DaemonAction::Status => {
                println!("Daemon status: {}", "TODO: check pid file / socket / service");
                println!("(not implemented yet)");
                Ok(())
            }
            _ => {
                println!("⚠️  {} not implemented yet", action_variant_name(&action));
                Ok(())
            }
        },

        Commands::Status => {
            println!("📊 Sync Status (TODO: read from real state)");
            println!("──────────────────────────────");
            println!("✅ Synced files    : 142");
            println!("⏳ Pending changes : 3");
            println!("⚠️  Conflicts      : 0");
            println!("📡 Remotes         : 1 (gdrive-personal)");
            Ok(())
        }

        _ => {
            println!("Command not implemented yet:");
            Ok(())
        }
    }
}

fn action_variant_name(a: &DaemonAction) -> &'static str {
    match a {
        DaemonAction::Start => "start",
        DaemonAction::Stop => "stop",
        DaemonAction::Restart => "restart",
        DaemonAction::Status => "status",
    }
}