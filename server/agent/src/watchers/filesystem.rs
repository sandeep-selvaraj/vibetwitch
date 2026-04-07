use std::path::PathBuf;

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use vibetwitch_shared::events::{CodeEvent, FileChangePayload, FileChangeType};

/// Watch a project directory for file system changes using inotify/fsevents.
pub async fn watch(project_dir: PathBuf, tx: mpsc::UnboundedSender<CodeEvent>) {
    tracing::info!(path = %project_dir.display(), "Watching file system changes");

    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel();

    let mut watcher = match RecommendedWatcher::new(
        move |result: Result<Event, notify::Error>| {
            if let Ok(event) = result {
                let _ = notify_tx.send(event);
            }
        },
        Config::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("Failed to create file watcher: {e}");
            return;
        }
    };

    if let Err(e) = watcher.watch(&project_dir, RecursiveMode::Recursive) {
        tracing::error!("Failed to watch directory: {e}");
        return;
    }

    while let Some(event) = notify_rx.recv().await {
        for path in &event.paths {
            // Skip hidden directories and common noisy paths
            let path_str = path.to_string_lossy();
            if should_ignore(&path_str) {
                continue;
            }

            let relative = path
                .strip_prefix(&project_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            let change_type = match event.kind {
                EventKind::Create(_) => FileChangeType::Created,
                EventKind::Modify(_) => FileChangeType::Modified,
                EventKind::Remove(_) => FileChangeType::Deleted,
                _ => continue,
            };

            let language = detect_language_from_path(&relative);

            let _ = tx.send(CodeEvent::FileChange(FileChangePayload {
                file_path: relative,
                change_type,
                language,
            }));
        }
    }

    // Keep watcher alive
    drop(watcher);
}

fn should_ignore(path: &str) -> bool {
    let ignore_patterns = [
        "/.git/",
        "/node_modules/",
        "/target/",
        "/.next/",
        "/dist/",
        "/__pycache__/",
        "/.venv/",
        "/.DS_Store",
        "/venv/",
        "/.claude/",
    ];
    ignore_patterns.iter().any(|p| path.contains(p))
}

fn detect_language_from_path(path: &str) -> Option<String> {
    let ext = path.rsplit('.').next()?;
    let lang = match ext {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "go" => "go",
        "css" => "css",
        "html" => "html",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "sql" => "sql",
        _ => return None,
    };
    Some(lang.to_string())
}
