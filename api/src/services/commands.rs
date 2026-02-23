use std::sync::LazyLock;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CommandNode {
    name: String,
    #[serde(default)]
    children: Vec<CommandNode>,
}

static COMMAND_TREE: LazyLock<Vec<CommandNode>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../shared/command-tree.json"))
        .expect("invalid command-tree.json")
});

pub fn validate_command(input: &str) -> Result<(), String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() {
        return Err("empty command".into());
    }

    let mut children = &*COMMAND_TREE;
    let mut matched_depth = 0;

    for token in &tokens {
        if let Some(node) = children.iter().find(|n| n.name == *token) {
            matched_depth += 1;
            children = &node.children;
            if children.is_empty() {
                // Leaf node — remaining tokens are arguments
                break;
            }
        } else {
            break;
        }
    }

    if matched_depth == 0 {
        return Err(format!("unrecognized command: {}", tokens[0]));
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
}
