// Adapted from Floe Manifest.swift. Copyright (Floe) © 2026 René Jiménez. GNU AGPLv3.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub enum Platform {
    Mac,
    Windows,
    Linux,
}
impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Mac
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::Mac => "macOS",
            Self::Windows => "Windows",
            Self::Linux => "Linux",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Field {
    pub name: String,
    pub title: String,
    pub detail: Option<String>,
    pub kind: String,
    pub required: bool,
    pub placeholder: Option<String>,
    pub label: Option<String>,
    pub options: Vec<(String, String)>,
    pub default: Option<Value>,
}
impl Field {
    pub fn is_secret(&self) -> bool {
        self.kind == "password"
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Command {
    pub id: String,
    pub extension_dir: PathBuf,
    pub extension_name: String,
    pub extension_title: String,
    pub name: String,
    pub title: String,
    pub mode: String,
    pub interval: Option<f64>,
    pub icon: Option<String>,
    pub arguments: Vec<Field>,
    pub extension_preferences: Vec<Field>,
    pub command_preferences: Vec<Field>,
    pub keywords: Vec<String>,
    pub disabled_by_default: bool,
    pub owner: Option<String>,
}
fn text(v: &Value, key: &str) -> Option<String> {
    v.get(key)?.as_str().map(String::from)
}
fn valid_optional(v: &Value, key: &str, check: fn(&Value) -> bool) -> bool {
    v.get(key).is_none_or(|v| v.is_null() || check(v))
}
fn strings(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(String::from)
        .collect()
}
fn fields(v: &Value, key: &str, platform: Platform) -> Vec<Field> {
    v.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|v| field(v, platform))
        .collect()
}
pub fn field(v: &Value, platform: Platform) -> Option<Field> {
    let name = text(v, "name")?;
    for key in ["title", "description", "type", "placeholder", "label"] {
        if !valid_optional(v, key, Value::is_string) {
            return None;
        }
    }
    if !valid_optional(v, "required", Value::is_boolean)
        || !valid_optional(v, "data", Value::is_array)
    {
        return None;
    }
    let kind = text(v, "type").unwrap_or("textfield".into());
    let placeholder = text(v, "placeholder");
    let title = text(v, "title").unwrap_or_else(|| {
        if kind == "checkbox" {
            String::new()
        } else {
            placeholder.clone().unwrap_or(name.clone())
        }
    });
    let scalar = |v: &Value| v.is_string() || v.is_boolean() || v.is_number();
    let default = v
        .get("default")
        .and_then(|v| {
            if scalar(v) {
                Some(v)
            } else {
                v.get(platform.key()).filter(|v| scalar(v))
            }
        })
        .cloned();
    let options = v
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|v| {
            if !valid_optional(v, "title", Value::is_string) {
                return None;
            }
            let value = text(v, "value")?;
            Some((text(v, "title").unwrap_or(value.clone()), value))
        })
        .collect();
    Some(Field {
        name,
        title,
        detail: text(v, "description"),
        kind,
        required: v.get("required").and_then(Value::as_bool).unwrap_or(false),
        placeholder,
        label: text(v, "label"),
        options,
        default,
    })
}
pub fn parse_interval(raw: Option<&str>) -> Option<f64> {
    let raw = raw?;
    let unit = raw.chars().last()?;
    let factor = match unit {
        's' => 1.,
        'm' => 60.,
        'h' => 3600.,
        'd' => 86400.,
        _ => return None,
    };
    let value: f64 = raw[..raw.len() - unit.len_utf8()].parse().ok()?;
    value.is_finite().then_some((value * factor).max(10.))
}
fn supports(v: &Value, platform: Platform) -> bool {
    v.get("platforms")
        .is_none_or(|x| x.is_null() || strings(v, "platforms").iter().any(|s| s == platform.key()))
}
/// Slugs are also used by the runtime as output paths. Reject separators/drive syntax on every OS.
pub fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 200
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
pub fn commands(bytes: &[u8], folder: &Path, platform: Platform) -> Vec<Command> {
    let Ok(v) = serde_json::from_slice::<Value>(bytes) else {
        return vec![];
    };
    let Some(name) = text(&v, "name").filter(|s| safe_name(s)) else {
        return vec![];
    };
    for key in ["title", "owner", "icon"] {
        if !valid_optional(&v, key, Value::is_string) {
            return vec![];
        }
    }
    for key in ["keywords", "platforms", "preferences"] {
        if !valid_optional(&v, key, Value::is_array) {
            return vec![];
        }
    }
    if !supports(&v, platform) {
        return vec![];
    }
    v.get("commands")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|c| {
            let command = text(c, "name").filter(|s| safe_name(s))?;
            for key in ["title", "mode", "interval", "icon"] {
                if !valid_optional(c, key, Value::is_string) {
                    return None;
                }
            }
            for key in ["keywords", "platforms", "preferences", "arguments"] {
                if !valid_optional(c, key, Value::is_array) {
                    return None;
                }
            }
            if !valid_optional(c, "disabledByDefault", Value::is_boolean) || !supports(c, platform)
            {
                return None;
            }
            let mode = text(c, "mode").unwrap_or("view".into());
            if !["view", "no-view", "menu-bar"].contains(&mode.as_str()) {
                return None;
            }
            let mut keywords = strings(c, "keywords");
            keywords.extend(strings(&v, "keywords"));
            Some(Command {
                id: format!("{name}/{command}"),
                extension_dir: folder.into(),
                extension_name: name.clone(),
                extension_title: text(&v, "title").unwrap_or(name.clone()),
                title: text(c, "title").unwrap_or(command.clone()),
                name: command,
                mode,
                interval: parse_interval(c.get("interval").and_then(Value::as_str)),
                icon: text(c, "icon").or_else(|| text(&v, "icon")),
                arguments: fields(c, "arguments", platform),
                extension_preferences: fields(&v, "preferences", platform),
                command_preferences: fields(c, "preferences", platform),
                keywords,
                disabled_by_default: c
                    .get("disabledByDefault")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                owner: text(&v, "owner")
                    .or_else(|| text(&v, "author"))
                    .or_else(|| v.get("author").and_then(|a| text(a, "name"))),
            })
        })
        .collect()
}
pub fn scan(root: &Path, platform: Platform) -> Vec<Command> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return vec![];
    };
    let mut folders: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    folders.sort();
    folders
        .iter()
        .flat_map(|p| {
            std::fs::read(p.join("package.json"))
                .ok()
                .map(|bytes| commands(&bytes, p, platform))
                .unwrap_or_default()
        })
        .collect()
}
