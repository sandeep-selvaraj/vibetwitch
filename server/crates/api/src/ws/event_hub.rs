use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::broadcast;
use uuid::Uuid;
use vibetwitch_shared::events::CodeEvent;

/// A sequenced code event broadcast to viewers.
#[derive(Debug, Clone)]
pub struct EventBroadcast {
    pub event: CodeEvent,
    pub sequence: i64,
}

struct EventRoom {
    tx: broadcast::Sender<EventBroadcast>,
    sequence: AtomicI64,
}

/// In-memory event hub. One broadcast channel per active stream.
#[derive(Clone)]
pub struct EventHub {
    rooms: Arc<DashMap<Uuid, Arc<EventRoom>>>,
}

impl EventHub {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
        }
    }

    fn get_or_create_room(&self, stream_id: Uuid) -> Arc<EventRoom> {
        self.rooms
            .entry(stream_id)
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(512);
                Arc::new(EventRoom {
                    tx,
                    sequence: AtomicI64::new(0),
                })
            })
            .clone()
    }

    /// Subscribe to a stream's code events.
    pub fn subscribe(&self, stream_id: Uuid) -> broadcast::Receiver<EventBroadcast> {
        let room = self.get_or_create_room(stream_id);
        room.tx.subscribe()
    }

    /// Publish an event from the streamer agent. Returns the assigned sequence number.
    pub fn publish(&self, stream_id: Uuid, event: CodeEvent) -> i64 {
        let room = self.get_or_create_room(stream_id);
        let seq = room.sequence.fetch_add(1, Ordering::Relaxed) + 1;
        let _ = room.tx.send(EventBroadcast {
            event,
            sequence: seq,
        });
        seq
    }

    /// Clean up a room when a stream ends.
    #[allow(dead_code)]
    pub fn remove_room(&self, stream_id: Uuid) {
        self.rooms.remove(&stream_id);
    }
}
