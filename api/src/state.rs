use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use sea_orm::DatabaseConnection;
use zeroize::Zeroizing;

use crate::services::matrix::MatrixClient;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub clients: Mutex<HashMap<i32, Arc<Mutex<MatrixClient>>>>,
    pub db: DatabaseConnection,
    pub jwt_secret: Zeroizing<Vec<u8>>,
    pub encryption_key: Zeroizing<Vec<u8>>,
    pub secure_cookies: bool,
}

impl AppState {
    pub fn new(
        db: DatabaseConnection,
        jwt_secret: Zeroizing<Vec<u8>>,
        encryption_key: Zeroizing<Vec<u8>>,
        secure_cookies: bool,
    ) -> SharedState {
        Arc::new(AppState {
            clients: Mutex::new(HashMap::new()),
            db,
            jwt_secret,
            encryption_key,
            secure_cookies,
        })
    }
}
