// Copyright (Floe) © 2026 René Jiménez. GNU AGPLv3.
// Behavioral equivalents of Floe's HostMessageDecoderTests, NodeTests, RenderReferenceTests.
use asaaan_extensions::protocol::{Decoder, MAX_LINE, Message, Node};
use serde_json::{Value, json};
use std::sync::Arc;
fn line(v: Value) -> Vec<u8> {
    let mut b = serde_json::to_vec(&v).unwrap();
    b.push(b'\n');
    b
}
fn node(kind: &str, id: i64, children: Vec<Value>) -> Value {
    json!({"type":kind,"id":id,"children":children})
}
fn item(title: &str, id: i64) -> Value {
    json!({"type":"List.Item","id":id,"props":{"title":title},"children":[
        {"type":"_slot","id":id+1,"props":{"name":"actions"},"children":[{"type":"ActionPanel","id":id+2,"children":[{"type":"Action","id":id+3,"props":{"title":format!("Open {title}")}}]}]},
        {"type":"_slot","id":id+4,"props":{"name":"detail"},"children":[{"type":"Detail","id":id+5,"props":{"markdown":format!("# {title}")}}]}
    ]})
}
fn root(children: Vec<Value>) -> Value {
    node(
        "root",
        0,
        vec![node("_screen", 2, vec![node("List", 3, children)])],
    )
}
fn whole(seq: i64, tree: Value) -> Value {
    json!({"type":"render","sequence":seq,"tree":tree})
}
fn part(seq: i64, base: i64, version: i64, tree: Value) -> Value {
    json!({"type":"render","sequence":seq,"base":base,"references":version,"tree":tree})
}
fn tree(d: &mut Decoder, v: Value) -> Arc<Node> {
    match d.append(&line(v)).unwrap().pop().unwrap() {
        Message::Render(Some(n)) => n,
        m => panic!("{m:?}"),
    }
}
fn unresolved(d: &mut Decoder, v: Value) {
    assert!(matches!(
        &d.append(&line(v)).unwrap()[..],
        [Message::UnresolvedRender]
    ));
}
fn list(n: &Node) -> &Node {
    &n.children[0].children[0]
}
fn planets() -> Vec<Value> {
    vec![item("Mercury", 10), item("Venus", 20), item("Earth", 30)]
}

