use std::sync::Arc;
use tokio::sync::Mutex;

use crate::matrix::MatrixClient;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub client: Mutex<Option<MatrixClient>>,
}

impl AppState {
    pub fn new() -> SharedState {
        Arc::new(AppState {
            client: Mutex::new(None),
        })
    }
}
