// List/view/session behavior adapted from Floe ViewState.swift and Session.swift.
// Copyright (Floe) © 2026 René Jiménez. GNU AGPLv3.
use crate::protocol::Node;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Operation {
    Read,
    Start {
        id: String,
        query: String,
    },
    Search {
        token: u64,
        text: String,
    },
    Select {
        token: u64,
        id: i64,
        query: String,
    },
    Run {
        token: u64,
        id: i64,
        revision: u64,
        query: String,
    },
    Back {
        token: u64,
    },
    Stop {
        token: u64,
    },
    Retry {
        token: u64,
    },
    OpenMenu {
        token: u64,
        id: i64,
        revision: u64,
        query: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub extension: String,
    pub keywords: Vec<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub active: Option<View>,
    pub commands: Vec<Entry>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub token: u64,
    pub revision: u64,
    pub command_id: String,
    pub title: String,
    pub screen: Option<i64>,
    pub kind: Option<String>,
    pub query: String,
    pub placeholder: String,
    pub loading: bool,
    pub selected: usize,
    pub rows: Vec<Row>,
    pub actions: Vec<Action>,
    pub empty_title: String,
    pub empty_description: Option<String>,
    pub failure: Option<Failure>,
    pub toast: Option<Toast>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    pub kind: String,
    pub message: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Toast {
    pub id: i64,
    pub style: String,
    pub title: String,
    pub message: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    pub id: i64,
    pub revision: u64,
    pub title: String,
    pub subtitle: Option<String>,
    pub section: Option<String>,
    pub icon: Option<String>,
    pub accessories: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub id: i64,
    pub revision: u64,
    pub title: String,
    pub section: Option<String>,
    pub submenu: bool,
    pub children: Vec<Action>,
    pub shortcut: Option<Value>,
}
pub fn screen(root: Option<&Node>) -> Option<&Node> {
    root?
        .children
        .iter()
        .rev()
        .find(|n| n.kind == "_screen")
        .map(Arc::as_ref)
}
pub fn view(root: Option<&Node>) -> Option<&Node> {
    screen(root)?.content().next().map(Arc::as_ref)
}
pub fn is_list(node: Option<&Node>) -> bool {
    node.is_some_and(|n| n.kind == "List" || n.kind == "Grid")
}
pub fn filters_locally(node: &Node) -> bool {
    if let Some(filtering) = node.props.get("filtering") {
        filtering.as_bool().unwrap_or(true)
    } else {
        !node.handlers.contains("onSearchTextChange")
    }
}
pub fn rows<'a>(node: Option<&'a Node>, query: &str) -> Vec<(&'a Node, Option<&'a str>)> {
    let Some(node) = node.filter(|n| is_list(Some(n))) else {
        return vec![];
    };
    let mut result = vec![];
    for child in node.content() {
        if child.kind.ends_with(".Section") {
            for item in child.content().filter(|n| n.kind.ends_with(".Item")) {
                result.push((item.as_ref(), child.string("title")));
            }
        } else if child.kind.ends_with(".Item") {
            result.push((child.as_ref(), None));
        }
    }
    let tokens: Vec<_> = query
        .to_lowercase()
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    if !filters_locally(node) || tokens.is_empty() {
        return result;
    }
    result
        .into_iter()
        .filter(|(n, _)| {
            let keywords = n
                .props
                .get("keywords")
                .and_then(Value::as_array)
                .filter(|a| a.iter().all(Value::is_string))
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();
            let haystack = format!(
                "{} {} {}",
                n.string("title").unwrap_or(""),
                n.string("subtitle").unwrap_or(""),
                keywords
            )
            .to_lowercase();
            tokens.iter().all(|t| haystack.contains(t))
        })
        .collect()
}
pub fn clamp(index: i64, count: usize) -> usize {
    index.max(0).min(count.saturating_sub(1) as i64) as usize
}
pub fn panel<'a>(node: Option<&'a Node>, selected: Option<&'a Node>) -> Option<&'a Node> {
    let node = node?;
    if !is_list(Some(node)) {
        return node.slot("actions").map(Arc::as_ref);
    }
    selected
        .and_then(|n| n.slot("actions"))
        .or_else(|| {
            node.content()
                .find(|n| n.kind == "EmptyView")
                .and_then(|n| n.slot("actions"))
        })
        .or_else(|| node.slot("actions"))
        .map(Arc::as_ref)
}
pub fn actions(node: Option<&Node>) -> Vec<&Node> {
    let mut found = vec![];
    if let Some(node) = node {
        node.descendants("Action", &mut found);
    }
    found
}
fn event(node: &Node, prop: &str, args: Vec<Value>) -> Value {
    json!({"type":"event","id":node.id,"prop":prop,"args":args})
}

#[derive(Default)]
pub struct Model {
    pub root: Option<Arc<Node>>,
    pub query: String,
    pub selection: usize,
    pub revision: u64,
    pub failure: Option<Failure>,
    pub toast: Option<Toast>,
}
impl Model {
    pub fn render(&mut self, root: Option<Arc<Node>>) {
        let previous = screen(self.root.as_deref()).map(|n| n.id);
        self.root = root;
        self.revision += 1;
        if screen(self.root.as_deref()).map(|n| n.id) != previous {
            self.query.clear();
            self.selection = 0;
        }
        self.selection = self.selection.min(self.rows().len().saturating_sub(1));
    }
    pub fn view(&self) -> Option<&Node> {
        view(self.root.as_deref())
    }
    pub fn rows(&self) -> Vec<(&Node, Option<&str>)> {
        rows(self.view(), &self.query)
    }
    pub fn selected(&self) -> Option<&Node> {
        let rows = self.rows();
        rows.get(self.selection.min(rows.len().saturating_sub(1)))
            .map(|(n, _)| *n)
    }
    pub fn panel(&self) -> Option<&Node> {
        panel(self.view(), self.selected())
    }
    pub fn actions(&self) -> Vec<&Node> {
        actions(self.panel())
    }
    pub fn search(&mut self, text: String) -> Result<Option<Value>, String> {
        if text.len() > 4096 || text.contains('\0') {
            return Err("Extension search exceeds 4096 bytes or contains NUL".into());
        }
        if text == self.query {
            return Ok(None);
        }
        let outgoing = self
            .view()
            .filter(|n| n.handlers.contains("onSearchTextChange"))
            .map(|n| event(n, "onSearchTextChange", vec![json!(text)]));
        self.query = text;
        self.selection = 0;
        self.revision += 1;
        Ok(outgoing)
    }
    pub fn select(&mut self, id: i64, query: &str) -> Result<(), String> {
        if self.query != query {
            return Err("The extension search changed; select the current result.".into());
        }
        let index = self
            .rows()
            .iter()
            .position(|(n, _)| n.id == id)
            .ok_or("The extension row is no longer visible.")?;
        self.selection = index;
        self.revision += 1;
        Ok(())
    }
    pub fn run(&self, id: i64, revision: u64, query: &str) -> Result<Value, String> {
        if self.failure.is_some() {
            return Err("The extension is unavailable. Return to search and reopen it.".into());
        }
        if self.query != query {
            return Err("The extension search changed; reopen its actions.".into());
        }
        let n = self
            .actions()
            .into_iter()
            .find(|n| n.id == id && n.revision == revision)
            .ok_or("The extension action changed; reopen its actions.")?;
        if n.bool("isSubmit") {
            return Err("Form submission is unavailable in this checkpoint.".into());
        }
        if !n.handlers.contains("onAction") {
            return Err("This extension action has no handler.".into());
        }
        Ok(event(n, "onAction", vec![]))
    }
    pub fn open_menu(&self, id: i64, revision: u64, query: &str) -> Result<Option<Value>, String> {
        if self.query != query {
            return Err("The extension search changed.".into());
        }
        let mut found = vec![];
        if let Some(panel) = self.panel() {
            panel.descendants("ActionPanel.Submenu", &mut found);
        }
        let node = found
            .into_iter()
            .find(|n| n.id == id && n.revision == revision)
            .ok_or("The submenu changed.")?;
        Ok(node
            .handlers
            .contains("onOpen")
            .then(|| event(node, "onOpen", vec![])))
    }
    pub fn fail(&mut self, kind: &str, message: impl Into<String>) {
        self.failure = Some(Failure {
            kind: kind.into(),
            message: message.into(),
        });
        self.revision += 1;
    }
    pub fn pong(&mut self) {
        if self
            .failure
            .as_ref()
            .is_some_and(|f| f.kind == "unresponsive")
        {
            self.failure = None;
            self.revision += 1;
        }
    }
    pub fn fields(&mut self, v: &serde_json::Map<String, Value>) {
        match v.get("type").and_then(Value::as_str) {
            Some("clearSearchBar") => {
                self.query.clear();
                self.selection = 0;
                self.revision += 1;
            }
            Some("error") => {
                let message = v
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Unknown extension error")
                    .to_string();
                if v.get("fatal").and_then(Value::as_bool) == Some(true) {
                    self.fail("error", message);
                } else {
                    self.toast = Some(Toast {
                        id: -1,
                        style: "failure".into(),
                        title: "Extension error".into(),
                        message: Some(message),
                    });
                    self.revision += 1;
                }
            }
            Some("toast") => {
                let id = v.get("id").and_then(Value::as_i64).unwrap_or(0);
                if v.get("hidden").and_then(Value::as_bool) == Some(true) {
                    if self.toast.as_ref().is_some_and(|t| t.id == id) {
                        self.toast = None;
                        self.revision += 1;
                    }
                } else {
                    self.toast = Some(Toast {
                        id,
                        style: v
                            .get("style")
                            .and_then(Value::as_str)
                            .unwrap_or("success")
                            .into(),
                        title: v.get("title").and_then(Value::as_str).unwrap_or("").into(),
                        message: v.get("message").and_then(Value::as_str).map(String::from),
                    });
                    self.revision += 1;
                }
            }
            _ => {}
        }
    }
    pub fn snapshot(&self, token: u64, entry: &Entry) -> Result<View, String> {
        let mut budget = Budget(1024 * 1024);
        let node = self.view();
        let empty = node.and_then(|n| n.content().find(|n| n.kind == "EmptyView"));
        let rows = self
            .rows()
            .into_iter()
            .map(|(n, section)| {
                let accessories = n
                    .props
                    .get("accessories")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .flat_map(|a| {
                        ["text", "tag", "date"].into_iter().filter_map(move |k| {
                            a.get(k)
                                .and_then(|v| v.as_str().or_else(|| v.get("value")?.as_str()))
                        })
                    })
                    .map(|s| budget.text(s))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Row {
                    id: n.id,
                    revision: n.revision,
                    title: budget.text(n.string("title").unwrap_or(""))?,
                    subtitle: budget.optional(n.string("subtitle"))?,
                    section: budget.optional(section)?,
                    icon: budget.optional(
                        n.props
                            .get("icon")
                            .and_then(|v| v.as_str().or_else(|| v.get("source")?.as_str())),
                    )?,
                    accessories,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let actions = menu(self.panel(), None, &mut budget)?;
        let failure = self
            .failure
            .as_ref()
            .map(|f| {
                Ok::<_, String>(Failure {
                    kind: budget.text(&f.kind)?,
                    message: budget.text(&f.message)?,
                })
            })
            .transpose()?;
        let toast = self
            .toast
            .as_ref()
            .map(|t| {
                Ok::<_, String>(Toast {
                    id: t.id,
                    style: budget.text(&t.style)?,
                    title: budget.text(&t.title)?,
                    message: budget.optional(t.message.as_deref())?,
                })
            })
            .transpose()?;
        let result = View {
            token,
            revision: self.revision,
            command_id: entry.id.clone(),
            title: budget.text(
                node.and_then(|n| n.string("navigationTitle"))
                    .unwrap_or(&entry.title),
            )?,
            screen: screen(self.root.as_deref()).map(|n| n.id),
            kind: node.map(|n| n.kind.clone()),
            query: self.query.clone(),
            placeholder: budget.text(
                node.and_then(|n| n.string("searchBarPlaceholder"))
                    .unwrap_or("Search…"),
            )?,
            loading: node.is_none_or(|n| n.bool("isLoading")),
            selected: self.selection,
            rows,
            actions,
            empty_title: budget.text(
                empty
                    .and_then(|n| n.string("title"))
                    .unwrap_or("No Results"),
            )?,
            empty_description: budget.optional(empty.and_then(|n| n.string("description")))?,
            failure,
            toast,
        };
        if serde_json::to_vec(&result)
            .map_err(|e| e.to_string())?
            .len()
            > 2 * 1024 * 1024
        {
            return Err("The extension view exceeds the display limit.".into());
        }
        Ok(result)
    }
}
struct Budget(usize);
impl Budget {
    fn text(&mut self, s: &str) -> Result<String, String> {
        if s.len() > 16384 || s.len() > self.0 {
            return Err("The extension view exceeds the display limit.".into());
        }
        self.0 -= s.len();
        Ok(s.into())
    }
    fn optional(&mut self, s: Option<&str>) -> Result<Option<String>, String> {
        s.map(|s| self.text(s)).transpose()
    }
}
fn menu(
    node: Option<&Node>,
    section: Option<&str>,
    budget: &mut Budget,
) -> Result<Vec<Action>, String> {
    let Some(node) = node else { return Ok(vec![]) };
    let mut found = vec![];
    for child in node.content() {
        match child.kind.as_str() {
            "ActionPanel.Section" => found.extend(menu(
                Some(child),
                Some(child.string("title").unwrap_or("")),
                budget,
            )?),
            "Action" | "ActionPanel.Submenu" => {
                let submenu = child.kind == "ActionPanel.Submenu";
                let shortcut = child
                    .props
                    .get("shortcut")
                    .filter(|v| serde_json::to_string(v).is_ok_and(|s| s.len() <= 256))
                    .cloned();
                found.push(Action {
                    id: child.id,
                    revision: child.revision,
                    title: budget.text(child.string("title").unwrap_or("Action"))?,
                    section: budget.optional(section)?,
                    submenu,
                    children: if submenu {
                        menu(Some(child), None, budget)?
                    } else {
                        vec![]
                    },
                    shortcut,
                });
            }
            _ => found.extend(menu(Some(child), section, budget)?),
        }
        if found.len() > 512 {
            return Err("The extension action menu exceeds the display limit.".into());
        }
    }
    Ok(found)
}
