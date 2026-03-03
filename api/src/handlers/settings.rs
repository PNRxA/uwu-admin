use std::collections::HashMap;

use axum::Json;
use axum::extract::State;
use serde_json::Value;

use crate::error::ApiError;
use crate::services::db;
use crate::state::SharedState;

const SETTING_DEFAULTS: &[(&str, &str)] = &[
    ("redact_messages", "true"),
    ("theme", r#"{"activeThemeId":"uwu","customThemes":[]}"#),
    ("flavour_text", "true"),
];

const ALLOWED_KEYS: &[&str] = &["redact_messages", "theme", "flavour_text"];

const BOOLEAN_KEYS: &[&str] = &["redact_messages", "flavour_text"];

const MAX_THEME_SIZE: usize = 10_240;

const PUBLIC_KEYS: &[&str] = &["theme", "flavour_text"];

fn build_settings_map(
    rows: &[crate::entity::setting::Model],
    filter: &[&str],
) -> serde_json::Map<String, Value> {
    let mut map: HashMap<String, String> = SETTING_DEFAULTS
        .iter()
        .filter(|&&(k, _)| filter.contains(&k))
        .map(|&(k, v)| (k.to_owned(), v.to_owned()))
        .collect();
    for row in rows {
        if filter.contains(&row.key.as_str()) {
            map.insert(row.key.clone(), row.value.clone());
        }
    }
    map.into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect()
}

pub async fn get_public_settings(
    State(state): State<SharedState>,
) -> Result<Json<Value>, ApiError> {
    let rows = db::get_all_settings(&state.db).await?;
    Ok(Json(Value::Object(build_settings_map(&rows, PUBLIC_KEYS))))
}

pub async fn get_settings(
    State(state): State<SharedState>,
) -> Result<Json<Value>, ApiError> {
    let rows = db::get_all_settings(&state.db).await?;
    Ok(Json(Value::Object(build_settings_map(&rows, ALLOWED_KEYS))))
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
        if key == "theme" {
            if value.len() > MAX_THEME_SIZE {
                return Err(ApiError::BadRequest(
                    "Theme setting exceeds maximum size".to_string(),
                ));
            }
            let parsed: Value = serde_json::from_str(value).map_err(|_| {
                ApiError::BadRequest("Theme setting must be valid JSON".to_string())
            })?;
            if !parsed.get("activeThemeId").is_some_and(|v| v.is_string()) {
                return Err(ApiError::BadRequest(
                    "Theme must have an 'activeThemeId' string".to_string(),
                ));
            }
            let custom_themes = parsed.get("customThemes").and_then(|v| v.as_array());
            let Some(custom_themes) = custom_themes else {
                return Err(ApiError::BadRequest(
                    "Theme must have a 'customThemes' array".to_string(),
                ));
            };
            for (i, item) in custom_themes.iter().enumerate() {
                let Some(obj) = item.as_object() else {
                    return Err(ApiError::BadRequest(
                        format!("customThemes[{i}] must be an object"),
                    ));
                };
                if !obj.get("id").is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty())) {
                    return Err(ApiError::BadRequest(
                        format!("customThemes[{i}].id must be a non-empty string"),
                    ));
                }
                if !obj.get("name").is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty())) {
                    return Err(ApiError::BadRequest(
                        format!("customThemes[{i}].name must be a non-empty string"),
                    ));
                }
                match obj.get("hue").and_then(|v| v.as_f64()) {
                    Some(h) if (0.0..=360.0).contains(&h) => {}
                    _ => {
                        return Err(ApiError::BadRequest(
                            format!("customThemes[{i}].hue must be a number between 0 and 360"),
                        ));
                    }
                }
                match obj.get("chromaScale").and_then(|v| v.as_f64()) {
                    Some(c) if (0.3..=1.5).contains(&c) => {}
                    _ => {
                        return Err(ApiError::BadRequest(
                            format!("customThemes[{i}].chromaScale must be a number between 0.3 and 1.5"),
                        ));
                    }
                }
            }
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