#[test]
fn a_message_is_reassembled_at_every_split_position() {
    let bytes = line(json!({"type":"hud","title":"wör👍d","message":"done"}));
    for split in 0..bytes.len() {
        let mut d = Decoder::default();
        assert!(d.append(&bytes[..split]).unwrap().is_empty());
        match &d.append(&bytes[split..]).unwrap()[..] {
            [Message::Fields(v)] => assert_eq!(v["title"], "wör👍d"),
            m => panic!("{m:?}"),
        }
    }
}
#[test]
fn a_split_inside_a_multibyte_character_is_reassembled() {
    let bytes = line(json!({"type":"hud","title":"wör👍d"}));
    let split = bytes.iter().position(|b| *b == 0xf0).unwrap() + 2;
    let mut d = Decoder::default();
    assert!(d.append(&bytes[..split]).unwrap().is_empty());
    assert!(
        matches!(&d.append(&bytes[split..]).unwrap()[..],[Message::Fields(v)] if v["title"]=="wör👍d")
    );
}
#[test]
fn lines_complete_in_order_across_and_within_chunks() {
    let mut d = Decoder::default();
    let mut bytes = line(whole(1, node("Detail", 7, vec![])));
    bytes.extend(line(json!({"type":"close"})));
    let messages = d.append(&bytes).unwrap();
    assert!(
        matches!(&messages[..],[Message::Render(Some(n)),Message::Fields(v)] if n.id==7&&v["type"]=="close")
    );
    assert!(
        matches!(&d.append(&line(json!({"type":"hud"}))).unwrap()[..],[Message::Fields(v)] if v["type"]=="hud")
    );
}
#[test]
fn unreadable_lines_are_skipped() {
    let mut d = Decoder::default();
    assert!(
        d.append(b"not json\n\n[1,2]\n\"text\"\n42\nnull\n")
            .unwrap()
            .is_empty()
    );
    assert_eq!(d.append(&line(json!({"type":"close"}))).unwrap().len(), 1);
}
#[test]
fn a_trailing_partial_line_waits_for_its_newline() {
    let bytes = line(json!({"type":"hud"}));
    let mut d = Decoder::default();
    assert!(d.append(&bytes[..bytes.len() - 1]).unwrap().is_empty());
    assert_eq!(d.append(b"\n").unwrap().len(), 1);
}
#[test]
fn many_partial_appends_keep_the_partial_line_intact() {
    let bytes = line(json!({"type":"hud","title":"Copied","message":"done"}));
    for size in [1, 3, 17, bytes.len() - 1, bytes.len()] {
        let mut d = Decoder::default();
        let all: Vec<_> = bytes
            .chunks(size)
            .flat_map(|c| d.append(c).unwrap())
            .collect();
        assert!(matches!(&all[..],[Message::Fields(v)] if v["title"]=="Copied"));
    }
}
#[test]
fn a_render_with_an_unreadable_tree_decodes_as_an_empty_root() {
    let mut d = Decoder::default();
    for v in [
        json!({"type":"render","tree":"nope"}),
        json!({"type":"render"}),
    ] {
        assert!(matches!(
            &d.append(&line(v)).unwrap()[..],
            [Message::Render(None)]
        ));
    }
    assert_eq!(
        tree(&mut d, whole(1, node("Detail", 7, vec![]))).kind,
        "Detail"
    );
}
#[test]
fn decoders_do_not_share_state() {
    let mut a = Decoder::default();
    let mut b = Decoder::default();
    assert!(a.append(b"partial ").unwrap().is_empty());
    assert_eq!(b.append(&line(json!({"type":"close"}))).unwrap().len(), 1);
    assert!(a.append(b"half\n").unwrap().is_empty());
    assert_eq!(a.append(&line(json!({"type":"hud"}))).unwrap().len(), 1);
}
#[test]
fn a_reference_is_replaced_by_the_node_kept_under_its_id() {
    let mut d = Decoder::default();
    let first = tree(&mut d, whole(1, root(planets())));
    let second = tree(
        &mut d,
        part(
            2,
            1,
            1,
            root(vec![
                json!({"ref":10}),
                json!({"ref":20}),
                json!({"ref":30}),
            ]),
        ),
    );
    assert_eq!(
        list(&second)
            .children
            .iter()
            .map(|n| n.string("title"))
            .collect::<Vec<_>>(),
        [Some("Mercury"), Some("Venus"), Some("Earth")]
    );
    assert_eq!(
        list(&second).children[1].slot("actions").unwrap().children[0].string("title"),
        Some("Open Venus")
    );
    assert_eq!(
        list(&first).children[2].json(),
        list(&second).children[2].json()
    );
}
#[test]
fn a_tree_built_from_references_equals_the_same_tree_sent_whole() {
    let mut d = Decoder::default();
    tree(&mut d, whole(1, root(planets())));
    let renamed = json!({"type":"List.Item","id":20,"props":{"title":"The morning star"},"children":[{"ref":21},{"ref":24}]});
    let built = tree(
        &mut d,
        part(
            2,
            1,
            1,
            root(vec![json!({"ref":30}), renamed, json!({"ref":10})]),
        ),
    );
    let mut changed = item("Venus", 20);
    changed["props"] = json!({"title":"The morning star"});
    let fresh = tree(
        &mut Decoder::default(),
        whole(
            1,
            root(vec![item("Earth", 30), changed, item("Mercury", 10)]),
        ),
    );
    assert_eq!(built.json(), fresh.json());
    let again = tree(&mut d, part(3, 2, 1, root(vec![json!({"ref":20})])));
    assert_eq!(
        list(&again).children[0].string("title"),
        Some("The morning star")
    );
    assert_eq!(
        list(&again).children[0]
            .slot("detail")
            .unwrap()
            .string("markdown"),
        Some("# Venus")
    );
}
#[test]
fn a_subtree_named_under_another_parent_is_found_by_its_id() {
    let mut d = Decoder::default();
    tree(&mut d, whole(1, root(planets())));
    let section = node(
        "List.Section",
        40,
        vec![json!({"ref":20}), json!({"ref":32})],
    );
    let moved = tree(
        &mut d,
        part(2, 1, 1, root(vec![json!({"ref":10}), section])),
    );
    let mut found = vec![];
    moved.descendants("List.Section", &mut found);
    assert_eq!(
        found[0].children.iter().map(|n| n.id).collect::<Vec<_>>(),
        [20, 32]
    );
    assert_eq!(found[0].children[1].kind, "ActionPanel");
}
#[test]
fn a_kept_node_keeps_its_revision_and_every_node_sent_anew_gets_another() {
    let mut d = Decoder::default();
    let first = tree(&mut d, whole(1, root(planets())));
    let renamed = json!({"type":"List.Item","id":20,"children":[{"ref":21},{"ref":24}]});
    let second = tree(
        &mut d,
        part(
            2,
            1,
            1,
            root(vec![json!({"ref":10}), renamed, json!({"ref":30})]),
        ),
    );
    assert!(Arc::ptr_eq(
        &list(&first).children[0],
        &list(&second).children[0]
    ));
    assert_ne!(
        list(&first).children[1].revision,
        list(&second).children[1].revision
    );
    assert_eq!(
        list(&first).children[1].children[0].revision,
        list(&second).children[1].children[0].revision
    );
    let other = tree(&mut Decoder::default(), whole(1, root(planets())));
    assert_ne!(
        list(&first).children[0].revision,
        list(&other).children[0].revision
    );
}
#[test]
fn a_reference_to_nothing_builds_no_tree_and_is_reported_once() {
    let mut d = Decoder::default();
    tree(&mut d, whole(1, root(planets())));
    unresolved(
        &mut d,
        part(2, 1, 1, root(vec![json!({"ref":10}), json!({"ref":99})])),
    );
    assert!(
        d.append(&line(part(3, 2, 1, root(vec![json!({"ref":10})]))))
            .unwrap()
            .is_empty()
    );
    tree(&mut d, whole(4, root(planets())));
    tree(&mut d, part(5, 4, 1, root(vec![json!({"ref":30})])));
    unresolved(&mut d, part(6, 5, 1, root(vec![json!({"ref":99})])));
}
#[test]
fn a_render_written_against_one_the_app_missed_is_not_read() {
    let mut d = Decoder::default();
    tree(&mut d, whole(1, root(planets())));
    unresolved(&mut d, part(3, 2, 1, root(vec![json!({"ref":10})])));
    unresolved(&mut Decoder::default(), part(1, 0, 1, root(vec![])));
}
#[test]
fn references_of_another_version_are_not_read() {
    for version in [Some(2), None] {
        let mut d = Decoder::default();
        tree(&mut d, whole(1, root(planets())));
        let mut msg = part(2, 1, 2, root(vec![json!({"ref":10})]));
        if version.is_none() {
            msg.as_object_mut().unwrap().remove("references");
        }
        unresolved(&mut d, msg);
    }
}
#[test]
fn whole_renders_need_no_sequence_and_may_name_nothing() {
    let mut d = Decoder::default();
    tree(&mut d, json!({"type":"render","tree":root(planets())}));
    assert_eq!(
        list(&tree(
            &mut d,
            json!({"type":"render","tree":root(vec![item("Mercury",10)])})
        ))
        .children
        .len(),
        1
    );
    unresolved(
        &mut d,
        json!({"type":"render","tree":root(vec![json!({"ref":10})])}),
    );
}
#[test]
fn a_screen_that_was_replaced_is_no_longer_held() {
    let mut d = Decoder::default();
    tree(&mut d, whole(1, root(planets())));
    tree(
        &mut d,
        part(
            2,
            1,
            1,
            node(
                "root",
                0,
                vec![node("_screen", 60, vec![node("Detail", 61, vec![])])],
            ),
        ),
    );
    unresolved(
        &mut d,
        part(3, 2, 1, node("root", 0, vec![json!({"ref":2})])),
    );
}
#[test]
fn the_decoder_holds_the_last_render_between_chunks_and_misses_what_it_could_not_read() {
    let mut d = Decoder::default();
    tree(&mut d, whole(1, root(planets())));
    tree(&mut d, part(2, 1, 1, root(vec![json!({"ref":10})])));
    assert!(
        d.append(b"{\"type\":\"render\",\"sequence\":3,\"tree\":\n")
            .unwrap()
            .is_empty()
    );
    unresolved(&mut d, part(4, 3, 1, root(vec![json!({"ref":10})])));
}
#[test]
fn parses_type_props_handlers_and_children() {
    let n = tree(
        &mut Decoder::default(),
        json!({"type":"render","tree":{"type":"List","id":7,"props":{"isLoading":true},"handlers":["onSearchTextChange"],"children":[{"type":"List.Item","id":8}]}}),
    );
    assert_eq!(n.id, 7);
    assert_eq!(n.kind, "List");
    assert!(n.bool("isLoading"));
    assert!(n.handlers.contains("onSearchTextChange"));
    assert_eq!(n.children[0].id, 8);
}
#[test]
fn rejects_anything_without_a_type() {
    for v in [Value::Null, json!("text"), json!(3), json!({"id":1})] {
        assert!(matches!(
            &Decoder::default()
                .append(&line(json!({"type":"render","tree":v})))
                .unwrap()[..],
            [Message::Render(None)]
        ));
    }
}
#[test]
fn missing_fields_get_defaults() {
    let n = tree(
        &mut Decoder::default(),
        json!({"type":"render","tree":{"type":"Detail"}}),
    );
    assert_eq!(n.id, -1);
    assert!(n.props.is_empty());
    assert!(n.children.is_empty());
    assert!(n.handlers.is_empty());
    assert!(!n.bool("isLoading"));
}
#[test]
fn children_that_are_not_nodes_are_dropped() {
    let n = tree(
        &mut Decoder::default(),
        json!({"type":"render","tree":{"type":"List","children":[{"type":"List.Item"},"stray text",{"id":2}]}}),
    );
    assert_eq!(n.children.len(), 1);
    assert_eq!(n.children[0].kind, "List.Item");
}
#[test]
fn string_reads_plain_text_or_a_value_object() {
    let n = tree(
        &mut Decoder::default(),
        json!({"type":"render","tree":{"type":"List.Item","props":{"title":"Plain","subtitle":{"value":"Wrapped","tooltip":"Hint"},"count":3}}}),
    );
    assert_eq!(n.string("title"), Some("Plain"));
    assert_eq!(n.string("subtitle"), Some("Wrapped"));
    assert_eq!(n.string("count"), None);
    assert_eq!(n.string("missing"), None);
}
#[test]
fn slot_returns_the_named_element_and_content_leaves_slots_out() {
    let mut v = item("One", 10);
    v["children"]
        .as_array_mut()
        .unwrap()
        .push(node("Extra", 22, vec![]));
    let n = tree(&mut Decoder::default(), whole(1, v));
    assert_eq!(n.slot("actions").unwrap().id, 12);
    assert_eq!(n.slot("detail").unwrap().id, 15);
    assert!(n.slot("metadata").is_none());
    assert_eq!(n.content().map(|n| n.id).collect::<Vec<_>>(), [22]);
}
#[test]
fn descendants_finds_nested_nodes_in_order_including_itself() {
    let n = tree(
        &mut Decoder::default(),
        whole(
            1,
            node(
                "ActionPanel",
                0,
                vec![
                    node("Action", 1, vec![]),
                    node("ActionPanel.Section", 2, vec![node("Action", 3, vec![])]),
                ],
            ),
        ),
    );
    let mut a = vec![];
    n.descendants("Action", &mut a);
    assert_eq!(a.iter().map(|n| n.id).collect::<Vec<_>>(), [1, 3]);
    let mut a = vec![];
    n.descendants("ActionPanel", &mut a);
    assert_eq!(a.len(), 1);
    let mut a = vec![];
    n.descendants("Missing", &mut a);
    assert!(a.is_empty());
}
#[test]
fn over_limit_partial_line_is_rejected() {
    assert!(
        Decoder::default()
            .append(&vec![b'x'; MAX_LINE + 1])
            .is_err()
    );
}

