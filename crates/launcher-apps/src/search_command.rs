//! Application-only translation of Floe App/SearchCommand.swift.
//! Copyright (Floe) © 2026 René Jiménez. GNU AGPLv3.
use crate::Store;
use asaaan_search::SearchContext;
use std::collections::HashMap;
pub const LIMIT: usize = 8;
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub query: String,
}
pub fn parse(args: &[String]) -> Result<Option<Options>, String> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        return Ok(None);
    }
    if !args
        .iter()
        .any(|a| a == "--search" || a.starts_with("--search="))
    {
        return Ok(None);
    }
    let mut query = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let value = if arg == "--search" {
            iter.next()
                .filter(|s| !s.starts_with("--"))
                .cloned()
                .ok_or("--search requires a query; use --search '' to browse")?
        } else if let Some(value) = arg.strip_prefix("--search=") {
            value.into()
        } else {
            return Err(format!("Unsupported search option: {arg}"));
        };
        if query.replace(value).is_some() {
            return Err("Repeated --search".into());
        }
    }
    let query = query.ok_or("--search requires a query")?;
    if query.len() > 4096 || query.contains('\0') {
        return Err("Search query exceeds 4096 UTF-8 bytes or contains NUL".into());
    }
    Ok(Some(Options { query }))
}
fn display(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| {
            if c.is_control() {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}
pub fn render(store: &Store, query: &str, context: SearchContext) -> Result<String, String> {
    let aliases: HashMap<_, _> = context.aliases.clone();
    let reply = store.search_with_context(query, true, context)?;
    if let Some(error) = reply.warning {
        return Err(error);
    }
    let mut output = String::new();
    for row in reply.rows.iter().take(LIMIT) {
        if let Some(section) = &row.section {
            output.push_str(&format!("[{}] ", display(section)));
        }
        output.push_str(&display(&row.item.title));
        output.push_str(" (Application)");
        if let Some(alias) = aliases.get(&row.item.id).filter(|s| !s.is_empty()) {
            output.push_str(&format!(" (alias {})", display(alias)));
        }
        output.push('\n');
    }
    Ok(output)
}
