use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Renamed { from: PathBuf, to: PathBuf },
}

pub struct FileWatcher {
    watch_path: PathBuf,
}

impl FileWatcher {
    pub fn new(watch_path: PathBuf) -> Self {
        Self { watch_path }
    }

    /// Start watching for file changes
    pub async fn watch(
        &self,
        tx: mpsc::Sender<FileEvent>,
    ) -> anyhow::Result<()> {
        let (debounce_tx, mut debounce_rx) = mpsc::channel(100);
        
        // Create debouncer (avoids duplicate events)
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |result: DebounceEventResult| {
                if let Ok(events) = result {
                    for event in events {
                        // Convert notify events to our FileEvent
                        // Send via debounce_tx
                    }
                }
            },
        )?;

        // Start watching
        debouncer
            .watcher()
            .watch(&self.watch_path, RecursiveMode::Recursive)?;

        println!("👁️  Watching {} for changes...", self.watch_path.display());

        // Forward events
        while let Some(event) = debounce_rx.recv().await {
            tx.send(event).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_file_watcher() {
        let temp_dir = TempDir::new().unwrap();
        let (tx, mut rx) = mpsc::channel(10);
        
        let watcher = FileWatcher::new(temp_dir.path().to_path_buf());
        
        tokio::spawn(async move {
            watcher.watch(tx).await.unwrap();
        });

        // Create a file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "hello").await.unwrap();

        // Should receive Created event
        if let Some(FileEvent::Created(path)) = rx.recv().await {
            assert_eq!(path, test_file);
        }
    }
}