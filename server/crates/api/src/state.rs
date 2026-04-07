use sqlx::PgPool;

use crate::config::Config;
use crate::ws::{ChatHub, EventHub};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub chat_hub: ChatHub,
    pub event_hub: EventHub,
}

impl AppState {
    pub fn new(db: PgPool, config: Config) -> Self {
        Self {
            db,
            config,
            chat_hub: ChatHub::new(),
            event_hub: EventHub::new(),
        }
    }
}
