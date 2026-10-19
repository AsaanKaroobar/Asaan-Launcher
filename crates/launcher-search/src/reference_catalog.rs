//! Explicit fixture catalog for CP002; never scans or launches installed applications.
use crate::{Contribution, Item, ItemKind, ResultRow, SearchContext, merge};

pub fn search_reference(query: &str) -> Vec<ResultRow> {
    let make = |id: &str, title: &str, kind, keywords: &[&str]| Item {
        id: id.into(),
        title: title.into(),
        kind,
        keywords: keywords.iter().map(|k| (*k).into()).collect(),
    };
    let items = vec![
        make(
            "reference:hacker-news",
            "Hacker News",
            ItemKind::Command,
            &["news"],
        ),
        make(
            "reference:planets",
            "Browse Planets",
            ItemKind::Command,
            &["hello"],
        ),
        make(
            "reference:settings",
            "Asaaan Launcher Settings",
            ItemKind::Command,
            &[],
        ),
        make(
            "reference:calculator",
            "Calculator",
            ItemKind::Application,
            &[],
        ),
        make("reference:calendar", "Calendar", ItemKind::Application, &[]),
        make("reference:notes", "Notes", ItemKind::Application, &[]),
        make(
            "reference:code",
            "Visual Studio Code",
            ItemKind::Application,
            &["editor"],
        ),
    ];
    let context = SearchContext {
        query: query.into(),
        favorites: vec!["reference:notes".into()],
        aliases: [("reference:hacker-news".into(), "hn".into())].into(),
        frecency: [
            ("reference:calendar".into(), 5.0),
            ("reference:hacker-news".into(), 2.0),
        ]
        .into(),
        ..Default::default()
    };
    merge(
        &[Contribution {
            ranked: items,
            ..Default::default()
        }],
        &context,
        &[],
    )
}
