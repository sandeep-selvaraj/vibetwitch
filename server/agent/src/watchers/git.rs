use std::path::PathBuf;

use tokio::sync::mpsc;
use vibetwitch_shared::events::{CodeDiffPayload, CodeEvent};

/// Watch a project directory for git changes by periodically running git diff.
pub async fn watch(project_dir: PathBuf, tx: mpsc::UnboundedSender<CodeEvent>) {
    tracing::info!(path = %project_dir.display(), "Watching git diffs");

    let mut last_diff = String::new();

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        // Get current diff (unstaged + staged)
        let diff = match get_git_diff(&project_dir) {
            Some(d) => d,
            None => continue,
        };

        if diff == last_diff || diff.is_empty() {
            continue;
        }

        // Parse the diff into per-file events
        let file_diffs = parse_diff_by_file(&diff);

        for file_diff in file_diffs {
            let language = detect_language(&file_diff.file_path);
            let additions = file_diff.diff.lines().filter(|l| l.starts_with('+')).count() as u32;
            let deletions = file_diff.diff.lines().filter(|l| l.starts_with('-')).count() as u32;

            let _ = tx.send(CodeEvent::CodeDiff(CodeDiffPayload {
                file_path: file_diff.file_path,
                diff: file_diff.diff,
                language,
                additions,
                deletions,
            }));
        }

        last_diff = diff;
    }
}

fn get_git_diff(project_dir: &PathBuf) -> Option<String> {
    // Combined: unstaged + staged changes
    let output = std::process::Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(project_dir)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

struct FileDiff {
    file_path: String,
    diff: String,
}

fn parse_diff_by_file(diff: &str) -> Vec<FileDiff> {
    let mut result = Vec::new();
    let mut current_file: Option<String> = None;
    let mut current_diff = String::new();

    for line in diff.lines() {
        if line.starts_with("diff --git") {
            // Save previous file diff
            if let Some(ref file) = current_file {
                if !current_diff.is_empty() {
                    result.push(FileDiff {
                        file_path: file.clone(),
                        diff: current_diff.clone(),
                    });
                }
            }
            // Extract filename from "diff --git a/path b/path"
            let parts: Vec<&str> = line.split(' ').collect();
            current_file = parts
                .last()
                .map(|p| p.strip_prefix("b/").unwrap_or(p).to_string());
            current_diff.clear();
        } else {
            current_diff.push_str(line);
            current_diff.push('\n');
        }
    }

    // Don't forget the last file
    if let Some(ref file) = current_file {
        if !current_diff.is_empty() {
            result.push(FileDiff {
                file_path: file.clone(),
                diff: current_diff,
            });
        }
    }

    result
}

fn detect_language(path: &str) -> Option<String> {
    let ext = path.rsplit('.').next()?;
    let lang = match ext {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "rb" => "ruby",
        "cpp" | "cc" | "cxx" => "cpp",
        "c" | "h" => "c",
        "css" => "css",
        "html" => "html",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" => "markdown",
        "sql" => "sql",
        "sh" | "bash" | "zsh" => "shell",
        _ => return None,
    };
    Some(lang.to_string())
}
