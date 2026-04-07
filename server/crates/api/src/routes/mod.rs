pub mod auth;
pub mod hooks;
pub mod streams;
pub mod users;

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/streams", streams::router())
        .nest("/hooks", hooks::router())
}
