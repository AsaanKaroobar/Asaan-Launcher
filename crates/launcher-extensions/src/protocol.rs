// Adapted from Floe Node.swift and HostMessageDecoder.swift.
// Copyright (Floe) © 2026 René Jiménez. Licensed under GNU AGPLv3.
use serde_json::{Map, Value, json};
use std::{
    collections::{BTreeSet, HashMap},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

static REVISION: AtomicU64 = AtomicU64::new(1);
pub const MAX_LINE: usize = 8 * 1024 * 1024;
const MAX_NODES: usize = 100_000;
const MAX_DEPTH: usize = 128;

#[derive(Debug)]
pub struct Node {
    pub id: i64,
    pub kind: String,
    pub props: Map<String, Value>,
    pub handlers: BTreeSet<String>,
    pub children: Vec<Arc<Node>>,
    pub revision: u64,
    weight: usize,
    height: usize,
}
impl Node {
    pub fn string(&self, key: &str) -> Option<&str> {
        self.props
            .get(key)
            .and_then(|v| v.as_str().or_else(|| v.get("value")?.as_str()))
    }
    pub fn bool(&self, key: &str) -> bool {
        self.props
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }
    pub fn slot(&self, name: &str) -> Option<&Arc<Node>> {
        self.children
            .iter()
            .find(|n| n.kind == "_slot" && n.string("name") == Some(name))?
            .children
            .first()
    }
    pub fn content(&self) -> impl Iterator<Item = &Arc<Node>> {
        self.children.iter().filter(|n| n.kind != "_slot")
    }
    pub fn descendants<'a>(&'a self, kind: &str, out: &mut Vec<&'a Node>) {
        if self.kind == kind {
            out.push(self);
        }
        for child in &self.children {
            child.descendants(kind, out);
        }
    }
    fn hold(node: &Arc<Node>, out: &mut HashMap<i64, Arc<Node>>) {
        out.insert(node.id, node.clone());
        for child in &node.children {
            Self::hold(child, out);
        }
    }
    pub fn json(&self) -> Value {
        json!({"id":self.id,"type":self.kind,"props":self.props,"handlers":self.handlers,"children":self.children.iter().map(|n|n.json()).collect::<Vec<_>>()})
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    LineTooLarge,
    TreeLimit,
}
#[derive(Debug)]
pub enum Message {
    Render(Option<Arc<Node>>),
    UnresolvedRender,
    Fields(Map<String, Value>),
}

enum BuildError {
    Missing,
    Limit,
}
fn build(
    v: &Value,
    kept: &HashMap<i64, Arc<Node>>,
    depth: usize,
    count: &mut usize,
) -> Result<Option<Arc<Node>>, BuildError> {
    let Some(fields) = v.as_object() else {
        return Ok(None);
    };
    let Some(kind) = fields.get("type").and_then(Value::as_str) else {
        return Ok(None);
    };
    *count += 1;
    if depth > MAX_DEPTH || *count > MAX_NODES {
        return Err(BuildError::Limit);
    }
    let mut children = Vec::new();
    for child in fields
        .get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if !child.is_object() {
            continue;
        }
        if let Some(id) = child.get("ref").and_then(Value::as_i64) {
            let node = kept.get(&id).ok_or(BuildError::Missing)?;
            *count = count.saturating_add(node.weight);
            if *count > MAX_NODES || depth + node.height > MAX_DEPTH {
                return Err(BuildError::Limit);
            }
            children.push(node.clone());
        } else if let Some(node) = build(child, kept, depth + 1, count)? {
            children.push(node);
        }
    }
    let weight = 1 + children.iter().map(|n| n.weight).sum::<usize>();
    let height = 1 + children.iter().map(|n| n.height).max().unwrap_or(0);
    Ok(Some(Arc::new(Node {
        weight,
        height,
        id: fields.get("id").and_then(Value::as_i64).unwrap_or(-1),
        kind: kind.into(),
        props: fields
            .get("props")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default(),
        handlers: fields
            .get("handlers")
            .and_then(Value::as_array)
            .filter(|a| a.iter().all(Value::is_string))
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
        children,
        revision: REVISION.fetch_add(1, Ordering::Relaxed),
    })))
}
#[derive(Default)]
pub struct Decoder {
    buffer: Vec<u8>,
    scanned: usize,
    nodes: HashMap<i64, Arc<Node>>,
    sequence: Option<i64>,
    asked: bool,
}
impl Decoder {
    pub fn deliver(
        &mut self,
        data: &[u8],
        mut apply: impl FnMut(Message),
    ) -> Result<(), DecodeError> {
        for message in self.append(data)? {
            apply(message);
        }
        Ok(())
    }
    /// The caller applies every returned message before receiving another pipe chunk.
    pub fn append(&mut self, data: &[u8]) -> Result<Vec<Message>, DecodeError> {
        self.buffer.extend_from_slice(data);
        let mut messages = Vec::new();
        let mut start = 0;
        for i in self.scanned..self.buffer.len() {
            if self.buffer[i] != b'\n' {
                continue;
            }
            if i - start > MAX_LINE {
                return Err(DecodeError::LineTooLarge);
            }
            let parsed = serde_json::from_slice::<Value>(&self.buffer[start..i]).ok();
            if let Some(Value::Object(fields)) = parsed {
                if fields.get("type").and_then(Value::as_str) == Some("render") {
                    if let Some(message) = self.render(&fields)? {
                        messages.push(message);
                    }
                } else {
                    messages.push(Message::Fields(fields));
                }
            }
            start = i + 1;
        }
        if self.buffer.len() - start > MAX_LINE {
            return Err(DecodeError::LineTooLarge);
        }
        self.buffer.drain(..start);
        self.scanned = self.buffer.len();
        Ok(messages)
    }
    fn render(&mut self, fields: &Map<String, Value>) -> Result<Option<Message>, DecodeError> {
        let empty = HashMap::new();
        let kept = if fields.contains_key("base") {
            if self.sequence.is_none()
                || fields.get("base").and_then(Value::as_i64) != self.sequence
                || fields.get("references").and_then(Value::as_i64) != Some(1)
            {
                return Ok(self.missing());
            }
            &self.nodes
        } else {
            &empty
        };
        let tree = match build(fields.get("tree").unwrap_or(&Value::Null), kept, 0, &mut 0) {
            Ok(tree) => tree,
            Err(BuildError::Missing) => return Ok(self.missing()),
            Err(BuildError::Limit) => return Err(DecodeError::TreeLimit),
        };
        let mut nodes = HashMap::new();
        if let Some(node) = &tree {
            Node::hold(node, &mut nodes);
        }
        // Only this render's nodes are retained.
        self.nodes = nodes;
        self.sequence = fields.get("sequence").and_then(Value::as_i64);
        self.asked = false;
        Ok(Some(Message::Render(tree)))
    }
    fn missing(&mut self) -> Option<Message> {
        if self.asked {
            None
        } else {
            self.asked = true;
            Some(Message::UnresolvedRender)
        }
    }
}
