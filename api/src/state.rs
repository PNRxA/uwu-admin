use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::SqlitePool;

use crate::matrix::MatrixClient;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub client: Mutex<Option<MatrixClient>>,
    pub db: SqlitePool,
}

impl AppState {
    pub fn new(db: SqlitePool) -> SharedState {
        Arc::new(AppState {
            client: Mutex::new(None),
            db,
        })
    }
}
