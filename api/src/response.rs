use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParsedResponse {
    Table {
        header: Option<String>,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    List {
        header: Option<String>,
        items: Vec<String>,
    },
    Kv {
        header: Option<String>,
        entries: Vec<KvEntry>,
    },
    Text {
        text: String,
    },
}

#[derive(Debug, Serialize)]
pub struct KvEntry {
    pub key: String,
    pub value: String,
}

static HEADER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^.+?(?:\s*\(\d+\))?:\s*$").unwrap());
static IDENTIFIER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[@!#]\S+:\S+$").unwrap());
static TABLE_LINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^!\S+\s+[A-Z]\w*:").unwrap());
static FIELD_KEY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[A-Z]\w*:\s+").unwrap());
static TABLE_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(!\S+)\s+").unwrap());
static KV_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+?):\s+(.+)$").unwrap());

/// Extract "Key: Value" pairs from a string like "Members: 5  Name: General".
/// Splits on successive `[A-Z]\w*:\s+` boundaries to avoid needing lookahead.
fn extract_fields(s: &str) -> Vec<(String, String)> {
    let matches: Vec<_> = FIELD_KEY_RE.find_iter(s).collect();
    let mut result = Vec::with_capacity(matches.len());
    for (i, m) in matches.iter().enumerate() {
        let key = m.as_str().trim_end().trim_end_matches(':').to_string();
        let value_start = m.end();
        let value_end = matches.get(i + 1).map_or(s.len(), |next| {
            // Walk back from next match start to skip the whitespace separator
            s[..next.start()].trim_end().len()
        });
        let value = s[value_start..value_end].trim().to_string();
        result.push((key, value));
    }
    result
}

/// Strip HTML tags and decode basic entities, converting block-level breaks to newlines.
pub fn strip_html(html: &str) -> String {
    // Replace block-level closing tags with newlines
    let s = html
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n")
        .replace("</li>", "\n");

    // Strip remaining HTML tags
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }

    // Decode common HTML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

pub fn parse_response(html: &str) -> ParsedResponse {
    let text = strip_html(html);
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        let trimmed = text.trim();
        return ParsedResponse::Text {
            text: if trimmed.is_empty() {
                "(empty response)".to_string()
            } else {
                trimmed.to_string()
            },
        };
    }

    // Detect optional header
    let (header, data_lines) = if lines.len() >= 2 && HEADER_RE.is_match(lines[0]) {
        let h = lines[0].trim_end_matches(':').trim_end().to_string();
        (Some(h), &lines[1..])
    } else {
        (None, &lines[..])
    };

    if data_lines.is_empty() {
        return ParsedResponse::Text {
            text: text.trim().to_string(),
        };
    }

    // Table: lines starting with "!roomid  Key: Value"
    if TABLE_LINE_RE.is_match(data_lines[0]) {
        let mut columns = vec!["ID".to_string()];
        if let Some(id_match) = TABLE_ID_RE.find(data_lines[0]) {
            let rest = &data_lines[0][id_match.end()..];
            for (key, _) in extract_fields(rest) {
                columns.push(key);
            }
        }

        let rows: Vec<Vec<String>> = data_lines
            .iter()
            .filter_map(|line| {
                let id_cap = TABLE_ID_RE.captures(line)?;
                let id = id_cap[1].to_string();
                let rest = &line[id_cap[0].len()..];
                let mut values = vec![id];
                for (_, value) in extract_fields(rest) {
                    values.push(value);
                }
                Some(values)
            })
            .collect();

        if !rows.is_empty() {
            return ParsedResponse::Table {
                header,
                columns,
                rows,
            };
        }
    }

    // List: all lines are identifiers
    if data_lines.iter().all(|l| IDENTIFIER_RE.is_match(l)) {
        return ParsedResponse::List {
            header,
            items: data_lines.iter().map(|l| l.to_string()).collect(),
        };
    }

    // Key-value pairs
    let kv_entries: Vec<Option<KvEntry>> = data_lines
        .iter()
        .map(|l| {
            KV_RE.captures(l).and_then(|cap| {
                let key = cap.get(1)?.as_str().trim().to_string();
                let value = cap.get(2)?.as_str().trim().to_string();
                Some(KvEntry { key, value })
            })
        })
        .collect();

    if kv_entries.iter().all(|e| e.is_some()) {
        return ParsedResponse::Kv {
            header,
            entries: kv_entries.into_iter().map(|e| e.unwrap()).collect(),
        };
    }

    // Fallback: plain text
    ParsedResponse::Text {
        text: text.trim().to_string(),
    }
}
