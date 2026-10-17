//! Resident-owned favorites and usage, adapted from Floe Settings.swift / UsageStore.
//! Copyright (Floe) © 2026 René Jiménez. GNU AGPLv3. Native persistence is an atomic file.
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Record {
    pub count: u64,
    pub last_used: u64,
}
fn yes() -> bool {
    true
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Snapshot {
    pub version: u32,
    #[serde(default)]
    pub favorites: Vec<String>,
    #[serde(default)]
    pub records: HashMap<String, Record>,
    #[serde(default)]
    pub queries: Vec<String>,
    #[serde(default = "yes")]
    pub remember_searches: bool,
    #[serde(default)]
    pub aliases: HashMap<String, String>,
    #[serde(default)]
    pub hidden_results: HashMap<String, String>,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            version: 2,
            favorites: vec![],
            records: HashMap::new(),
            queries: vec![],
            remember_searches: true,
            aliases: HashMap::new(),
            hidden_results: HashMap::new(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Operation {
    Read,
    Favorite { id: String, enabled: bool },
    RecordLaunch { id: String, query: String },
    ClearQueries,
    RememberSearches { enabled: bool },
    Alias { id: String, alias: String },
    Hide { id: String, title: String },
    Restore { id: String },
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
fn valid_id(id: &str) -> bool {
    id.starts_with("app:") && id.len() > 4 && id.len() <= 4096 && !id.chars().any(char::is_control)
}
impl Snapshot {
    pub fn context(&self, query: &str, at: u64) -> asaaan_search::SearchContext {
        asaaan_search::SearchContext {
            query: query.into(),
            favorites: self.favorites.clone(),
            aliases: self.aliases.clone(),
            hidden: self.hidden_results.keys().cloned().collect(),
            frecency: self
                .records
                .iter()
                .map(|(id, r)| {
                    (
                        id.clone(),
                        asaaan_search::ranking::frecency(
                            r.count,
                            at.saturating_sub(r.last_used) as f64 / 1000.0,
                        ),
                    )
                })
                .collect(),
        }
    }
    fn validate(&self) -> Result<(), String> {
        if self.version != 2 {
            return Err("Unsupported personalization version; existing data was preserved".into());
        }
        let mut seen = HashSet::new();
        if self
            .favorites
            .iter()
            .any(|id| !valid_id(id) || !seen.insert(id))
            || self.records.keys().any(|id| !valid_id(id))
            || self.aliases.iter().any(|(id, value)| {
                !valid_id(id)
                    || value.is_empty()
                    || value.len() > 1024
                    || value.chars().any(char::is_control)
            })
            || self.hidden_results.iter().any(|(id, title)| {
                !valid_id(id)
                    || title.is_empty()
                    || title.len() > 4096
                    || title.chars().any(|c| c == '\0')
            })
            || self.queries.len() > 50
            || self
                .queries
                .iter()
                .any(|q| q.trim().is_empty() || q.len() > 4096)
        {
            return Err("Invalid personalization data; existing file was preserved".into());
        }
        Ok(())
    }
    pub fn apply(&mut self, operation: &Operation, at: u64) -> Result<(), String> {
        match operation {
            Operation::Read => {}
            Operation::Favorite { id, enabled } => {
                if !valid_id(id) {
                    return Err("Invalid application identity".into());
                }
                if *enabled {
                    if self.hidden_results.contains_key(id) {
                        return Err("Restore the application before adding it to Favorites".into());
                    }
                    if !self.favorites.contains(id) {
                        self.favorites.push(id.clone());
                    }
                } else {
                    self.favorites.retain(|old| old != id);
                }
            }
            Operation::RecordLaunch { id, query } => {
                if !valid_id(id) || query.len() > 4096 {
                    return Err("Invalid launch history".into());
                }
                let record = self.records.entry(id.clone()).or_default();
                record.count = record.count.saturating_add(1);
                record.last_used = at;
                let query = query.trim();
                if self.remember_searches && !query.is_empty() {
                    self.queries.retain(|q| q != query);
                    self.queries.push(query.into());
                    if self.queries.len() > 50 {
                        self.queries.remove(0);
                    }
                }
            }
            Operation::Alias { id, alias } => {
                if !valid_id(id) || alias.len() > 1024 || alias.chars().any(char::is_control) {
                    return Err("Invalid application alias".into());
                }
                if alias.is_empty() {
                    self.aliases.remove(id);
                } else {
                    self.aliases.insert(id.clone(), alias.clone());
                }
            }
            Operation::Hide { id, title } => {
                if !valid_id(id) || title.is_empty() || title.len() > 4096 || title.contains('\0') {
                    return Err("Invalid hidden application".into());
                }
                self.hidden_results.insert(id.clone(), title.clone());
                self.favorites.retain(|old| old != id);
            }
            Operation::Restore { id } => {
                if !valid_id(id) {
                    return Err("Invalid application identity".into());
                }
                self.hidden_results.remove(id);
            }
            Operation::ClearQueries => self.queries.clear(),
            Operation::RememberSearches { enabled } => self.remember_searches = *enabled,
        }
        Ok(())
    }
}
pub struct Store {
    path: PathBuf,
    state: Snapshot,
}
impl Store {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let mut state = match std::fs::File::open(&path) {
            Ok(file) => {
                let mut bytes = Vec::new();
                file.take(2 * 1024 * 1024 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                if bytes.len() > 2 * 1024 * 1024 {
                    return Err(
                        "Personalization file is too large; preserved without changes".into(),
                    );
                }
                serde_json::from_slice::<Snapshot>(&bytes).map_err(|e| {
                    format!("Could not read personalization at line {}, column {}; existing file preserved",e.line(),e.column())
                })?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Snapshot::default(),
            Err(e) => return Err(e.to_string()),
        };
        // CP005 v1 files gain empty customization maps without changing existing values.
        // A read does not rewrite disk; the first successful mutation publishes v2 atomically.
        if state.version == 1 {
            state.version = 2;
        }
        state.validate()?;
        Ok(Self { path, state })
    }
    pub fn snapshot(&self) -> Snapshot {
        self.state.clone()
    }
    pub fn execute(&mut self, operation: &Operation, at: u64) -> Result<Snapshot, String> {
        if matches!(operation, Operation::Read) {
            return Ok(self.snapshot());
        }
        let mut next = self.state.clone();
        next.apply(operation, at)?;
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.as_file()
                .set_permissions(std::fs::Permissions::from_mode(0o600))
                .map_err(|e| e.to_string())?;
        }
        let bytes = serde_json::to_vec(&next).map_err(|e| e.to_string())?;
        if bytes.len() > 2 * 1024 * 1024 {
            return Err("Personalization storage limit reached".into());
        }
        file.write_all(&bytes)
            .and_then(|_| file.as_file().sync_all())
            .map_err(|e| e.to_string())?;
        file.persist(&self.path).map_err(|e| e.to_string())?;
        self.state = next;
        Ok(self.snapshot())
    }
}
