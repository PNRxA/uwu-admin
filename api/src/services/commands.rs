use std::sync::LazyLock;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CommandArg {
    name: String,
    #[serde(default)]
    required: bool,
    #[serde(rename = "type", default = "default_arg_type")]
    arg_type: String,
}

fn default_arg_type() -> String {
    "string".into()
}

#[derive(Debug, Deserialize)]
struct CommandNode {
    name: String,
    #[serde(default)]
    children: Vec<CommandNode>,
    #[serde(default)]
    args: Vec<CommandArg>,
}

static COMMAND_TREE: LazyLock<Vec<CommandNode>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../shared/command-tree.json"))
        .expect("invalid command-tree.json")
});

pub fn init() {
    LazyLock::force(&COMMAND_TREE);
}

fn validate_arg_format(value: &str, arg_type: &str) -> Result<(), String> {
    match arg_type {
        "user_id" => {
            // Accept both full Matrix IDs (@user:server) and bare usernames
            if value.is_empty() {
                return Err("user_id must not be empty".into());
            }
        }
        "room_id" => {
            if !(value.starts_with('!') || value.starts_with('#')) || !value.contains(':') {
                return Err(format!("invalid room_id format: {value} (expected !room:server or #alias:server)"));
            }
        }
        "event_id" => {
            if !value.starts_with('$') {
                return Err(format!("invalid event_id format: {value} (expected $event_id)"));
            }
        }
        "number" => {
            if value.parse::<i64>().is_err() && value.parse::<u64>().is_err() {
                return Err(format!("invalid number: {value}"));
            }
        }
        "server" => {
            if value.is_empty() || value.chars().any(char::is_whitespace) {
                return Err(format!("invalid server name: {value}"));
            }
        }
        _ => {} // string, path, unknown — accept anything
    }
    Ok(())
}

fn validate_arguments(tokens: &[&str], args: &[CommandArg]) -> Result<(), String> {
    // Separate --flag tokens from positional tokens
    let positional: Vec<&str> = tokens.iter().filter(|t| !t.starts_with("--")).copied().collect();

    // Validate required positional args are present
    let required_args: Vec<&CommandArg> = args.iter().filter(|a| a.required).collect();
    if positional.len() < required_args.len() {
        let missing: Vec<&str> = required_args[positional.len()..]
            .iter()
            .map(|a| a.name.as_str())
            .collect();
        return Err(format!("missing required argument(s): {}", missing.join(", ")));
    }

    // Validate format of each positional arg that has a type definition
    for (i, arg_def) in args.iter().enumerate() {
        if let Some(value) = positional.get(i) {
            validate_arg_format(value, &arg_def.arg_type)?;
        }
    }

    Ok(())
}

pub fn validate_command(input: &str) -> Result<(), String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() {
        return Err("empty command".into());
    }

    let mut children = &*COMMAND_TREE;
    let mut matched_depth = 0;
    let mut leaf_node: Option<&CommandNode> = None;

    for token in &tokens {
        if let Some(node) = children.iter().find(|n| n.name == *token) {
            matched_depth += 1;
            children = &node.children;
            if children.is_empty() {
                // Leaf node — remaining tokens are arguments
                leaf_node = Some(node);
                break;
            }
        } else {
            break;
        }
    }

    if matched_depth == 0 {
        return Err(format!("unrecognized command: {}", tokens[0]));
    }

    // Validate arguments if we reached a leaf node with arg definitions
    if let Some(node) = leaf_node {
        if !node.args.is_empty() {
            let remaining = &tokens[matched_depth..];
            validate_arguments(remaining, &node.args)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_top_level_with_subcommand() {
        assert!(validate_command("server uptime").is_ok());
    }

    #[test]
    fn valid_deeply_nested() {
        assert!(validate_command("rooms alias set !room:example.com localpart").is_ok());
    }

    #[test]
    fn valid_with_args() {
        assert!(validate_command("users deactivate @user:example.com").is_ok());
    }

    #[test]
    fn invalid_top_level() {
        let err = validate_command("nonexistent foo").unwrap_err();
        assert!(err.contains("nonexistent"));
    }

    #[test]
    fn empty_command() {
        assert!(validate_command("").is_err());
        assert!(validate_command("   ").is_err());
    }

    #[test]
    fn valid_query_nested() {
        assert!(validate_command("query globals database-version").is_ok());
    }

    // --- Argument validation ---

    #[test]
    fn user_id_accepts_bare_username() {
        assert!(validate_command("users deactivate notauser").is_ok());
    }

    #[test]
    fn user_id_accepts_full_id() {
        assert!(validate_command("users deactivate @alice:example.com").is_ok());
    }

    #[test]
    fn missing_required_arg() {
        let err = validate_command("users deactivate").unwrap_err();
        assert!(err.contains("missing required"));
    }

    #[test]
    fn optional_arg_missing_ok() {
        // create-user requires username (user_id), password is optional
        assert!(validate_command("users create-user alice").is_ok());
    }

    #[test]
    fn bad_number_format() {
        let err = validate_command("rooms list-rooms notanumber").unwrap_err();
        assert!(err.contains("invalid number"));
    }

    #[test]
    fn valid_room_id_formats() {
        assert!(validate_command("rooms info list-joined-members !room:example.com").is_ok());
        assert!(validate_command("rooms info list-joined-members #alias:example.com").is_ok());
    }

    #[test]
    fn bad_room_id_format() {
        let err = validate_command("rooms info list-joined-members notaroom").unwrap_err();
        assert!(err.contains("invalid room_id"));
    }

    #[test]
    fn bad_event_id_format() {
        let err = validate_command("users redact-event notevent").unwrap_err();
        assert!(err.contains("invalid event_id"));
    }

    #[test]
    fn validate_arg_format_number() {
        assert!(validate_arg_format("42", "number").is_ok());
        assert!(validate_arg_format("-1", "number").is_ok());
        assert!(validate_arg_format("abc", "number").is_err());
    }

    #[test]
    fn validate_arg_format_server() {
        assert!(validate_arg_format("matrix.org", "server").is_ok());
        assert!(validate_arg_format("", "server").is_err());
    }
}
