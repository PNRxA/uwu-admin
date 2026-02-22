use std::sync::Arc;
use tokio::sync::Mutex;
use sea_orm::DatabaseConnection;

use crate::matrix::MatrixClient;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub client: Mutex<Option<MatrixClient>>,
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> SharedState {
        Arc::new(AppState {
            client: Mutex::new(None),
            db,
        })
    }
}
