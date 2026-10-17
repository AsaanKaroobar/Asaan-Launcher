// Translated from Floe PickerTests.swift, © 2026 René Jiménez, AGPLv3.
use asaaan_search::picker::*;
fn texts(items: &[Item]) -> Vec<&str> {
    items.iter().map(|i| i.text.as_str()).collect()
}
#[test]
fn each_line_of_the_input_is_an_item() {
    let a = items("alpha\nbeta\ngamma\n");
    assert_eq!(texts(&a), ["alpha", "beta", "gamma"]);
    assert_eq!(a.iter().map(|i| i.index).collect::<Vec<_>>(), [0, 1, 2]);
}
#[test]
fn blank_lines_are_dropped_and_the_others_keep_their_place_in_the_input() {
    let a = items("alpha\n\n   \nbeta\n\t\ngamma");
    assert_eq!(texts(&a), ["alpha", "beta", "gamma"]);
    assert_eq!(a.iter().map(|i| i.index).collect::<Vec<_>>(), [0, 3, 5]);
}
#[test]
fn a_line_keeps_its_own_spaces() {
    assert_eq!(texts(&items("  two words  \n")), ["  two words  "]);
}
#[test]
fn windows_line_endings_split_like_any_other() {
    let a = items("alpha\r\nbeta\r\n");
    assert_eq!(texts(&a), ["alpha", "beta"]);
    assert_eq!(a[1].index, 1);
}
#[test]
fn input_with_nothing_but_blank_lines_has_no_items() {
    for input in ["", "\n", "\n\n  \n"] {
        assert!(items(input).is_empty());
    }
}
#[test]
fn a_tab_splits_a_line_into_a_title_and_what_is_shown_beside_it() {
    let a = items("142\tFix the nav\tana\nplain line\n\tonly a detail");
    assert_eq!(
        a.iter().map(|i| i.title.as_str()).collect::<Vec<_>>(),
        ["142", "plain line", ""]
    );
    assert_eq!(
        a.iter().map(|i| i.detail.as_deref()).collect::<Vec<_>>(),
        [Some("Fix the nav  ana"), None, Some("only a detail")]
    );
    assert_eq!(output(&a[0], false), "142\tFix the nav\tana");
}
#[test]
fn a_query_finds_a_row_by_its_detail_but_puts_a_title_match_first() {
    let a = matches(
        &items("142\tnav rework\nnav\tsomething else\n7\tunrelated"),
        "nav",
    );
    assert_eq!(
        a.iter().map(|i| i.title.as_str()).collect::<Vec<_>>(),
        ["nav", "142"]
    );
}
#[test]
fn an_empty_query_lists_everything_in_input_order() {
    let a = items("zeta\nalpha\nmid");
    assert_eq!(matches(&a, ""), a);
}
#[test]
fn a_query_keeps_only_what_matches_best_first() {
    assert_eq!(
        texts(&matches(&items("my notes\nnotes\nphotos\nkeynote"), "note")),
        ["notes", "my notes", "keynote"]
    );
}
#[test]
fn equal_scores_keep_input_order() {
    assert_eq!(
        texts(&matches(&items("bcd\nacd\nxcd\nccd"), "cd")),
        ["bcd", "acd", "xcd", "ccd"]
    );
}
#[test]
fn a_query_that_matches_nothing_leaves_nothing() {
    assert!(matches(&items("alpha\nbeta"), "xyz").is_empty());
}
#[test]
fn ten_thousand_lines_are_filtered_in_input_order() {
    let a = items(
        &(0..10000)
            .map(|i| format!("item {i}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    let result = matches(&a, "item 99");
    assert_eq!(a.len(), 10000);
    assert_eq!(
        texts(&result[..4]),
        ["item 99", "item 990", "item 991", "item 992"]
    );
    assert!(result.len() < a.len());
}
#[test]
fn a_choice_prints_its_text_or_its_place_in_the_input() {
    let item = items("\n\n\nbeta").remove(0);
    assert_eq!(output(&item, false), "beta");
    assert_eq!(output(&item, true), "3");
}
#[test]
fn the_selection_stays_inside_the_list() {
    for (from, delta, expected) in [
        (0, 1, 1),
        (0, -1, 0),
        (3, 9, 4),
        (4, 1, 4),
        (2, isize::MAX / 2, 4),
        (2, -(isize::MAX / 2), 0),
    ] {
        assert_eq!(selection(from, delta, 5), expected);
    }
}
#[test]
fn the_selection_of_an_empty_list_is_the_first_row() {
    assert_eq!(selection(0, 1, 0), 0);
}
#[test]
fn the_count_says_how_much_of_the_input_still_matches() {
    assert_eq!(count_label(1, 1, false), "1 item");
    assert_eq!(count_label(12, 12, false), "12 items");
    assert_eq!(count_label(3, 12, true), "3 of 12");
}
#[test]
fn the_exit_codes_are_the_ones_scripts_rely_on() {
    assert_eq!((CHOSEN, CANCELLED, USAGE), (0, 1, 64));
}
#[test]
fn encoding_and_unicode_line_boundaries() {
    assert_eq!(decode(&[0xe9]), "é");
    assert_eq!(decode("日本語".as_bytes()), "日本語");
    let a = items("a\r\nb\rc\u{0085}d\u{2028}e\u{2029}f\u{000b}g\u{000c}h");
    assert_eq!(texts(&a), ["a", "b", "c", "d", "e", "f", "g", "h"]);
    assert_eq!(a.last().unwrap().index, 7);
}
