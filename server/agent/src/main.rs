mod transport;
mod watchers;

use clap::Parser;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;
use vibetwitch_shared::events::CodeEvent;

#[derive(Parser)]
#[command(name = "vibetwitch-agent", about = "Stream your AI coding sessions to VibeTwitch")]
struct Args {
    /// Project directory to watch
    #[arg(short, long, default_value = ".")]
    project_dir: String,

    /// VibeTwitch server URL
    #[arg(short, long, default_value = "ws://localhost:3001")]
    server_url: String,

    /// Your stream ID (UUID)
    #[arg(long)]
    stream_id: String,

    /// Your stream key
    #[arg(short = 'k', long)]
    stream_key: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let args = Args::parse();

    tracing::info!(
        project_dir = %args.project_dir,
        stream_id = %args.stream_id,
        "Starting VibeTwitch agent"
    );

    let (tx, rx) = mpsc::unbounded_channel::<CodeEvent>();

    // Spawn watchers
    let project_dir = std::path::PathBuf::from(&args.project_dir)
        .canonicalize()
        .expect("Invalid project directory");

    // Claude Code watcher
    let tx_claude = tx.clone();
    tokio::spawn(watchers::ai::claude_code::watch(tx_claude));

    // Git diff watcher
    let tx_git = tx.clone();
    let project_dir_git = project_dir.clone();
    tokio::spawn(watchers::git::watch(project_dir_git, tx_git));

    // File system watcher
    let tx_fs = tx.clone();
    let project_dir_fs = project_dir.clone();
    tokio::spawn(watchers::filesystem::watch(project_dir_fs, tx_fs));

    // WebSocket transport — sends events to server
    let ws_url = format!(
        "{}/ws/ingest/{}?stream_key={}",
        args.server_url, args.stream_id, args.stream_key
    );

    tracing::info!("Connecting to {}", ws_url);
    transport::run(ws_url, rx).await;
}
