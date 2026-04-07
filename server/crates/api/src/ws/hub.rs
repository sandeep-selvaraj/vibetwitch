use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

/// A chat message broadcast through the hub.
#[derive(Debug, Clone)]
pub struct ChatBroadcast {
    pub username: String,
    pub display_name: String,
    pub content: String,
    pub timestamp: String,
}

/// Per-stream room state.
struct Room {
    tx: broadcast::Sender<ChatBroadcast>,
    viewer_count: AtomicU32,
}

/// In-memory chat hub. One broadcast channel per active stream.
#[derive(Clone)]
pub struct ChatHub {
    rooms: Arc<DashMap<Uuid, Arc<Room>>>,
}

impl ChatHub {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
        }
    }

    fn get_or_create_room(&self, stream_id: Uuid) -> Arc<Room> {
        self.rooms
            .entry(stream_id)
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(256);
                Arc::new(Room {
                    tx,
                    viewer_count: AtomicU32::new(0),
                })
            })
            .clone()
    }

    /// Subscribe to a stream's chat. Returns a receiver and increments viewer count.
    pub fn join(&self, stream_id: Uuid) -> (broadcast::Receiver<ChatBroadcast>, u32) {
        let room = self.get_or_create_room(stream_id);
        let rx = room.tx.subscribe();
        let count = room.viewer_count.fetch_add(1, Ordering::Relaxed) + 1;
        (rx, count)
    }

    /// Decrement viewer count when a viewer disconnects.
    pub fn leave(&self, stream_id: Uuid) -> u32 {
        if let Some(room) = self.rooms.get(&stream_id) {
            let prev = room.viewer_count.fetch_sub(1, Ordering::Relaxed);
            let count = prev.saturating_sub(1);
            // Clean up empty rooms
            if count == 0 && room.tx.receiver_count() == 0 {
                drop(room);
                self.rooms.remove(&stream_id);
                return 0;
            }
            count
        } else {
            0
        }
    }

    /// Broadcast a message to all viewers of a stream.
    pub fn send(&self, stream_id: Uuid, msg: ChatBroadcast) {
        if let Some(room) = self.rooms.get(&stream_id) {
            // Ignore send errors (no receivers)
            let _ = room.tx.send(msg);
        }
    }

    /// Get the current viewer count for a stream.
    pub fn viewer_count(&self, stream_id: Uuid) -> u32 {
        self.rooms
            .get(&stream_id)
            .map(|r| r.viewer_count.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
}
