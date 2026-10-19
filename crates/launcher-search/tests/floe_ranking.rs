// Behavioral assertions ported from Floe Tests/FloeTests/Search/RankingTests.swift.
// Copyright (Floe) © 2026 René Jiménez. Licensed under GNU AGPLv3.
// Original function names retained for traceability; parameterized cases become loops.
#![allow(non_snake_case)]
use asaaan_search::{fuzzy, ranking::*};
use std::time::{Duration, Instant};

fn app(title: &str) -> Item {
    Item {
        id: title.into(),
        title: title.into(),
        kind: ItemKind::Application,
        keywords: vec![],
    }
}
fn command(title: &str, keywords: &[&str]) -> Item {
    Item {
        kind: ItemKind::Command,
        keywords: keywords.iter().map(|s| (*s).into()).collect(),
        ..app(title)
    }
}
fn items() -> Vec<Item> {
    vec![
        command("Hacker News", &["Hacker News"]),
        command("Browse Planets", &["hello"]),
        app("Calculator"),
        app("Calendar"),
        app("Notes"),
        command("Search Menu Bar Items", &[]),
        command("Floe Settings", &[]),
    ]
}
fn context(query: &str) -> SearchContext {
    SearchContext {
        query: query.into(),
        ..Default::default()
    }
}
fn titles(rows: &[ResultRow]) -> Vec<&str> {
    rows.iter().map(|r| r.item.title.as_str()).collect()
}
fn scored(q: &str, c: &str) -> i32 {
    fuzzy::score(q, c).unwrap()
}

#[test]
fn prefixBeatsWordPrefixBeatsAcronymBeatsSubstringBeatsSubsequence() {
    let values = [
        scored("saf", "Safari"),
        scored("mon", "Activity Monitor"),
        scored("vsc", "Visual Studio Code"),
        scored("tivi", "Activity Monitor"),
        scored("sfr", "Safari"),
    ];
    assert!(values.windows(2).all(|s| s[0] > s[1]));
}
#[test]
fn charactersOutOfOrderOrMissingDoNotMatch() {
    for (q, c) in [("xyz", "Safari"), ("iras", "Safari"), ("safarii", "Safari")] {
        assert_eq!(fuzzy::score(q, c), None);
    }
}
#[test]
fn matchingIgnoresCase() {
    assert_eq!(fuzzy::score("SAF", "safari"), fuzzy::score("saf", "Safari"));
}
#[test]
fn shorterCandidateWinsForTheSamePrefix() {
    assert!(scored("cal", "Calendar") > scored("cal", "Calculator and More Things"));
}
#[test]
fn prefixPenaltyStopsAtTwentyCharacters() {
    assert_eq!(fuzzy::score("a", &format!("a{}", "b".repeat(60))), Some(80));
}

