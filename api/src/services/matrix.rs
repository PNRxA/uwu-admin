use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use sea_orm::DatabaseConnection;
use serde_json::{Value, json};
use uuid::Uuid;

use super::db;
use crate::error::ApiError;

const MATRIX_API_VERSION: &str = "v3";

pub struct CommandResult {
    pub response: String,
    pub command_event_id: String,
    pub response_event_ids: Vec<String>,
}

pub struct RedactionContext {
    pub http: reqwest::Client,
    pub homeserver: String,
    pub access_token: String,
    pub room_id: String,
}

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
        let http = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ApiError::MatrixError(e.to_string()))?;
        let hs = homeserver.trim_end_matches('/');

        let login_url = format!("{hs}/_matrix/client/{MATRIX_API_VERSION}/login");
        let body = json!({
            "type": "m.login.password",
            "identifier": {
                "type": "m.id.user",
                "user": username
            },
            "password": password,
            "device_id": "UWUADMIN",
            "initial_device_display_name": "uwu-admin"
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
        let http = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ApiError::MatrixError(e.to_string()))?;

        let client = MatrixClient {
            http,
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
            "{}/_matrix/client/{MATRIX_API_VERSION}/sync?timeout=0&filter={{\"room\":{{\"timeline\":{{\"limit\":0}}}}}}",
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
            "{}/_matrix/client/{MATRIX_API_VERSION}/sync?timeout=0&filter={{\"room\":{{\"timeline\":{{\"limit\":0}}}}}}",
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

    async fn send_message(&self, body: &str) -> Result<String, ApiError> {
        let txn_id = Uuid::new_v4().to_string();
        let url = format!(
            "{}/_matrix/client/{MATRIX_API_VERSION}/rooms/{}/send/m.room.message/{txn_id}",
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

        let data: Value = resp
            .json()
            .await
            .map_err(|e| ApiError::MatrixError(e.to_string()))?;

        data["event_id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ApiError::MatrixError("No event_id in send response".into()))
    }

    async fn wait_for_response(
        &mut self,
        server_id: i32,
        db: &DatabaseConnection,
    ) -> Result<(String, Vec<String>), ApiError> {
        for _ in 0..3 {
            let mut url = format!(
                "{}/_matrix/client/{MATRIX_API_VERSION}/sync?timeout=10000",
                self.homeserver
            );
            if let Some(since) = &self.since {
                url.push_str("&since=");
                url.push_str(&utf8_percent_encode(since, NON_ALPHANUMERIC).to_string());
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

            // Collect all response messages from the bot in this sync batch
            let mut response_body = String::new();
            let mut event_ids: Vec<String> = Vec::new();

            if let Some(events) = data["rooms"]["join"][&self.room_id]["timeline"]["events"]
                .as_array()
            {
                for event in events {
                    let sender = event["sender"].as_str().unwrap_or_default();
                    let msg_type = event["type"].as_str().unwrap_or_default();
                    if sender != self.user_id && msg_type == "m.room.message" {
                        let event_id = event["event_id"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string();
                        let content = &event["content"];
                        let msgtype = content["msgtype"].as_str().unwrap_or_default();

                        // If the bot sent a file (output too large for text),
                        // download the file content via the media API.
                        if msgtype == "m.file" {
                            if let Some(mxc_url) = content["url"].as_str() {
                                let body = self.download_mxc(mxc_url).await?;
                                if !response_body.is_empty() {
                                    response_body.push_str("<br>");
                                }
                                response_body.push_str(&body);
                                event_ids.push(event_id);
                                continue;
                            }
                            tracing::warn!("m.file event missing url field");
                        }

                        let body = content["body"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string();
                        let formatted = content["formatted_body"]
                            .as_str()
                            .map(|s| s.to_string());

                        if !response_body.is_empty() {
                            response_body.push_str("<br>");
                        }
                        response_body.push_str(&formatted.unwrap_or(body));
                        event_ids.push(event_id);
                    }
                }
            }

            if !event_ids.is_empty() {
                return Ok((response_body, event_ids));
            }
        }

        Err(ApiError::Timeout)
    }

    /// Download a file from a Matrix `mxc://` URL and return its contents as a
    /// UTF-8 string.
    async fn download_mxc(&self, mxc_url: &str) -> Result<String, ApiError> {
        let (server_name, media_id) = parse_mxc_url(mxc_url)?;
        let url = format!(
            "{}/_matrix/media/{MATRIX_API_VERSION}/download/{}/{}",
            self.homeserver, urlencoded(server_name), urlencoded(media_id),
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
            return Err(ApiError::MatrixError(format!(
                "Failed to download media: {text}"
            )));
        }

        const MAX_MEDIA_SIZE: u64 = 10 * 1024 * 1024; // 10 MiB
        if let Some(len) = resp.content_length() {
            if len > MAX_MEDIA_SIZE {
                return Err(ApiError::MatrixError(format!(
                    "Media too large ({len} bytes, max {MAX_MEDIA_SIZE})"
                )));
            }
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ApiError::MatrixError(e.to_string()))?;

        if bytes.len() as u64 > MAX_MEDIA_SIZE {
            return Err(ApiError::MatrixError(format!(
                "Media too large ({} bytes, max {MAX_MEDIA_SIZE})",
                bytes.len()
            )));
        }

        String::from_utf8(bytes.to_vec())
            .map_err(|e| ApiError::MatrixError(format!("Media is not valid UTF-8: {e}")))
    }

    /// Non-blocking sync that advances the `since` token to the present moment,
    /// discarding any stale bot messages so the next `wait_for_response` only
    /// sees messages that arrive after the command is sent.
    async fn drain_pending_messages(
        &mut self,
        server_id: i32,
        db: &DatabaseConnection,
    ) -> Result<(), ApiError> {
        let mut url = format!(
            "{}/_matrix/client/{MATRIX_API_VERSION}/sync?timeout=0&filter={{\"room\":{{\"timeline\":{{\"limit\":0}}}}}}",
            self.homeserver
        );
        if let Some(since) = &self.since {
            url.push_str("&since=");
            url.push_str(&utf8_percent_encode(since, NON_ALPHANUMERIC).to_string());
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
            return Err(ApiError::MatrixError(format!("Drain sync failed: {text}")));
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

        Ok(())
    }

    pub async fn execute_command(
        &mut self,
        command: &str,
        server_id: i32,
        db: &DatabaseConnection,
        redact_messages: bool,
    ) -> Result<CommandResult, ApiError> {
        self.drain_pending_messages(server_id, db).await?;
        let message = format!("!admin {command}");
        let command_event_id = self.send_message(&message).await?;
        match self.wait_for_response(server_id, db).await {
            Ok((response, response_event_ids)) => Ok(CommandResult {
                response,
                command_event_id,
                response_event_ids,
            }),
            Err(e) => {
                if redact_messages {
                    let ctx = self.redaction_context();
                    redact_event(
                        &ctx.http,
                        &ctx.homeserver,
                        &ctx.access_token,
                        &ctx.room_id,
                        &command_event_id,
                    )
                    .await;
                }
                Err(e)
            }
        }
    }

    /// Log out from the homeserver, invalidating the access token and removing
    /// the device.
    pub async fn logout(&self) {
        let url = format!(
            "{}/_matrix/client/{MATRIX_API_VERSION}/logout",
            self.homeserver
        );
        match self.http.post(&url).bearer_auth(&self.access_token).json(&json!({})).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("Logged out from {}", self.homeserver);
            }
            Ok(resp) => {
                let text = resp.text().await.unwrap_or_default();
                tracing::warn!("Logout failed for {}: {text}", self.homeserver);
            }
            Err(e) => {
                tracing::warn!("Logout request failed for {}: {e}", self.homeserver);
            }
        }
    }

    /// Returns cloned values needed for redaction without holding the mutex.
    pub fn redaction_context(&self) -> RedactionContext {
        RedactionContext {
            http: self.http.clone(),
            homeserver: self.homeserver.clone(),
            access_token: self.access_token.clone(),
            room_id: self.room_id.clone(),
        }
    }
}

async fn redact_event(
    http: &reqwest::Client,
    homeserver: &str,
    access_token: &str,
    room_id: &str,
    event_id: &str,
) {
    for attempt in 0..2 {
        let txn_id = Uuid::new_v4().to_string();
        let url = format!(
            "{homeserver}/_matrix/client/{MATRIX_API_VERSION}/rooms/{}/redact/{}/{txn_id}",
            urlencoded(room_id),
            urlencoded(event_id),
        );

        let result = http
            .put(&url)
            .bearer_auth(access_token)
            .json(&json!({}))
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => return,
            Ok(resp) => {
                let text = resp.text().await.unwrap_or_default();
                if attempt == 0 {
                    tracing::warn!("Failed to redact event {event_id}, retrying: {text}");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                } else {
                    tracing::warn!("Failed to redact event {event_id} after retry: {text}");
                }
            }
            Err(e) => {
                if attempt == 0 {
                    tracing::warn!("Failed to redact event {event_id}, retrying: {e}");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                } else {
                    tracing::warn!("Failed to redact event {event_id} after retry: {e}");
                }
            }
        }
    }
}

pub async fn redact_command_events(
    ctx: &RedactionContext,
    command_event_id: &str,
    response_event_ids: &[String],
) {
    let mut set = tokio::task::JoinSet::new();

    let (http, hs, token, room) = (
        ctx.http.clone(),
        ctx.homeserver.clone(),
        ctx.access_token.clone(),
        ctx.room_id.clone(),
    );
    let cmd_eid = command_event_id.to_string();
    set.spawn(async move {
        redact_event(&http, &hs, &token, &room, &cmd_eid).await;
    });

    for eid in response_event_ids {
        let (http, hs, token, room) = (
            ctx.http.clone(),
            ctx.homeserver.clone(),
            ctx.access_token.clone(),
            ctx.room_id.clone(),
        );
        let eid = eid.clone();
        set.spawn(async move {
            redact_event(&http, &hs, &token, &room, &eid).await;
        });
    }

    while set.join_next().await.is_some() {}
}

async fn resolve_alias(
    http: &reqwest::Client,
    homeserver: &str,
    access_token: &str,
    alias: &str,
) -> Result<String, ApiError> {
    let url = format!(
        "{homeserver}/_matrix/client/{MATRIX_API_VERSION}/directory/room/{}",
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

fn parse_mxc_url(mxc_url: &str) -> Result<(&str, &str), ApiError> {
    let path = mxc_url
        .strip_prefix("mxc://")
        .ok_or_else(|| ApiError::MatrixError("Invalid mxc URL: missing mxc:// prefix".into()))?;
    let (server_name, media_id) = path
        .split_once('/')
        .ok_or_else(|| ApiError::MatrixError("Invalid mxc URL: missing media_id".into()))?;
    if server_name.is_empty() || media_id.is_empty() {
        return Err(ApiError::MatrixError("Invalid mxc URL: empty component".into()));
    }
    if server_name.contains("..") || media_id.contains("..") || media_id.contains('/') {
        return Err(ApiError::MatrixError("Invalid mxc URL: path traversal".into()));
    }
    Ok((server_name, media_id))
}

fn urlencoded(s: &str) -> String {
    utf8_percent_encode(s, NON_ALPHANUMERIC).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mxc_url_valid() {
        let (server, media) = parse_mxc_url("mxc://matrix.org/abc123").unwrap();
        assert_eq!(server, "matrix.org");
        assert_eq!(media, "abc123");
    }

    #[test]
    fn parse_mxc_url_rejects_wrong_prefix() {
        assert!(parse_mxc_url("https://matrix.org/abc123").is_err());
    }

    #[test]
    fn parse_mxc_url_rejects_missing_media_id() {
        assert!(parse_mxc_url("mxc://matrix.org").is_err());
    }

    #[test]
    fn parse_mxc_url_rejects_empty_components() {
        assert!(parse_mxc_url("mxc:///abc123").is_err());
        assert!(parse_mxc_url("mxc://matrix.org/").is_err());
    }

    #[test]
    fn parse_mxc_url_rejects_path_traversal() {
        assert!(parse_mxc_url("mxc://matrix.org/../etc/passwd").is_err());
        assert!(parse_mxc_url("mxc://..evil.com/media").is_err());
    }
}