#[test]
fn a_suspended_application_holds_back_the_next_chunk() {
    let (entered, seen) = std::sync::mpsc::channel();
    let (open, gate) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let mut d = Decoder::default();
        let mut applied = vec![];
        d.deliver(&line(json!({"type":"hud"})), |_| {
            applied.push("hud");
            entered.send(()).unwrap();
            gate.recv().unwrap();
        })
        .unwrap();
        d.deliver(&line(json!({"type":"close"})), |_| applied.push("close"))
            .unwrap();
        applied
    });
    seen.recv().unwrap();
    assert!(!worker.is_finished());
    open.send(()).unwrap();
    assert_eq!(worker.join().unwrap(), ["hud", "close"]);
}

#[test]
fn compact_reference_expansion_cannot_bypass_the_tree_budget() {
    use asaaan_extensions::protocol::DecodeError;
    let mut decoder = Decoder::default();
    tree(&mut decoder, whole(1, node("List.Item", 1, vec![])));
    let mut limited = false;
    for sequence in 2..10 {
        let children = vec![json!({"ref":sequence-1}); 8];
        let bytes = line(part(
            sequence,
            sequence - 1,
            1,
            node("List", sequence, children),
        ));
        assert!(bytes.len() < 1024);
        match decoder.append(&bytes) {
            Err(DecodeError::TreeLimit) => {
                limited = true;
                break;
            }
            Ok(_) => {}
            Err(other) => panic!("{other:?}"),
        }
    }
    assert!(
        limited,
        "compact references must count their expanded subtrees"
    );
}
