//! Picker model translated from Floe Picker.swift, © 2026 René Jiménez, AGPLv3.
use serde::Serialize;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    pub index: usize,
    pub text: String,
    pub title: String,
    pub detail: Option<String>,
}
pub const CHOSEN: i32 = 0;
pub const CANCELLED: i32 = 1;
pub const USAGE: i32 = 64;
pub fn decode(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec())
        .unwrap_or_else(|_| bytes.iter().map(|b| char::from(*b)).collect())
}
pub fn items(input: &str) -> Vec<Item> {
    // Swift Character treats CRLF as a single newline; retain blank-line positions.
    input
        .replace("\r\n", "\n")
        .split([
            '\n', '\r', '\u{000b}', '\u{000c}', '\u{0085}', '\u{2028}', '\u{2029}',
        ])
        .enumerate()
        .filter_map(|(index, line)| {
            if line.chars().all(char::is_whitespace) {
                return None;
            }
            let mut columns = line.split('\t');
            let title = columns.next().unwrap_or_default().to_owned();
            let detail = columns
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("  ");
            Some(Item {
                index,
                text: line.into(),
                title,
                detail: (!detail.is_empty()).then_some(detail),
            })
        })
        .collect()
}
pub fn matches(items: &[Item], query: &str) -> Vec<Item> {
    if query.is_empty() {
        return items.to_vec();
    }
    let mut scored: Vec<_> = items
        .iter()
        .filter_map(|item| {
            let title = crate::fuzzy::score(query, &item.title);
            let detail = item
                .detail
                .as_ref()
                .and_then(|d| crate::fuzzy::score(query, d))
                .map(|s| s - 10);
            title
                .into_iter()
                .chain(detail)
                .max()
                .map(|score| (item, score))
        })
        .collect();
    scored.sort_by(|(a, x), (b, y)| y.cmp(x).then(a.index.cmp(&b.index)));
    scored.into_iter().map(|(i, _)| i.clone()).collect()
}
pub fn output(item: &Item, index: bool) -> String {
    if index {
        item.index.to_string()
    } else {
        item.text.clone()
    }
}
pub fn selection(from: usize, delta: isize, count: usize) -> usize {
    from.saturating_add_signed(delta)
        .min(count.saturating_sub(1))
}
pub fn count_label(shown: usize, total: usize, filtered: bool) -> String {
    if filtered {
        format!("{shown} of {total}")
    } else if total == 1 {
        "1 item".into()
    } else {
        format!("{total} items")
    }
}