// The independent fixed-tier grid is also replayed through the original Swift oracle fixtures.
#[test]
fn theFourFixedTiersScoreExactlyAsBefore() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/floe-search.json")).unwrap();
    let cases = data["matches"].as_array().unwrap();
    assert!(cases.len() >= 540);
    for case in cases {
        let q = case["query"].as_str().unwrap();
        let c = case["candidate"].as_str().unwrap();
        let expected: Option<fuzzy::Match> =
            serde_json::from_value(case["expected"].clone()).unwrap();
        assert_eq!(fuzzy::match_text(q, c), expected, "{q:?} in {c:?}");
        assert_eq!(fuzzy::score(q, c), expected.as_ref().map(|m| m.score));
    }
}
#[test]
fn everyFixedTierOutranksTheBestScatteredMatch() {
    assert_eq!(scored("ac", "A-B-C"), 54);
    assert!(scored("a", &format!("a{}", "b".repeat(60))) > 75);
    let tiers = [
        scored("mon", "Activity Monitor"),
        scored("vsc", "Visual Studio Code"),
        scored("tivi", "Activity Monitor"),
        scored("ac", "A-B-C"),
    ];
    assert!(tiers.windows(2).all(|pair| pair[0] > pair[1]));
}
#[test]
fn lettersAtWordStartsBeatLettersInsideAWord() {
    assert!(scored("gchr", "Google Chrome") > scored("gchr", "Bigger Machinery"));
}
#[test]
fn lettersInARowBeatTheSameLettersSpreadOut() {
    assert!(scored("abcd", "zabzcd") > scored("abcd", "zazbzczd"));
}
#[test]
fn aMatchThatStartsEarlyBeatsOneThatStartsLate() {
    assert!(scored("sr", "safari") > scored("sr", "xxxxxxsafari"));
}
#[test]
fn aLetterAfterASeparatorCountsAsAWordStart() {
    for c in [
        "color-sync",
        "color_sync",
        "color.sync",
        "color/sync",
        "color sync",
    ] {
        assert!(scored("os", c) > scored("os", "colorsync"));
    }
}
#[test]
fn camelCaseHumpsCountAsWordStarts() {
    let humped = fuzzy::match_text("os", "ColorSync").unwrap();
    assert!(humped.score > scored("os", "Colorsync"));
    assert_eq!(humped.matched, [1, 5]);
    assert_eq!(scored("oé", "ColorÉcole"), humped.score);
}
#[test]
fn theBestAlignmentIsFoundNotTheFirst() {
    let m = fuzzy::match_text("abc", "xaxbxc ab-c").unwrap();
    assert_eq!(m.matched, [7, 8, 10]);
    assert!(m.score > scored("abc", "xaxbxc"));
}
#[test]
fn noScatteredMatchReachesTheSubstringScore() {
    for (q, c) in [
        ("sfr", "Safari"),
        ("gchr", "Google Chrome"),
        ("ac", "A-B-C"),
        ("abdefgh", "a-b-c-d-e-f-g-h x"),
        ("os", "ColorSync"),
        ("aba", "a-b-b-a"),
        ("abab", "ab-ab ab_ab"),
        ("pz", "pizza buzz"),
    ] {
        assert!((1..55).contains(&scored(q, c)));
    }
}
#[test]
fn keywordsStillRejectScatteredLetters() {
    assert_eq!(keyword_score("sfr", "Safari"), None);
    assert_eq!(keyword_score("ac", "A-B-C"), None);
    assert_eq!(keyword_score("far", "Safari"), Some(40));
}
#[test]
fn matchedPositionsAreRightForEachTier() {
    for (q, c, s, p) in [
        ("saf", "Safari", 97, vec![0, 1, 2]),
        ("mon", "Activity Monitor", 75, vec![9, 10, 11]),
        ("vsc", "Visual Studio Code", 70, vec![0, 7, 14]),
        ("tivi", "Activity Monitor", 55, vec![2, 3, 4, 5]),
        ("gchr", "Google Chrome", 50, vec![0, 7, 8, 9]),
    ] {
        let m = fuzzy::match_text(q, c).unwrap();
        assert_eq!(m.score, s);
        assert_eq!(m.matched, p);
        assert_eq!(fuzzy::score(q, c), Some(s));
    }
}
#[test]
fn accentedLettersMatchThemselvesAndCountAsOneCharacter() {
    assert_eq!(scored("é", "Café"), 55);
    assert_eq!(fuzzy::match_text("é", "Café").unwrap().matched, [3]);
    assert_eq!(scored("CAFÉ", "café"), 100);
    assert_eq!(fuzzy::score("e", "Caf\u{e9}"), None);
    let m = fuzzy::match_text("cé", "Crème brûlée").unwrap();
    assert_eq!(m.matched, [0, 10]);
    assert!((1..55).contains(&m.score));
    assert_eq!(fuzzy::match_text("b", "a\r\nb").unwrap().matched, [2]);
}
#[test]
fn uppercaseQueriesScoreLikeLowercaseOnes() {
    assert_eq!(
        fuzzy::score("GCHR", "google chrome"),
        fuzzy::score("gchr", "google chrome")
    );
    assert_eq!(
        fuzzy::match_text("GCHR", "Google Chrome").unwrap().matched,
        [0, 7, 8, 9]
    );
}
#[test]
fn anEmptyQueryIsAPrefixOfEverythingAndMatchesNoCharacters() {
    let m = fuzzy::match_text("", "Safari").unwrap();
    assert_eq!(m.score, 94);
    assert!(m.matched.is_empty());
    assert_eq!(scored("", ""), 100);
}
#[test]
fn aQueryLongerThanTheCandidateDoesNotMatch() {
    for (q, c) in [("safari browser", "Safari"), ("ab", "a"), ("a", "")] {
        assert_eq!(fuzzy::match_text(q, c), None);
    }
}
#[test]
fn menuBarItemsRankAGoodScatteredNameAboveAPoorOne() {
    assert!(
        menu_bar_score("gchr", "Google Chrome", "x")
            > menu_bar_score("gchr", "Bigger Machinery", "x")
    );
}
#[test]
fn tenThousandCandidatesScoreWellWithinAKeystroke() {
    let words = [
        "Google",
        "Chrome",
        "System",
        "Settings",
        "Activity",
        "Monitor",
        "ColorSync",
        "Utility",
        "Terminal",
        "Notes",
    ];
    let candidates: Vec<_> = (0..10000)
        .map(|i| {
            format!(
                "{} {} {} {i}",
                words[i % 10],
                words[(i / 10) % 10],
                words[(i / 100) % 10]
            )
        })
        .collect();
    let start = Instant::now();
    let count = candidates
        .iter()
        .filter(|c| fuzzy::score("gcset", c).is_some())
        .count();
    assert!(count > 0);
    assert!(
        start.elapsed() < Duration::from_millis(500),
        "{:?}",
        start.elapsed()
    );
}
#[test]
fn frecencyWeightFallsOffWithAge() {
    for (age, weight) in [
        (0.0, 4.0),
        (3599.0, 4.0),
        (3600.0, 2.0),
        (86399.0, 2.0),
        (86400.0, 1.0),
        (604799.0, 1.0),
        (604800.0, 0.5),
        (2591999.0, 0.5),
        (2592000.0, 0.25),
        (1e9, 0.25),
    ] {
        assert_eq!(frecency(1, age), weight);
        assert_eq!(frecency(3, age), weight * 3.0);
    }
}
#[test]
fn exactAliasOutranksEverything() {
    assert_eq!(alias_score("hn", "Hacker News", Some("hn")), Some(1000));
    assert_eq!(alias_score("HN", "Hacker News", Some("hn")), Some(1000));
}
#[test]
fn aliasPrefixRanksAtLeastAsWellAsATitlePrefix() {
    assert_eq!(alias_score("ca", "Zebra", Some("cal")), Some(95));
    assert_eq!(alias_score("ca", "Ca", Some("cal")), Some(100));
}
#[test]
fn aliasThatDoesNotMatchFallsBackToTheTitle() {
    for alias in [Some("web"), Some("")] {
        assert_eq!(
            alias_score("saf", "Safari", alias),
            fuzzy::score("saf", "Safari")
        );
    }
    assert_eq!(alias_score("zzz", "Safari", None), None);
}
#[test]
fn browsingListsFavoritesThenSuggestionsThenCommandsThenApplications() {
    let mut c = context("");
    c.favorites = vec!["Notes".into()];
    c.frecency
        .extend([("Calendar".into(), 5.0), ("Hacker News".into(), 2.0)]);
    let rows = browse(&items(), &[], &c);
    assert_eq!(
        titles(&rows),
        [
            "Notes",
            "Calendar",
            "Hacker News",
            "Browse Planets",
            "Search Menu Bar Items",
            "Floe Settings",
            "Calculator"
        ]
    );
    assert_eq!(
        rows.iter()
            .map(|r| r.section.as_deref())
            .collect::<Vec<_>>(),
        [
            Some("Favorites"),
            Some("Suggestions"),
            Some("Suggestions"),
            Some("Commands"),
            Some("Commands"),
            Some("Commands"),
            Some("Applications")
        ]
    );
}
#[test]
fn browsingKeepsFavoritesInTheirSavedOrderAndOutOfSuggestions() {
    let mut c = context("");
    c.favorites = vec![
        "Calendar".into(),
        "Notes".into(),
        "app:/Applications/Gone.app".into(),
    ];
    c.frecency.insert("Calendar".into(), 9.0);
    let rows = browse(&items(), &[], &c);
    assert_eq!(titles(&rows[..2]), ["Calendar", "Notes"]);
    assert!(
        rows.iter()
            .all(|r| r.section.as_deref() != Some("Suggestions"))
    );
    assert_eq!(rows.len(), items().len());
}
#[test]
fn browsingSuggestsAtMostFiveItems() {
    let all = items();
    let mut c = context("");
    for item in &all {
        c.frecency.insert(item.id.clone(), 1.0);
    }
    assert_eq!(
        browse(&all, &[], &c)
            .iter()
            .filter(|r| r.section.as_deref() == Some("Suggestions"))
            .count(),
        5
    );
}
#[test]
fn equallyUsedSuggestionsKeepTheirOrder() {
    let all = items();
    let mut c = context("");
    for item in &all {
        c.frecency.insert(item.id.clone(), 1.0);
    }
    let r = browse(&all, &[], &c);
    assert_eq!(
        titles(&r[..5]),
        all[..5]
            .iter()
            .map(|i| i.title.as_str())
            .collect::<Vec<_>>()
    );
    let many: Vec<_> = (1..=80).map(|i| app(&format!("App {i}"))).collect();
    c.frecency.clear();
    for (i, item) in many.iter().enumerate() {
        c.frecency
            .insert(item.id.clone(), if i == 40 { 2.0 } else { 1.0 });
    }
    let r = browse(&many, &[], &c);
    assert_eq!(
        titles(&r[..5]),
        ["App 41", "App 1", "App 2", "App 3", "App 4"]
    );
}
#[test]
fn browsingListsCommandsBeforeApplicationsInTheirOwnOrder() {
    assert_eq!(
        titles(&browse(&items(), &[], &context(""))),
        [
            "Hacker News",
            "Browse Planets",
            "Search Menu Bar Items",
            "Floe Settings",
            "Calculator",
            "Calendar",
            "Notes"
        ]
    );
}
#[test]
fn searchingOrdersByMatchThenUsage() {
    let mut c = context("cal");
    let rows = search(&items(), &c, 40);
    assert_eq!(titles(&rows), ["Calendar", "Calculator"]);
    assert!(rows.iter().all(|r| r.section.is_none()));
    c.frecency.insert("Calculator".into(), 6.0);
    assert_eq!(
        titles(&search(&items(), &c, 40)),
        ["Calculator", "Calendar"]
    );
}
#[test]
fn usageBoostIsCappedSoAnExactAliasStillWins() {
    let mut c = context("c");
    c.aliases.insert("Hacker News".into(), "c".into());
    c.frecency.insert("Calculator".into(), 1e6);
    assert_eq!(search(&items(), &c, 40)[0].item.title, "Hacker News");
}
#[test]
fn favoritesGetASmallBoostInSearch() {
    let mut c = context("cal");
    c.favorites = vec!["Calculator".into()];
    assert_eq!(
        titles(&search(&items(), &c, 40)),
        ["Calculator", "Calendar"]
    );
}
#[test]
fn searchRespectsTheLimit() {
    let many: Vec<_> = (0..60).map(|i| app(&format!("App {i}"))).collect();
    assert_eq!(search(&many, &context("app"), 40).len(), 40);
    assert_eq!(search(&many, &context("app"), 3).len(), 3);
    assert!(search(&many, &context("app"), 0).is_empty());
}
#[test]
fn menuBarItemsMatchOnNameAndLessStronglyOnOwner() {
    let name = menu_bar_score("tail", "Tailscale", "Tailscale").unwrap();
    let owner = menu_bar_score("codex", "Usage 86%", "CodexBar").unwrap();
    assert!(name > owner);
    assert_eq!(owner, scored("codex", "CodexBar") - 10);
    assert_eq!(menu_bar_score("zzz", "Tailscale", "Tailscale"), None);
}
#[test]
fn aCommandAnswersToItsExtensionsNameBehindATitleThatMatches() {
    let all = vec![
        command("Show Installed", &["brew"]),
        command("Upgrade", &["brew"]),
        app("Brewfile"),
        app("Notes"),
    ];
    assert_eq!(
        titles(&search(&all, &context("brew"), 40)),
        ["Brewfile", "Show Installed", "Upgrade"]
    );
    assert_eq!(titles(&search(&all, &context("bw"), 40)), ["Brewfile"]);
}
#[test]
fn aCommandAnswersToItsManifestKeywordsBehindATitleThatMatches() {
    let all = vec![
        command("Search Items", &["vault", "1p", "login"]),
        app("Login Items"),
        app("Notes"),
    ];
    assert_eq!(titles(&search(&all, &context("1p"), 40)), ["Search Items"]);
    assert_eq!(
        titles(&search(&all, &context("login"), 40)),
        ["Login Items", "Search Items"]
    );
}
