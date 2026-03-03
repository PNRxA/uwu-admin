use std::collections::HashMap;

use axum::Json;
use axum::extract::State;
use serde_json::Value;

use crate::error::ApiError;
use crate::services::db;
use crate::state::SharedState;

const SETTING_DEFAULTS: &[(&str, &str)] = &[("redact_messages", "true")];

const ALLOWED_KEYS: &[&str] = &["redact_messages"];

const BOOLEAN_KEYS: &[&str] = &["redact_messages"];

pub async fn get_settings(
    State(state): State<SharedState>,
) -> Result<Json<Value>, ApiError> {
    let rows = db::get_all_settings(&state.db).await?;
    let mut map: HashMap<String, String> =
        SETTING_DEFAULTS.iter().map(|&(k, v)| (k.to_owned(), v.to_owned())).collect();
    for row in &rows {
        if ALLOWED_KEYS.contains(&row.key.as_str()) {
            map.insert(row.key.clone(), row.value.clone());
        }
    }
    let obj: serde_json::Map<String, Value> = map
        .into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect();
    Ok(Json(Value::Object(obj)))
}

pub async fn update_settings(
    State(state): State<SharedState>,
    Json(body): Json<HashMap<String, String>>,
) -> Result<Json<Value>, ApiError> {
    for key in body.keys() {
        if !ALLOWED_KEYS.contains(&key.as_str()) {
            return Err(ApiError::BadRequest(format!("Unknown setting: {key}")));
        }
    }
    for (key, value) in &body {
        if BOOLEAN_KEYS.contains(&key.as_str()) && value != "true" && value != "false" {
            return Err(ApiError::BadRequest(format!(
                "Setting '{key}' must be \"true\" or \"false\""
            )));
        }
    }
    for (key, value) in &body {
        db::set_setting(&state.db, key, value).await?;
    }
    {
        let mut cache = state.settings_cache.write().unwrap();
        for (key, value) in &body {
            cache.insert(key.clone(), value.clone());
        }
    }
    get_settings(State(state)).await
}
