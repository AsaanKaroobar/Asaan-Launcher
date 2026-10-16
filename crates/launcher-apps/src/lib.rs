//! Installed application catalog. No filesystem path or command line is accepted from the UI.
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
pub mod bundles;
#[cfg(target_os = "linux")]
pub mod linux;
pub mod windows;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Desktop(PathBuf),
    Bundle(PathBuf),
    Windows(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub location: PathBuf,
    pub identity: Option<String>,
    pub origin: Option<String>,
    pub keywords: Vec<String>,
    pub target: Target,
}
// Duplicate policy adapted from Floe Catalog.swift, © 2026 René Jiménez, GNU AGPLv3.
pub fn distinct(found: Vec<Application>) -> Vec<Application> {
    let mut groups: HashMap<String, Vec<Application>> = HashMap::new();
    let mut order = vec![];
    for app in found {
        if !groups.contains_key(&app.name) {
            order.push(app.name.clone());
        }
        groups.entry(app.name.clone()).or_default().push(app);
    }
    let mut result = vec![];
    for name in order {
        let group = groups.remove(&name).unwrap();
        if group.len() == 1 {
            result.extend(group);
            continue;
        }
        let mut seen = HashSet::new();
        let mut unique: Vec<_> = group
            .into_iter()
            .filter(|a| {
                seen.insert(a.identity.clone().unwrap_or_else(|| {
                    a.location
                        .canonicalize()
                        .unwrap_or_else(|_| a.location.clone())
                        .to_string_lossy()
                        .into_owned()
                }))
            })
            .collect();
        if unique.len() > 1 {
            let mut parents = HashSet::new();
            let same_folder = unique
                .iter()
                .any(|a| !parents.insert(a.location.parent().map(|p| p.to_path_buf())));
            for a in &mut unique {
                a.origin = Some(
                    if same_folder {
                        a.location.file_stem()
                    } else {
                        a.location.parent().and_then(|p| p.file_name())
                    }
                    .unwrap_or_default()
                    .to_string_lossy()
                    .trim_end_matches(".localized")
                    .to_string(),
                );
            }
        }
        result.extend(unique);
    }
    result
}
pub fn scan() -> Result<Vec<Application>, String> {
    #[cfg(target_os = "linux")]
    {
        linux::scan()
    }
    #[cfg(target_os = "macos")]
    {
        Ok(bundles::scan(&bundles::roots()))
    }
    #[cfg(target_os = "windows")]
    {
        windows::scan()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err("This platform has no application adapter".into())
    }
}
pub fn launch(app: &Application) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        linux::launch(app)
    }
    #[cfg(target_os = "macos")]
    {
        match &app.target {
            Target::Bundle(path) if path.is_dir() => {
                let status = std::process::Command::new("/usr/bin/open")
                    .arg("-a")
                    .arg(path)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map_err(|e| e.to_string())?;
                if status.success() {
                    Ok(())
                } else {
                    Err("macOS could not open the application".into())
                }
            }
            _ => Err("Application is no longer available".into()),
        }
    }
    #[cfg(target_os = "windows")]
    {
        match &app.target {
            Target::Windows(id) => {
                std::process::Command::new("explorer.exe")
                    .arg(format!("shell:AppsFolder\\{id}"))
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .map_err(|e| e.to_string())?;
                Ok(())
            }
            _ => Err("Invalid Windows application target".into()),
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = app;
        Err("Unsupported platform".into())
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Row {
    pub item: asaaan_search::Item,
    pub section: Option<String>,
    pub origin: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct SearchReply {
    pub rows: Vec<Row>,
    pub total: usize,
    pub generation: u64,
    pub warning: Option<String>,
}
struct Cache {
    apps: Vec<Application>,
    generation: u64,
    at: Option<Instant>,
    warning: Option<String>,
}
type Scanner = Arc<dyn Fn() -> Result<Vec<Application>, String> + Send + Sync>;
pub struct Store {
    cache: Mutex<Cache>,
    scanner: Scanner,
    scanning: Mutex<()>,
}
impl Default for Store {
    fn default() -> Self {
        Self::new(Arc::new(scan))
    }
}
impl Store {
    pub fn new(scanner: Scanner) -> Self {
        Self {
            cache: Mutex::new(Cache {
                apps: vec![],
                generation: 0,
                at: None,
                warning: None,
            }),
            scanner,
            scanning: Mutex::new(()),
        }
    }
    pub fn search(&self, query: &str, force: bool) -> Result<SearchReply, String> {
        let stale = self
            .cache
            .lock()
            .unwrap()
            .at
            .is_none_or(|at| at.elapsed() >= Duration::from_secs(5));
        if force || stale {
            let empty = self.cache.lock().unwrap().at.is_none();
            let serial = if force || empty {
                Some(self.scanning.lock().unwrap())
            } else {
                self.scanning.try_lock().ok()
            };
            if let Some(_serial) = serial {
                let still_stale = self
                    .cache
                    .lock()
                    .unwrap()
                    .at
                    .is_none_or(|at| at.elapsed() >= Duration::from_secs(5));
                if force || still_stale {
                    let result = (self.scanner)();
                    let mut cache = self.cache.lock().unwrap();
                    match result {
                        Ok(apps) => {
                            cache.apps = distinct(apps);
                            cache.apps.sort_by(|a, b| {
                                a.name
                                    .to_lowercase()
                                    .cmp(&b.name.to_lowercase())
                                    .then(a.id.cmp(&b.id))
                            });
                            cache.generation += 1;
                            cache.warning = None;
                        }
                        Err(error) => {
                            if cache.at.is_none() {
                                return Err(error);
                            }
                            cache.warning = Some(format!(
                                "Refresh failed; showing the previous catalog: {error}"
                            ));
                        }
                    }
                    cache.at = Some(Instant::now());
                }
            }
        }
        let cache = self.cache.lock().unwrap();
        let items: Vec<_> = cache
            .apps
            .iter()
            .map(|a| asaaan_search::Item {
                id: a.id.clone(),
                title: a.name.clone(),
                kind: asaaan_search::ItemKind::Application,
                keywords: a.keywords.clone(),
            })
            .collect();
        let context = asaaan_search::SearchContext {
            query: query.into(),
            ..Default::default()
        };
        let rows = asaaan_search::merge(
            &[asaaan_search::Contribution {
                ranked: items,
                ..Default::default()
            }],
            &context,
            &[],
        )
        .into_iter()
        .map(|r| Row {
            origin: cache
                .apps
                .iter()
                .find(|a| a.id == r.item.id)
                .and_then(|a| a.origin.clone()),
            item: r.item,
            section: r.section,
        })
        .collect();
        Ok(SearchReply {
            rows,
            total: cache.apps.len(),
            generation: cache.generation,
            warning: cache.warning.clone(),
        })
    }
    pub fn get(&self, id: &str) -> Result<Application, String> {
        self.cache
            .lock()
            .unwrap()
            .apps
            .iter()
            .find(|a| a.id == id)
            .cloned()
            .ok_or_else(|| "Application is no longer in the catalog. Refresh and try again.".into())
    }
}
