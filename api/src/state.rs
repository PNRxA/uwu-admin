use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use sea_orm::DatabaseConnection;

use crate::matrix::MatrixClient;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub clients: Mutex<HashMap<i32, MatrixClient>>,
    pub db: DatabaseConnection,
    pub jwt_secret: Vec<u8>,
    pub encryption_key: Vec<u8>,
}

impl AppState {
    pub fn new(db: DatabaseConnection, jwt_secret: Vec<u8>, encryption_key: Vec<u8>) -> SharedState {
        Arc::new(AppState {
            clients: Mutex::new(HashMap::new()),
            db,
            jwt_secret,
            encryption_key,
        })
    }
}
