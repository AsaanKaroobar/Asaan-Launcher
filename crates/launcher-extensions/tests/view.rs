// Behavioral equivalents of Floe ViewStateTests.swift and scoped SessionTests.swift.
// Copyright (Floe) © 2026 René Jiménez. GNU AGPLv3.
use asaaan_extensions::{
    protocol::{Decoder, Message, Node},
    view::*,
};
use serde_json::{Value, json};
use std::sync::Arc;
fn tree(value: Value) -> Arc<Node> {
    let mut bytes = serde_json::to_vec(&json!({"type":"render","tree":value})).unwrap();
    bytes.push(b'\n');
    match Decoder::default().append(&bytes).unwrap().pop().unwrap() {
        Message::Render(Some(node)) => node,
        _ => panic!("Missing fixture tree"),
    }
}
fn node(kind: &str, id: i64, children: Vec<Value>) -> Value {
    json!({"type":kind,"id":id,"children":children})
}
fn slot(name: &str, child: Value) -> Value {
    json!({"type":"_slot","props":{"name":name},"children":[child]})
}
fn item(title: &str, id: i64) -> Value {
    json!({"type":"List.Item","id":id,"props":{"title":title},"children":[]})
}
fn action(title: &str, id: i64) -> Value {
    json!({"type":"Action","id":id,"props":{"title":title},"handlers":["onAction"]})
}
fn root(view: Value, screen_id: i64) -> Value {
    node("root", 0, vec![node("_screen", screen_id, vec![view])])
}
fn model(value: Value) -> Model {
    let mut m = Model::default();
    m.render(Some(tree(root(value, 50))));
    m
}
fn list() -> Value {
    node(
        "List",
        60,
        vec![item("Mercury", 1), item("Venus", 2), item("Mars", 3)],
    )
}
fn ids(rows: Vec<(&Node, Option<&str>)>) -> Vec<i64> {
    rows.into_iter().map(|(n, _)| n.id).collect()
}
fn entry() -> Entry {
    Entry {
        id: "command:sample/planets".into(),
        title: "Planets".into(),
        extension: "Sample".into(),
        keywords: vec![],
    }
}
fn panel_fixture() -> Value {
    node(
        "ActionPanel",
        100,
        vec![
            action("Open", 1),
            json!({"type":"ActionPanel.Section","props":{"title":"Copy"},"children":[action("Copy Title",2),action("Copy Link",3)]}),
            node(
                "ActionPanel.Section",
                101,
                vec![action("Untitled Section Action", 4)],
            ),
            json!({"type":"ActionPanel.Submenu","id":5,"props":{"title":"Set Priority"},"handlers":["onOpen"],"children":[action("High",6),action("Low",7)]}),
        ],
    )
}
fn with_actions() -> Value {
    let mut row = item("Row", 10);
    row["children"] = json!([slot("actions", panel_fixture())]);
    node("List", 60, vec![row])
}
#[test]
fn finds_the_view_on_the_top_screen() {
    let root = tree(node(
        "root",
        0,
        vec![
            node("_screen", 10, vec![node("List", 1, vec![])]),
            node(
                "_screen",
                11,
                vec![
                    slot("unused", node("X", 12, vec![])),
                    node("Detail", 2, vec![]),
                ],
            ),
        ],
    ));
    assert_eq!(screen(Some(&root)).unwrap().children.len(), 2);
    assert_eq!(view(Some(&root)).unwrap().id, 2);
    assert!(view(None).is_none());
}
#[test]
fn finds_the_view_on_a_single_screen_snapshot() {
    let root = tree(node(
        "root",
        0,
        vec![node(
            "_screen",
            11,
            vec![
                slot("unused", node("X", 12, vec![])),
                node("Detail", 2, vec![]),
            ],
        )],
    ));
    assert_eq!(screen(Some(&root)).unwrap().children.len(), 2);
    assert_eq!(view(Some(&root)).unwrap().id, 2);
    assert!(!is_list(Some(&root)));
}
#[test]
fn lists_and_grids_are_lists() {
    for (kind, want) in [
        ("List", true),
        ("Grid", true),
        ("Detail", false),
        ("Form", false),
    ] {
        assert_eq!(is_list(Some(&tree(node(kind, 1, vec![])))), want);
    }
    assert!(!is_list(None));
}
#[test]
fn rows_flatten_sections_and_keep_their_titles() {
    let m = model(node(
        "List",
        60,
        vec![
            item("Loose", 1),
            json!({"type":"List.Section","props":{"title":"Rocky"},"children":[item("Mercury",2),item("Venus",3)]}),
            node("List.Section", 70, vec![item("Untitled", 4)]),
            node("EmptyView", 80, vec![]),
        ],
    ));
    assert_eq!(ids(m.rows()), [1, 2, 3, 4]);
    assert_eq!(
        m.rows().iter().map(|(_, s)| *s).collect::<Vec<_>>(),
        [None, Some("Rocky"), Some("Rocky"), None]
    );
}
#[test]
fn grid_items_are_rows_too() {
    let m = model(
        json!({"type":"Grid","children":[{"type":"Grid.Section","props":{"title":"Icons"},"children":[{"type":"Grid.Item","id":9,"props":{"title":"Star"}}]}]}),
    );
    assert_eq!(ids(m.rows()), [9]);
}
#[test]
fn a_view_that_is_not_a_list_has_no_rows() {
    assert!(rows(Some(&tree(node("Detail", 1, vec![]))), "").is_empty());
    assert!(rows(None, "").is_empty());
}
#[test]
fn filters_by_every_word_across_title_subtitle_and_keywords() {
    let n = tree(
        json!({"type":"List","children":[{"type":"List.Item","id":1,"props":{"title":"Mercury","subtitle":"0.39 AU","keywords":["rocky","inner"]}},{"type":"List.Item","id":2,"props":{"title":"Jupiter","subtitle":"5.2 AU","keywords":["gas"]}},{"type":"List.Item","id":3,"props":{"title":"Mars","subtitle":"1.52 AU","keywords":["rocky"]}}]}),
    );
    for (query, want) in [
        ("ROCKY", vec![1, 3]),
        ("rocky m", vec![1, 3]),
        ("rocky inner", vec![1]),
        ("5.2", vec![2]),
        ("pluto", vec![]),
        ("   ", vec![1, 2, 3]),
    ] {
        assert_eq!(ids(rows(Some(&n), query)), want);
    }
}
#[test]
fn a_list_that_handles_search_itself_is_not_filtered_here() {
    let mut v = list();
    v["handlers"] = json!(["onSearchTextChange"]);
    assert_eq!(ids(rows(Some(&tree(v)), "mars")), [1, 2, 3]);
}
#[test]
fn the_filtering_prop_overrides_the_default() {
    for (value, want) in [
        (json!(true), vec![3]),
        (json!(false), vec![1, 2, 3]),
        (json!({"keepSectionOrder":true}), vec![3]),
    ] {
        let mut v = list();
        v["handlers"] = json!(["onSearchTextChange"]);
        v["props"] = json!({"filtering":value});
        assert_eq!(ids(rows(Some(&tree(v)), "mars")), want);
    }
}
#[test]
fn selection_is_clamped_to_the_rows() {
    let mut m = model(list());
    for (index, want) in [(0, 1), (9, 3)] {
        m.selection = index;
        assert_eq!(m.selected().unwrap().id, want);
    }
    assert_eq!(clamp(-3, 3), 0);
    m.render(Some(tree(root(node("List", 2, vec![]), 50))));
    assert!(m.selected().is_none());
}
#[test]
fn clamp_keeps_an_index_in_range() {
    for (index, count, want) in [
        (-5, 3, 0),
        (0, 3, 0),
        (2, 3, 2),
        (7, 3, 2),
        (i64::MAX / 2, 3, 2),
        (4, 0, 0),
    ] {
        assert_eq!(clamp(index, count), want);
    }
}
#[test]
fn a_rows_own_panel_wins_over_the_lists_panel() {
    let mut v = with_actions();
    v["children"]
        .as_array_mut()
        .unwrap()
        .push(slot("actions", node("ActionPanel", 12, vec![])));
    let m = model(v);
    assert_eq!(m.panel().unwrap().id, 100);
    assert_eq!(panel(m.view(), None).unwrap().id, 12);
}
#[test]
fn an_empty_list_uses_its_empty_views_panel_then_its_own() {
    let m = model(node(
        "List",
        1,
        vec![
            slot("actions", node("ActionPanel", 22, vec![])),
            node(
                "EmptyView",
                3,
                vec![slot("actions", node("ActionPanel", 21, vec![]))],
            ),
        ],
    ));
    assert_eq!(m.panel().unwrap().id, 21);
    assert!(panel(Some(&tree(node("List", 1, vec![]))), None).is_none());
    assert!(panel(None, None).is_none());
}
#[test]
fn a_detail_uses_its_own_panel() {
    let m = model(node(
        "Detail",
        1,
        vec![slot("actions", node("ActionPanel", 31, vec![]))],
    ));
    assert_eq!(m.panel().unwrap().id, 31);
}
#[test]
fn actions_are_flattened_in_order_through_sections_and_submenus() {
    assert_eq!(
        actions(Some(&tree(panel_fixture())))
            .iter()
            .map(|n| n.id)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 6, 7]
    );
    assert!(actions(None).is_empty());
}
#[test]
fn the_menu_lists_actions_and_submenus_with_section_titles() {
    let m = model(with_actions());
    let v = m.snapshot(1, &entry()).unwrap();
    assert_eq!(
        v.actions.iter().map(|n| n.id).collect::<Vec<_>>(),
        [1, 2, 3, 4, 5]
    );
    assert_eq!(
        v.actions
            .iter()
            .map(|n| n.section.as_deref())
            .collect::<Vec<_>>(),
        [None, Some("Copy"), Some("Copy"), Some(""), None]
    );
    assert_eq!(
        v.actions.iter().map(|n| n.submenu).collect::<Vec<_>>(),
        [false, false, false, false, true]
    );
}
#[test]
fn an_open_submenu_lists_only_its_own_actions() {
    let m = model(with_actions());
    let v = m.snapshot(1, &entry()).unwrap();
    assert_eq!(
        v.actions[4]
            .children
            .iter()
            .map(|n| n.id)
            .collect::<Vec<_>>(),
        [6, 7]
    );
    assert_eq!(
        m.open_menu(5, v.actions[4].revision, "").unwrap().unwrap()["prop"],
        "onOpen"
    );
    assert!(m.open_menu(5, 0, "").is_err());
}
#[test]
fn a_render_shows_the_view_and_its_rows() {
    let m = model(list());
    assert_eq!(m.view().unwrap().id, 60);
    assert_eq!(ids(m.rows()), [1, 2, 3]);
    assert_eq!(m.selected().unwrap().id, 1);
}
#[test]
fn typing_filters_rows_and_resets_the_selection() {
    let mut m = model(list());
    m.selection = 2;
    assert!(m.search("ma".into()).unwrap().is_none());
    assert_eq!(ids(m.rows()), [3]);
    assert_eq!(m.selection, 0);
}
#[test]
fn typing_is_sent_to_a_list_that_handles_search_itself() {
    let mut v = list();
    v["handlers"] = json!(["onSearchTextChange"]);
    let mut m = model(v);
    let event = m.search("swift".into()).unwrap().unwrap();
    assert_eq!(event["id"], 60);
    assert_eq!(event["args"], json!(["swift"]));
    assert!(m.search("swift".into()).unwrap().is_none());
}
#[test]
fn a_new_screen_clears_the_search_selection_without_telling_the_extension() {
    let mut m = model(list());
    m.search("b".into()).unwrap();
    m.selection = 1;
    m.render(Some(tree(root(node("Detail", 70, vec![]), 51))));
    assert!(m.query.is_empty());
    assert_eq!(m.selection, 0);
    assert!(m.rows().is_empty());
}
#[test]
fn a_rerender_of_the_same_screen_keeps_what_the_user_typed() {
    let mut m = model(list());
    m.search("m".into()).unwrap();
    m.selection = 1;
    m.render(Some(tree(root(list(), 50))));
    assert_eq!(m.query, "m");
    assert_eq!(m.selection, 1);
}
#[test]
fn single_screen_snapshots_keep_identity_and_drop_the_hidden_screen() {
    let mut m = model(list());
    m.search("mer".into()).unwrap();
    m.render(Some(tree(root(
        node("Detail", 70, vec![slot("actions", panel_fixture())]),
        51,
    ))));
    assert_eq!(m.actions().len(), 6);
    m.render(Some(tree(root(list(), 50))));
    assert_eq!(screen(m.root.as_deref()).unwrap().id, 50);
    assert_eq!(ids(m.rows()), [1, 2, 3]);
    assert!(m.query.is_empty());
    assert!(m.actions().is_empty());
}
#[test]
fn a_toast_is_shown_and_hidden_by_its_identifier() {
    let mut m = Model::default();
    m.fields(
        json!({"type":"toast","id":4,"style":"animated","title":"Loading","message":"planets"})
            .as_object()
            .unwrap(),
    );
    assert_eq!(m.toast.as_ref().unwrap().title, "Loading");
    m.fields(
        json!({"type":"toast","id":9,"hidden":true})
            .as_object()
            .unwrap(),
    );
    assert!(m.toast.is_some());
    m.fields(
        json!({"type":"toast","id":4,"hidden":true})
            .as_object()
            .unwrap(),
    );
    assert!(m.toast.is_none());
}
#[test]
fn fatal_and_nonfatal_errors_have_distinct_outcomes() {
    let mut m = model(list());
    m.fields(
        json!({"type":"error","message":"request failed","fatal":false})
            .as_object()
            .unwrap(),
    );
    assert!(m.failure.is_none());
    assert_eq!(m.toast.as_ref().unwrap().style, "failure");
    m.fields(
        json!({"type":"error","message":"render failed","fatal":true})
            .as_object()
            .unwrap(),
    );
    assert_eq!(m.failure.as_ref().unwrap().message, "render failed");
    m.pong();
    assert!(m.failure.is_some());
}
#[test]
fn action_dispatch_revalidates_the_current_query_selection_and_revision() {
    let mut m = model(with_actions());
    let a = m.snapshot(1, &entry()).unwrap().actions[0].clone();
    assert_eq!(m.run(a.id, a.revision, "").unwrap()["prop"], "onAction");
    assert!(m.run(a.id, a.revision + 1, "").is_err());
    assert!(m.run(a.id, a.revision, "stale").is_err());
    m.search("missing".into()).unwrap();
    assert!(m.run(a.id, a.revision, "missing").is_err());
}
#[test]
fn hidden_or_missing_rows_and_forged_operations_are_rejected() {
    let mut m = model(list());
    assert!(m.select(99, "").is_err());
    assert!(m.select(1, "changed").is_err());
    for value in [
        json!({"kind":"run","token":1,"id":1,"revision":1,"query":"","prop":"onAction"}),
        json!({"kind":"start","id":"command:a","query":"","path":"/tmp/evil"}),
        json!({"kind":"event","id":1,"prop":"anything"}),
    ] {
        assert!(serde_json::from_value::<Operation>(value).is_err());
    }
}
#[test]
fn display_limits_fail_instead_of_cloning_unbounded_text() {
    let mut v = list();
    v["children"][0]["props"]["title"] = json!("x".repeat(16385));
    let m = model(v);
    assert!(m.snapshot(1, &entry()).is_err());
}
