use super::*;
use crate::error::ApiError;
use crate::services::matrix;
use crate::services::response;
use serial_test::serial;
use std::sync::Arc;

#[derive(Debug)]
struct TestArg {
    name: String,
    arg_type: String,
    required: bool,
    long: Option<String>,
}

#[derive(Debug)]
struct TestNode {
    name: String,
    children: Vec<TestNode>,
    args: Vec<TestArg>,
}

fn parse_command_tree(value: &[Value]) -> Vec<TestNode> {
    value
        .iter()
        .map(|v| {
            let name = v["name"].as_str().unwrap_or("").to_string();
            let children = v
                .get("children")
                .and_then(|c| c.as_array())
                .map(|arr| parse_command_tree(arr))
                .unwrap_or_default();
            let args = v
                .get("args")
                .and_then(|a| a.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|a| TestArg {
                            name: a["name"].as_str().unwrap_or("").to_string(),
                            arg_type: a["type"].as_str().unwrap_or("string").to_string(),
                            required: a["required"].as_bool().unwrap_or(false),
                            long: a["long"].as_str().map(|s| s.to_string()),
                        })
                        .collect()
                })
                .unwrap_or_default();
            TestNode { name, children, args }
        })
        .collect()
}

fn collect_leaf_commands<'a>(
    nodes: &'a [TestNode],
    prefix: &str,
    out: &mut Vec<(String, Vec<&'a TestArg>)>,
) {
    for node in nodes {
        let path = if prefix.is_empty() {
            node.name.clone()
        } else {
            format!("{prefix} {}", node.name)
        };
        if node.children.is_empty() {
            let args: Vec<&TestArg> = node.args.iter().collect();
            out.push((path, args));
        } else {
            collect_leaf_commands(&node.children, &path, out);
        }
    }
}

