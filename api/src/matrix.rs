use sea_orm::DatabaseConnection;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::db;
use crate::error::ApiError;

pub struct MatrixClient {
    http: reqwest::Client,
    pub homeserver: String,
    access_token: String,
    pub room_id: String,
    pub user_id: String,
    since: Option<String>,
}

impl MatrixClient {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn since(&self) -> Option<&str> {
        self.since.as_deref()
    }

    pub async fn login(
        homeserver: &str,
        username: &str,
        password: &str,
        room_id: &str,
    ) -> Result<Self, ApiError> {
        let http = reqwest::Client::new();
        let hs = homeserver.trim_end_matches('/');

        let login_url = format!("{hs}/_matrix/client/v3/login");
        let body = json!({
            "type": "m.login.password",
            "identifier": {
                "type": "m.id.user",
                "user": username
            },
            "password": password
        });

        let resp = http
            .post(&login_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ApiError::MatrixError(e.to_string()))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ApiError::MatrixError(format!("Login failed: {text}")));
        }

        let data: Value = resp
            .json()
            .await
            .map_err(|e| ApiError::MatrixError(e.to_string()))?;

        let access_token = data["access_token"]
            .as_str()
            .ok_or_else(|| ApiError::MatrixError("No access_token in response".into()))?
            .to_string();

        let user_id = data["user_id"]
            .as_str()
            .ok_or_else(|| ApiError::MatrixError("No user_id in response".into()))?
            .to_string();

        let resolved_room_id = if room_id.starts_with('#') {
            resolve_alias(&http, hs, &access_token, room_id).await?
        } else {
            room_id.to_string()
        };

        let mut client = MatrixClient {
            http,
            homeserver: hs.to_string(),
            access_token,
            room_id: resolved_room_id,
            user_id,
            since: None,
        };

        client.initial_sync().await?;
        Ok(client)
    }

    pub async fn restore(
        homeserver: String,
        access_token: String,
        room_id: String,
        user_id: String,
        since: Option<String>,
    ) -> Result<Self, ApiError> {
        let client = MatrixClient {
            http: reqwest::Client::new(),
            homeserver,
            access_token,
            room_id,
            user_id,
            since,
        };

        client.validate_token().await?;
        Ok(client)
    }

    async fn validate_token(&self) -> Result<(), ApiError> {
        let url = format!(
            "{}/_matrix/client/v3/sync?timeout=0&filter={{\"room\":{{\"timeline\":{{\"limit\":0}}}}}}",
            self.homeserver
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| ApiError::MatrixError(e.to_string()))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ApiError::MatrixError(format!("Token validation failed: {text}")));
        }

        Ok(())
    }

    async fn initial_sync(&mut self) -> Result<(), ApiError> {
        let url = format!(
            "{}/_matrix/client/v3/sync?timeout=0&filter={{\"room\":{{\"timeline\":{{\"limit\":0}}}}}}",
            self.homeserver
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| ApiError::MatrixError(e.to_string()))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ApiError::MatrixError(format!("Initial sync failed: {text}")));
        }

        let data: Value = resp
            .json()
            .await
            .map_err(|e| ApiError::MatrixError(e.to_string()))?;

        self.since = data["next_batch"].as_str().map(|s| s.to_string());
        Ok(())
    }

    async fn send_message(&self, body: &str) -> Result<(), ApiError> {
        let txn_id = Uuid::new_v4().to_string();
        let url = format!(
            "{}/_matrix/client/v3/rooms/{}/send/m.room.message/{txn_id}",
            self.homeserver,
            urlencoded(&self.room_id)
        );

        let msg = json!({
            "msgtype": "m.text",
            "body": body
        });

        let resp = self
            .http
            .put(&url)
            .bearer_auth(&self.access_token)
            .json(&msg)
            .send()
            .await
            .map_err(|e| ApiError::MatrixError(e.to_string()))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ApiError::MatrixError(format!("Send failed: {text}")));
        }

        Ok(())
    }

    async fn wait_for_response(
        &mut self,
        server_id: i32,
        db: &DatabaseConnection,
    ) -> Result<String, ApiError> {
        for _ in 0..3 {
            let mut url = format!(
                "{}/_matrix/client/v3/sync?timeout=10000",
                self.homeserver
            );
            if let Some(since) = &self.since {
                url.push_str(&format!("&since={since}"));
            }

            let resp = self
                .http
                .get(&url)
                .bearer_auth(&self.access_token)
                .send()
                .await
                .map_err(|e| ApiError::MatrixError(e.to_string()))?;

            if !resp.status().is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(ApiError::MatrixError(format!("Sync failed: {text}")));
            }

            let data: Value = resp
                .json()
                .await
                .map_err(|e| ApiError::MatrixError(e.to_string()))?;

            self.since = data["next_batch"].as_str().map(|s| s.to_string());

            if let Some(ref since) = self.since {
                if let Err(e) = db::update_server_since(db, server_id, since).await {
                    tracing::warn!("Failed to persist since token: {e}");
                }
            }

            // Look for messages in the admin room not from our user
            if let Some(events) = data["rooms"]["join"][&self.room_id]["timeline"]["events"]
                .as_array()
            {
                for event in events {
                    let sender = event["sender"].as_str().unwrap_or_default();
                    let msg_type = event["type"].as_str().unwrap_or_default();
                    if sender != self.user_id && msg_type == "m.room.message" {
                        let body = event["content"]["body"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string();
                        // Also check formatted_body for HTML responses
                        let formatted = event["content"]["formatted_body"]
                            .as_str()
                            .map(|s| s.to_string());
                        return Ok(formatted.unwrap_or(body));
                    }
                }
            }
        }

        Err(ApiError::Timeout)
    }

    pub async fn execute_command(
        &mut self,
        command: &str,
        server_id: i32,
        db: &DatabaseConnection,
    ) -> Result<String, ApiError> {
        let message = format!("!admin {command}");
        self.send_message(&message).await?;
        self.wait_for_response(server_id, db).await
    }
}

async fn resolve_alias(
    http: &reqwest::Client,
    homeserver: &str,
    access_token: &str,
    alias: &str,
) -> Result<String, ApiError> {
    let url = format!(
        "{homeserver}/_matrix/client/v3/directory/room/{}",
        urlencoded(alias)
    );

    let resp = http
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::MatrixError(e.to_string()))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(ApiError::MatrixError(format!(
            "Failed to resolve alias {alias}: {text}"
        )));
    }

    let data: Value = resp
        .json()
        .await
        .map_err(|e| ApiError::MatrixError(e.to_string()))?;

    data["room_id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| ApiError::MatrixError("No room_id in alias response".into()))
}

fn urlencoded(s: &str) -> String {
    s.replace('#', "%23")
        .replace(':', "%3A")
}
