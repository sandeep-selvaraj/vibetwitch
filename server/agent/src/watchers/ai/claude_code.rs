use std::collections::HashSet;
use std::path::PathBuf;

use tokio::sync::mpsc;
use vibetwitch_shared::events::{
    AiMessagePayload, AiResponsePayload, CodeEvent, ToolUsePayload,
};

/// Watch Claude Code's JSONL conversation logs for new messages.
///
/// Claude Code stores conversations in ~/.claude/projects/<project-hash>/
/// Each conversation is a JSONL file where each line is a message object.
pub async fn watch(tx: mpsc::UnboundedSender<CodeEvent>) {
    let claude_dir = match find_claude_dir() {
        Some(d) => d,
        None => {
            tracing::warn!("Could not find ~/.claude directory, Claude Code watcher disabled");
            // Keep the task alive so it doesn't terminate
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        }
    };

    tracing::info!(path = %claude_dir.display(), "Watching Claude Code conversations");

    let mut seen_lines: HashSet<String> = HashSet::new();
    let mut known_files: HashSet<PathBuf> = HashSet::new();

    loop {
        // Scan for JSONL files in the claude projects directory
        if let Ok(entries) = std::fs::read_dir(&claude_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // Look in subdirectories for conversation files
                if path.is_dir() {
                    scan_conversation_dir(&path, &tx, &mut seen_lines, &mut known_files);
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

fn scan_conversation_dir(
    dir: &PathBuf,
    tx: &mpsc::UnboundedSender<CodeEvent>,
    seen_lines: &mut HashSet<String>,
    known_files: &mut HashSet<PathBuf>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "jsonl") {
            process_jsonl_file(&path, tx, seen_lines, known_files);
        }
    }
}

fn process_jsonl_file(
    path: &PathBuf,
    tx: &mpsc::UnboundedSender<CodeEvent>,
    seen_lines: &mut HashSet<String>,
    known_files: &mut HashSet<PathBuf>,
) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let is_new_file = known_files.insert(path.clone());

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Use a hash of the line content to deduplicate
        // Use first 100 chars + length as key for deduplication
        let dedup_key = format!(
            "{}:{}:{}",
            path.display(),
            line.len(),
            &line[..line.len().min(100)]
        );

        if !seen_lines.insert(dedup_key) {
            continue;
        }

        // Skip existing content when we first discover a file
        if is_new_file {
            continue;
        }

        // Parse the JSONL line
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };

        let session_id = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let role = value.get("role").and_then(|r| r.as_str()).unwrap_or("");

        match role {
            "human" | "user" => {
                let content = extract_content(&value);
                if !content.is_empty() {
                    let _ = tx.send(CodeEvent::AiMessage(AiMessagePayload {
                        tool: "claude_code".into(),
                        content,
                        session_id: session_id.clone(),
                    }));
                }
            }
            "assistant" => {
                // Check for tool use blocks
                if let Some(content_arr) = value.get("content").and_then(|c| c.as_array()) {
                    for block in content_arr {
                        let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");

                        match block_type {
                            "text" => {
                                let text = block.get("text").and_then(|t| t.as_str()).unwrap_or("");
                                if !text.is_empty() {
                                    let _ = tx.send(CodeEvent::AiResponse(AiResponsePayload {
                                        tool: "claude_code".into(),
                                        content: text.to_string(),
                                        model: value
                                            .get("model")
                                            .and_then(|m| m.as_str())
                                            .map(|s| s.to_string()),
                                        session_id: session_id.clone(),
                                    }));
                                }
                            }
                            "tool_use" => {
                                let tool_name = block
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("unknown");
                                let input = block.get("input");
                                let file_path = input
                                    .and_then(|i| {
                                        i.get("file_path")
                                            .or_else(|| i.get("path"))
                                    })
                                    .and_then(|p| p.as_str())
                                    .map(|s| s.to_string());

                                let summary = summarize_tool_use(tool_name, input);

                                let _ = tx.send(CodeEvent::ToolUse(ToolUsePayload {
                                    tool: "claude_code".into(),
                                    tool_name: tool_name.to_string(),
                                    file_path,
                                    summary,
                                    session_id: session_id.clone(),
                                }));
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Simple text response
                    let content = extract_content(&value);
                    if !content.is_empty() {
                        let _ = tx.send(CodeEvent::AiResponse(AiResponsePayload {
                            tool: "claude_code".into(),
                            content,
                            model: None,
                            session_id: session_id.clone(),
                        }));
                    }
                }
            }
            _ => {}
        }
    }
}

fn extract_content(value: &serde_json::Value) -> String {
    if let Some(s) = value.get("content").and_then(|c| c.as_str()) {
        return s.to_string();
    }
    if let Some(arr) = value.get("content").and_then(|c| c.as_array()) {
        let mut parts = Vec::new();
        for block in arr {
            if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                    parts.push(text.to_string());
                }
            }
        }
        return parts.join("\n");
    }
    String::new()
}

fn summarize_tool_use(tool_name: &str, input: Option<&serde_json::Value>) -> String {
    match tool_name {
        "Read" => {
            let path = input
                .and_then(|i| i.get("file_path").and_then(|p| p.as_str()))
                .unwrap_or("unknown");
            format!("Reading {path}")
        }
        "Write" => {
            let path = input
                .and_then(|i| i.get("file_path").and_then(|p| p.as_str()))
                .unwrap_or("unknown");
            format!("Writing {path}")
        }
        "Edit" => {
            let path = input
                .and_then(|i| i.get("file_path").and_then(|p| p.as_str()))
                .unwrap_or("unknown");
            format!("Editing {path}")
        }
        "Bash" => {
            let cmd = input
                .and_then(|i| i.get("command").and_then(|c| c.as_str()))
                .unwrap_or("...");
            let short = if cmd.len() > 60 { &cmd[..60] } else { cmd };
            format!("Running: {short}")
        }
        "Glob" => {
            let pattern = input
                .and_then(|i| i.get("pattern").and_then(|p| p.as_str()))
                .unwrap_or("*");
            format!("Searching for {pattern}")
        }
        "Grep" => {
            let pattern = input
                .and_then(|i| i.get("pattern").and_then(|p| p.as_str()))
                .unwrap_or("...");
            format!("Grepping for {pattern}")
        }
        _ => format!("Using {tool_name}"),
    }
}

fn find_claude_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let claude_dir = PathBuf::from(home).join(".claude").join("projects");
    if claude_dir.exists() {
        Some(claude_dir)
    } else {
        None
    }
}