#[tokio::test]
#[serial]
async fn execute_all_command_tree_commands() {
    let _ = dotenvy::dotenv();
    let Some((homeserver, username, password, room_id)) = test_server_env() else {
        eprintln!("Skipping execute_all_command_tree_commands: env vars not set");
        return;
    };

    let state = test_state().await;
    let app = test_app_with_state(state.clone());
    let (token, _) = do_setup(&app).await;

    // Add server via HTTP handler
    let body = json!({
        "homeserver": homeserver,
        "username": username,
        "password": password,
        "room_id": room_id,
    });
    let resp = app
        .clone()
        .oneshot(post_json_auth("/api/servers", &body, &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    let server_id = i32::try_from(json["id"].as_i64().unwrap()).unwrap();

    // Get client for direct command execution
    let client_arc = {
        let clients = state.clients.lock().await;
        Arc::clone(clients.get(&server_id).unwrap())
    };
    let mut client = client_arc.lock().await;

    // Collect all event IDs for redaction verification
    let mut event_ids: Vec<(String, Vec<String>)> = Vec::new();

    // Extract server name from room_id (part after ':')
    let server_name = room_id
        .split(':')
        .nth(1)
        .expect("TEST_ROOM_ID must contain ':'");

    // Resolve the room alias to a real !room_id
    let resolve_result = client
        .execute_command(
            &format!("query room-alias resolve-local-alias {room_id}"),
            server_id,
            &state.db,
        )
        .await
        .unwrap();
    event_ids.push((
        resolve_result.command_event_id,
        resolve_result.response_event_ids,
    ));
    let resolve_resp = &resolve_result.response;
    // Extract !room_id from the response (look for a string starting with '!')
    let real_room_id = resolve_resp
        .split_whitespace()
        .chain(resolve_resp.split(|c: char| {
            c == '"' || c == '`' || c == '\n' || c == '<' || c == '>'
        }))
        .find(|s| s.starts_with('!') && s.contains(':'))
        .map(|s| {
            s.trim_end_matches(|c: char| {
                !c.is_alphanumeric() && c != ':' && c != '!' && c != '.' && c != '_' && c != '-'
            })
            .to_string()
        })
        .unwrap_or_else(|| {
            eprintln!(
                "WARNING: Could not resolve room alias, falling back to alias. Response: {resolve_resp}"
            );
            room_id.clone()
        });
    eprintln!("  Resolved room: {room_id} -> {real_room_id}");

    // Create a test user
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let test_username = format!("uwu_test_{ts}");
    let test_user_id = format!("@{test_username}:{server_name}");

    let create_result = client
        .execute_command(
            &format!("users create-user {test_user_id}"),
            server_id,
            &state.db,
        )
        .await
        .expect("Failed to create test user");
    event_ids.push((
        create_result.command_event_id,
        create_result.response_event_ids,
    ));

    // Parse command tree
    let tree_json: Vec<Value> =
        serde_json::from_str(include_str!("../../../shared/command-tree.json"))
            .expect("Failed to parse command-tree.json");
    let tree = parse_command_tree(&tree_json);

    let mut leaves = Vec::new();
    collect_leaf_commands(&tree, "", &mut leaves);

    // Skip lists
    let skip: &[&str] = &[
        // Destructive
        "server shutdown",
        "server restart",
        "server reload-mods",
        "users deactivate-all",
        "users make-user-admin",
        "rooms moderation ban-list-of-rooms",
        // Code-block input required
        "appservices register",
        "debug parse-pdu",
        "debug verify-json",
        "debug get-remote-pdu-list",
        "media delete-list",
        "users force-join-list-of-local-users",
        // Requires confirmation flag
        "users force-join-all-local-users",
        // Times out (long-running operation)
        "query raw compact",
        "debug trim-memory",
    ];

    let mut tested = 0u32;
    let mut skipped = 0u32;
    let mut failures: Vec<(String, String)> = Vec::new();

    for (path, args) in &leaves {
        if skip.contains(&path.as_str()) {
            eprintln!("  SKIP: {path}");
            skipped += 1;
            continue;
        }

        // Build command string — some commands need special arg handling
        let cmd_string = match path.as_str() {
            "token issue" => "token issue --once".to_string(),
            "media delete" => "media delete".to_string(),
            "media delete-past-remote-media" => {
                "media delete-past-remote-media 1d".to_string()
            }
            "query room-alias resolve-local-alias" => {
                format!("query room-alias resolve-local-alias {room_id}")
            }
            "query raw compact" => "query raw compact".to_string(),
            _ => {
                let mut s = path.clone();
                for arg in args {
                    let val = match arg.arg_type.as_str() {
                        "user_id" => test_user_id.clone(),
                        "room_id" => real_room_id.clone(),
                        "server" => server_name.to_string(),
                        "number" => "1".to_string(),
                        "event_id" => "$placeholder:test".to_string(),
                        "path" => "/tmp/test".to_string(),
                        _ => "test".to_string(),
                    };
                    if let Some(long) = &arg.long {
                        s.push_str(&format!(" --{long} {val}"));
                    } else {
                        s.push(' ');
                        s.push_str(&val);
                    }
                }
                s
            }
        };

        eprintln!("  RUN:  {cmd_string}");

        match client
            .execute_command(&cmd_string, server_id, &state.db)
            .await
        {
            Ok(result) => {
                event_ids.push((result.command_event_id, result.response_event_ids));
                let plain = response::strip_html(&result.response);
                let lower = plain.to_lowercase();
                // Only flag CLI arg-parse errors (indicates wrong command tree definition).
                // Responses containing "Command failed with error:" are expected with dummy data.
                if lower.contains("error:")
                    && !lower.contains("command failed with error:")
                {
                    failures.push((path.clone(), plain));
                }
            }
            Err(ApiError::Timeout) => {
                failures.push((path.clone(), "status=504 Gateway Timeout".to_string()));
            }
            Err(e) => {
                failures.push((path.clone(), format!("error: {e}")));
            }
        }

        tested += 1;
    }

    // Cleanup: deactivate the test user
    if let Ok(result) = client
        .execute_command(
            &format!("users deactivate {test_user_id}"),
            server_id,
            &state.db,
        )
        .await
    {
        event_ids.push((result.command_event_id, result.response_event_ids));
    }

    // Cleanup: unban any rooms that may have been banned during the test
    if let Ok(result) = client
        .execute_command(
            &format!("rooms moderation unban-room {real_room_id}"),
            server_id,
            &state.db,
        )
        .await
    {
        event_ids.push((result.command_event_id, result.response_event_ids));
    }

    // Get redaction context and release the client mutex
    let ctx = client.redaction_context();
    drop(client);

    eprintln!("\n=== Command Tree Test Summary ===");
    eprintln!("  Tested:  {tested}");
    eprintln!("  Skipped: {skipped}");
    eprintln!("  Failed:  {}", failures.len());

    if !failures.is_empty() {
        for (path, reason) in &failures {
            eprintln!("  FAIL: {path} — {reason}");
        }
        panic!(
            "{} command(s) failed: {}",
            failures.len(),
            failures
                .iter()
                .map(|(p, r)| format!("{p} ({r})"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Redact all command/response events
    eprintln!("\n=== Redaction Verification ===");
    eprintln!("  Command groups to redact: {}", event_ids.len());

    for (cmd_eid, resp_eids) in &event_ids {
        matrix::redact_command_events(&ctx, cmd_eid, resp_eids).await;
    }

    // Verify all events are actually redacted
    let mut total_events = 0u32;
    let mut redaction_failures = 0u32;
    for (cmd_eid, resp_eids) in &event_ids {
        let mut all = vec![cmd_eid.as_str()];
        all.extend(resp_eids.iter().map(|s| s.as_str()));
        for event_id in all {
            total_events += 1;
            if !is_event_redacted(&ctx.http, &ctx.homeserver, &ctx.access_token, &ctx.room_id, event_id).await {
                redaction_failures += 1;
                eprintln!("  UNREDACTED: {event_id}");
            }
        }
    }

    eprintln!("  Verified: {total_events} events");
    eprintln!("  Redaction failures: {redaction_failures}");
    assert_eq!(
        redaction_failures, 0,
        "{redaction_failures} event(s) were not redacted"
    );
}
