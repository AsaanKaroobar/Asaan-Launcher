// Copyright (Floe) © 2026 René Jiménez. GNU AGPLv3.
// Equivalent observable manifest contracts, with explicit per-platform selection.
use asaaan_extensions::manifest::*;
use serde_json::{Value, json};
use std::path::Path;
fn f(v: Value) -> Field {
    field(&v, Platform::Mac).unwrap()
}
fn c(v: Value) -> Vec<Command> {
    commands(
        &serde_json::to_vec(&v).unwrap(),
        Path::new("/tmp/extensions/weather"),
        Platform::Mac,
    )
}
fn weather() -> Value {
    json!({"name":"weather","title":"Weather","icon":"icon.png","preferences":[{"name":"units","type":"dropdown"}],"commands":[
 {"name":"forecast","title":"Forecast","mode":"view","icon":"forecast.png","arguments":[{"name":"city","type":"text"}],"preferences":[{"name":"days"}]},
 {"name":"refresh","mode":"no-view"},{"name":"status","mode":"menu-bar"},{"name":"plain"},{"title":"No name"}]})
}
#[test]
fn reads_every_manifest_key() {
    let x = f(
        json!({"name":"region","title":"Region","description":"Where to search","type":"dropdown","required":true,"placeholder":"Pick one","label":"Unused","default":"eu","data":[{"title":"Europe","value":"eu"},{"value":"us"},{"title":"No value"}]}),
    );
    assert_eq!(x.name, "region");
    assert_eq!(x.title, "Region");
    assert_eq!(x.detail.as_deref(), Some("Where to search"));
    assert_eq!(x.kind, "dropdown");
    assert!(x.required);
    assert_eq!(x.placeholder.as_deref(), Some("Pick one"));
    assert_eq!(x.label.as_deref(), Some("Unused"));
    assert_eq!(x.default, Some(json!("eu")));
    assert_eq!(
        x.options,
        [("Europe".into(), "eu".into()), ("us".into(), "us".into())]
    );
}
#[test]
fn defaults_to_an_optional_text_field() {
    let x = f(json!({"name":"query"}));
    assert_eq!(x.kind, "textfield");
    assert!(!x.required);
    assert_eq!(x.title, "query");
    assert!(x.options.is_empty());
    assert!(x.default.is_none());
    assert_eq!(f(json!({"name":"enabled","type":"checkbox"})).title, "");
}
#[test]
fn scalar_defaults_survive_the_round_trip() {
    for v in [json!(true), json!("eu"), json!(5), json!(1.5)] {
        let x = f(json!({"name":"x","default":v}));
        assert_eq!(x.default, Some(v));
        assert_eq!(
            serde_json::from_value::<Field>(serde_json::to_value(&x).unwrap()).unwrap(),
            x
        );
    }
}
#[test]
fn a_whole_number_default_stays_an_int_and_a_fraction_becomes_a_double() {
    assert_eq!(
        f(json!({"name":"x","default":5})).default.unwrap().as_i64(),
        Some(5)
    );
    assert_eq!(
        f(json!({"name":"x","default":1.5}))
            .default
            .unwrap()
            .as_i64(),
        None
    );
}
#[test]
fn defaults_of_unsupported_types_become_nil() {
    for v in [json!(["structured"]), json!({"nested":1})] {
        assert_eq!(f(json!({"name":"x","default":v})).default, None);
    }
}
#[test]
fn a_default_given_per_platform_takes_the_macs_value() {
    let v = json!({"name":"bin","default":{"macOS":"/usr/local/bin","Windows":"%LOCALAPPDATA%\\JetBrains","Linux":"/usr/bin"}});
    assert_eq!(
        field(&v, Platform::Mac).unwrap().default,
        Some(json!("/usr/local/bin"))
    );
    assert_eq!(
        field(&v, Platform::Windows).unwrap().default,
        Some(json!("%LOCALAPPDATA%\\JetBrains"))
    );
    assert_eq!(
        field(&v, Platform::Linux).unwrap().default,
        Some(json!("/usr/bin"))
    );
    assert_eq!(
        f(json!({"name":"x","type":"checkbox","default":{"macOS":true,"Windows":false}})).default,
        Some(json!(true))
    );
    assert!(
        f(json!({"name":"x","default":{"Windows":"C:\\Tools"}}))
            .default
            .is_none()
    );
}
#[test]
fn arguments_use_their_placeholder_as_title() {
    assert_eq!(f(json!({"name":"text","placeholder":"Text"})).title, "Text");
}
#[test]
fn a_field_needs_a_name() {
    assert!(field(&json!({"title":"Nameless"}), Platform::Mac).is_none());
}
#[test]
fn only_passwords_are_secret() {
    for (kind, secret) in [
        ("password", true),
        ("textfield", false),
        ("checkbox", false),
    ] {
        assert_eq!(f(json!({"name":"x","type":kind})).is_secret(), secret);
    }
}
#[test]
fn lists_view_no_view_and_menu_bar_commands() {
    let commands = c(weather());
    assert_eq!(
        commands.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(),
        ["forecast", "refresh", "status", "plain"]
    );
    assert_eq!(
        commands.iter().map(|x| x.mode.as_str()).collect::<Vec<_>>(),
        ["view", "no-view", "menu-bar", "view"]
    );
}
#[test]
fn commands_carry_extension_and_command_details() {
    let x = &c(weather())[0];
    assert_eq!(x.id, "weather/forecast");
    assert_eq!(x.extension_name, "weather");
    assert_eq!(x.extension_title, "Weather");
    assert_eq!(x.title, "Forecast");
    assert_eq!(x.icon.as_deref(), Some("forecast.png"));
    assert_eq!(
        x.extension_dir.join("assets"),
        Path::new("/tmp/extensions/weather/assets")
    );
    assert_eq!(
        x.arguments
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["city"]
    );
    assert_eq!(x.extension_preferences[0].name, "units");
    assert_eq!(x.command_preferences[0].name, "days");
}
#[test]
fn command_falls_back_to_its_name_and_the_extension_icon() {
    let x = &c(weather())[1];
    assert_eq!(x.title, "refresh");
    assert_eq!(x.icon.as_deref(), Some("icon.png"));
    assert!(x.arguments.is_empty());
}
#[test]
fn extension_title_falls_back_to_its_name() {
    assert_eq!(
        c(json!({"name":"bare","commands":[{"name":"run"}]}))[0].extension_title,
        "bare"
    );
}
#[test]
fn the_owner_is_the_manifests_owner_else_its_author_in_either_shape() {
    for (extra, want) in [
        (json!({"author":"someone"}), Some("someone")),
        (json!({"author":{"name":"someone"}}), Some("someone")),
        (json!({"author":"someone","owner":"team"}), Some("team")),
        (json!({"author":42}), None),
        (json!({}), None),
    ] {
        let mut v = json!({"name":"bare","commands":[{"name":"run"}]});
        v.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        assert_eq!(c(v)[0].owner.as_deref(), want);
    }
}
#[test]
fn a_command_carries_its_own_keywords_and_its_extensions() {
    let x = c(
        json!({"name":"vault","keywords":["secrets"],"commands":[{"name":"items","keywords":["1p","login",7]},{"name":"lock"}]}),
    );
    assert_eq!(x[0].keywords, ["1p", "login", "secrets"]);
    assert_eq!(x[1].keywords, ["secrets"]);
}
#[test]
fn a_command_knows_whether_its_manifest_leaves_it_off_by_default() {
    let x = c(
        json!({"name":"vault","commands":[{"name":"items"},{"name":"renew","disabledByDefault":true},{"name":"lock","disabledByDefault":false}]}),
    );
    assert_eq!(
        x.iter().map(|x| x.disabled_by_default).collect::<Vec<_>>(),
        [false, true, false]
    );
}
#[test]
fn an_extension_whose_platforms_lack_the_mac_has_no_commands() {
    for (platforms, count) in [
        (json!(["Windows"]), 0),
        (json!([]), 0),
        (json!(["macOS", "Windows"]), 2),
        (Value::Null, 2),
    ] {
        assert_eq!(c(json!({"name":"tools","platforms":platforms,"commands":[{"name":"run"},{"name":"extra"}]})).len(),count);
    }
}
#[test]
fn a_command_whose_platforms_lack_the_mac_is_left_out() {
    let v = json!({"name":"tools","commands":[{"name":"run"},{"name":"extra","platforms":["Windows"]}]});
    assert_eq!(
        c(v.clone())
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["run"]
    );
    let bytes = serde_json::to_vec(&v).unwrap();
    assert_eq!(commands(&bytes, Path::new("."), Platform::Windows).len(), 2);
    assert_eq!(commands(&bytes, Path::new("."), Platform::Linux).len(), 1);
}
#[test]
fn manifests_without_a_name_or_commands_yield_nothing() {
    for v in [
        json!({}),
        json!({"name":"no-commands"}),
        json!({"commands":[{"name":"run"}]}),
    ] {
        assert!(c(v).is_empty());
    }
}
#[test]
fn scans_a_folder_of_extensions_in_name_order() {
    let temp = tempfile::tempdir().unwrap();
    for (name, body) in [
        ("zeta", r#"{"name":"zeta","commands":[{"name":"z"}]}"#),
        (
            "alpha",
            r#"{"name":"alpha","commands":[{"name":"a"},{"name":"b","mode":"no-view"}]}"#,
        ),
        ("broken", "not json"),
    ] {
        let path = temp.path().join(name);
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("package.json"), body).unwrap();
    }
    std::fs::create_dir(temp.path().join("no-manifest")).unwrap();
    assert_eq!(
        scan(temp.path(), Platform::Mac)
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>(),
        ["alpha/a", "alpha/b", "zeta/z"]
    );
    assert!(scan(&temp.path().join("missing"), Platform::Mac).is_empty());
}
#[test]
fn parse_interval_matches_units_and_clamps_to_ten_seconds() {
    for (raw, want) in [
        ("90s", Some(90.)),
        ("10m", Some(600.)),
        ("1h", Some(3600.)),
        ("1d", Some(86400.)),
        ("0s", Some(10.)),
        ("-3s", Some(10.)),
        ("NaNs", None),
        ("infs", None),
        ("", None),
        ("10x", None),
    ] {
        assert_eq!(parse_interval(Some(raw)), want);
    }
    assert_eq!(parse_interval(None), None);
}
#[test]
fn unsafe_path_names_and_bad_optional_types_do_not_launch() {
    for name in ["../escape", "a/b", "C:\\Tools", ".", "..", "a\nb"] {
        assert!(c(json!({"name":name,"commands":[{"name":"run"}]})).is_empty());
        assert!(c(json!({"name":"valid","commands":[{"name":name}]})).is_empty());
    }
    assert!(c(json!({"name":"valid","title":42,"commands":[{"name":"run"}]})).is_empty());
    assert!(c(json!({"name":"valid","commands":[{"name":"run","arguments":"bad"}]})).is_empty());
}
