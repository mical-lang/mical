use mical_cli_config::{Config, Value};
use proptest::{prelude::*, property_test};

fn qp<'a>(config: &'a Config, prefix: &str) -> Vec<(&'a str, Value<'a>)> {
    config.query_prefix(prefix).collect()
}

#[test]
fn empty_config_returns_nothing() {
    let config = Config::from_kv_entries(std::iter::empty());
    assert!(qp(&config, "any").is_empty());
    assert!(qp(&config, "").is_empty());
}

#[test]
fn single_entry_prefix_match() {
    let config = Config::from_kv_entries([("key", Value::String("v"))]);
    assert_eq!(qp(&config, "k"), [("key", Value::String("v"))]);
}

#[test]
fn single_entry_exact_key_as_prefix() {
    let config = Config::from_kv_entries([("key", Value::String("v"))]);
    assert_eq!(qp(&config, "key"), [("key", Value::String("v"))]);
}

#[test]
fn no_match() {
    let config =
        Config::from_kv_entries([("apple", Value::String("a")), ("banana", Value::String("b"))]);
    assert!(qp(&config, "cherry").is_empty());
    assert!(qp(&config, "c").is_empty());
    assert!(qp(&config, "apples").is_empty());
}

#[test]
fn empty_prefix_returns_all_in_first_occurrence_order() {
    let config = Config::from_kv_entries([
        ("b", Value::String("1")),
        ("a", Value::String("2")),
        ("c", Value::String("3")),
    ]);
    assert_eq!(
        qp(&config, ""),
        [("b", Value::String("1")), ("a", Value::String("2")), ("c", Value::String("3")),]
    );
}

#[test]
fn groups_ordered_by_first_occurrence() {
    let config = Config::from_kv_entries([
        ("x", Value::String("x1")),
        ("y", Value::String("y1")),
        ("x", Value::String("x2")),
    ]);
    assert_eq!(
        qp(&config, ""),
        [("x", Value::String("x1")), ("x", Value::String("x2")), ("y", Value::String("y1")),]
    );
}

#[test]
fn hierarchical_keys_with_shared_prefix() {
    let config = Config::from_kv_entries([
        ("app.name", Value::String("MyApp")),
        ("app.version", Value::String("1.0")),
        ("db.host", Value::String("localhost")),
        ("app.name", Value::String("Renamed")),
    ]);
    assert_eq!(
        qp(&config, "app."),
        [
            ("app.name", Value::String("MyApp")),
            ("app.name", Value::String("Renamed")),
            ("app.version", Value::String("1.0")),
        ]
    );
    assert_eq!(qp(&config, "db."), [("db.host", Value::String("localhost"))]);
}

#[test]
fn prefix_longer_than_any_key() {
    let config = Config::from_kv_entries([("a", Value::String("1")), ("ab", Value::String("2"))]);
    assert!(qp(&config, "abc").is_empty());
}

#[test]
fn mixed_value_types() {
    let config = Config::from_kv_entries([
        ("s.flag", Value::Bool(true)),
        ("s.count", Value::Integer("42")),
        ("s.name", Value::String("test")),
    ]);
    assert_eq!(
        qp(&config, "s."),
        [
            ("s.flag", Value::Bool(true)),
            ("s.count", Value::Integer("42")),
            ("s.name", Value::String("test")),
        ]
    );
}

#[test]
fn complex_interleaving_with_prefix_filter() {
    let config = Config::from_kv_entries([
        ("b.x", Value::String("b.x-1")),
        ("a.x", Value::String("a.x-1")),
        ("b.y", Value::String("b.y-1")),
        ("a.y", Value::String("a.y-1")),
        ("b.x", Value::String("b.x-2")),
    ]);
    // prefix "a." → a.x (first at idx 1), a.y (first at idx 3)
    assert_eq!(
        qp(&config, "a."),
        [("a.x", Value::String("a.x-1")), ("a.y", Value::String("a.y-1")),]
    );
    // prefix "b." → b.x group (first at idx 0), b.y (first at idx 2)
    assert_eq!(
        qp(&config, "b."),
        [
            ("b.x", Value::String("b.x-1")),
            ("b.x", Value::String("b.x-2")),
            ("b.y", Value::String("b.y-1")),
        ]
    );
    // prefix "" → all: b.x(0), a.x(1), b.y(2), a.y(3)
    assert_eq!(
        qp(&config, ""),
        [
            ("b.x", Value::String("b.x-1")),
            ("b.x", Value::String("b.x-2")),
            ("a.x", Value::String("a.x-1")),
            ("b.y", Value::String("b.y-1")),
            ("a.y", Value::String("a.y-1")),
        ]
    );
}

#[test]
fn prefix_distinguishes_similar_keys() {
    let config = Config::from_kv_entries([
        ("a", Value::String("1")),
        ("ab", Value::String("2")),
        ("abc", Value::String("3")),
        ("b", Value::String("4")),
    ]);
    assert_eq!(qp(&config, "ab"), [("ab", Value::String("2")), ("abc", Value::String("3")),]);
}

#[test]
fn many_keys_interleaved() {
    // 100 entries across 4 keys, interleaved: first occurrences at idx 0,1,2,3
    let keys = ["ui.theme", "ui.font", "net.host", "net.port"];
    let entries: Vec<(&str, Value)> = (0..100)
        .map(|i| {
            let k = keys[i % keys.len()];
            let v = Value::Integer("1");
            (k, v)
        })
        .collect();
    let config = Config::from_kv_entries(entries);

    // prefix "ui." → ui.theme first (idx 0), ui.font second (idx 1)
    let ui = qp(&config, "ui.");
    let ui_keys: Vec<&str> = ui.iter().map(|(k, _)| *k).collect();
    let first_font = ui_keys.iter().position(|k| *k == "ui.font").unwrap();
    assert!(ui_keys[..first_font].iter().all(|k| *k == "ui.theme"));
    assert_eq!(ui.iter().filter(|(k, _)| *k == "ui.theme").count(), 25);
    assert_eq!(ui.iter().filter(|(k, _)| *k == "ui.font").count(), 25);

    // prefix "" → groups appear in first-occurrence order: ui.theme, ui.font, net.host, net.port
    let all = qp(&config, "");
    let mut seen_order: Vec<&str> = Vec::new();
    for (k, _) in &all {
        if !seen_order.contains(k) {
            seen_order.push(k);
        }
    }
    assert_eq!(seen_order, vec!["ui.theme", "ui.font", "net.host", "net.port"]);
    assert_eq!(all.len(), 100);
}

#[test]
fn many_overrides_of_single_key_among_others() {
    // Simulate a key being overridden many times, mixed with other keys
    let mut entries: Vec<(&str, Value)> = Vec::new();
    entries.push(("server.port", Value::Integer("8080")));
    for i in 0..50 {
        entries.push(("server.log", Value::String(if i % 2 == 0 { "info" } else { "debug" })));
        entries.push(("server.port", Value::Integer("9090")));
    }
    let config = Config::from_kv_entries(entries);

    // server.port first appears before server.log
    let result = qp(&config, "server.");
    let mut seen_order: Vec<&str> = Vec::new();
    for (k, _) in &result {
        if !seen_order.contains(k) {
            seen_order.push(k);
        }
    }
    assert_eq!(seen_order, vec!["server.port", "server.log"]);
    assert_eq!(result.iter().filter(|(k, _)| *k == "server.port").count(), 51);
    assert_eq!(result.iter().filter(|(k, _)| *k == "server.log").count(), 50);
}

fn reference_query_prefix<'a>(
    entries: &'a [(String, String)],
    prefix: &str,
) -> Vec<(&'a str, &'a str)> {
    let mut key_order: Vec<&str> = Vec::new();
    for (k, _) in entries {
        if k.starts_with(prefix) && !key_order.contains(&k.as_str()) {
            key_order.push(k.as_str());
        }
    }
    let mut result = Vec::new();
    for group_key in &key_order {
        for (k, v) in entries {
            if k.as_str() == *group_key {
                result.push((k.as_str(), v.as_str()));
            }
        }
    }
    result
}

#[property_test]
fn matches_linear_reference(
    #[strategy = prop::collection::vec(("[a-d]{1,4}", "[a-z]{0,8}"), 0..100)] entries: Vec<(
        String,
        String,
    )>,
    #[strategy = "[a-d]{0,3}"] prefix: String,
) {
    let config = Config::from_kv_entries(
        entries.iter().map(|(k, v)| (k.as_str(), Value::String(v.as_str()))),
    );
    let actual: Vec<(&str, &str)> = config
        .query_prefix(&prefix)
        .map(|(k, v)| match v {
            Value::String(s) => (k, s),
            _ => unreachable!(),
        })
        .collect();
    let expected = reference_query_prefix(&entries, &prefix);
    prop_assert_eq!(actual, expected);
}

#[property_test]
fn empty_prefix_equals_entries(
    #[strategy = prop::collection::vec(("[a-z]{1,5}", "[a-z]{0,5}"), 0..50)] entries: Vec<(
        String,
        String,
    )>,
) {
    let config = Config::from_kv_entries(
        entries.iter().map(|(k, v)| (k.as_str(), Value::String(v.as_str()))),
    );
    let from_prefix: Vec<_> = config
        .query_prefix("")
        .map(|(k, v)| {
            (
                k,
                match v {
                    Value::String(s) => s,
                    _ => unreachable!(),
                },
            )
        })
        .collect();
    let from_entries: Vec<_> = config
        .entries()
        .map(|(k, v)| {
            (
                k,
                match v {
                    Value::String(s) => s,
                    _ => unreachable!(),
                },
            )
        })
        .collect();
    prop_assert_eq!(from_prefix, from_entries);
}

#[property_test]
fn all_keys_start_with_prefix(
    #[strategy = prop::collection::vec(("[a-d]{1,4}", "[a-z]{0,5}"), 0..20)] entries: Vec<(
        String,
        String,
    )>,
    #[strategy = "[a-d]{0,3}"] prefix: String,
) {
    let config = Config::from_kv_entries(
        entries.iter().map(|(k, v)| (k.as_str(), Value::String(v.as_str()))),
    );
    for (key, _) in config.query_prefix(&prefix) {
        prop_assert!(
            key.starts_with(&prefix),
            "key {:?} does not start with prefix {:?}",
            key,
            prefix
        );
    }
}

#[property_test]
fn same_keys_are_consecutive(
    #[strategy = prop::collection::vec(("[a-c]{1,3}", "[a-z]{0,5}"), 0..100)] entries: Vec<(
        String,
        String,
    )>,
    #[strategy = "[a-c]{0,2}"] prefix: String,
) {
    let config = Config::from_kv_entries(
        entries.iter().map(|(k, v)| (k.as_str(), Value::String(v.as_str()))),
    );
    let keys: Vec<&str> = config.query_prefix(&prefix).map(|(k, _)| k).collect();
    let mut finished: Vec<&str> = Vec::new();
    for window in keys.windows(2) {
        if window[0] != window[1] {
            finished.push(window[0]);
        }
        prop_assert!(
            !finished.contains(&window[1]),
            "key {:?} reappears after gap (finished groups: {:?})",
            window[1],
            finished
        );
    }
}

#[property_test]
fn values_consistent_with_query(
    #[strategy = prop::collection::vec(("[a-c]{1,3}", "[a-z]{0,5}"), 0..15)] entries: Vec<(
        String,
        String,
    )>,
    #[strategy = "[a-c]{0,2}"] prefix: String,
) {
    let config = Config::from_kv_entries(
        entries.iter().map(|(k, v)| (k.as_str(), Value::String(v.as_str()))),
    );
    let prefix_results: Vec<(String, String)> = config
        .query_prefix(&prefix)
        .map(|(k, v)| {
            (
                k.to_owned(),
                match v {
                    Value::String(s) => s.to_owned(),
                    _ => unreachable!(),
                },
            )
        })
        .collect();

    let mut unique_keys: Vec<&str> = Vec::new();
    for (k, _) in &prefix_results {
        if !unique_keys.contains(&k.as_str()) {
            unique_keys.push(k.as_str());
        }
    }

    for key in unique_keys {
        let from_prefix: Vec<&str> =
            prefix_results.iter().filter(|(k, _)| k == key).map(|(_, v)| v.as_str()).collect();
        let from_query: Vec<&str> = config
            .query(key)
            .map(|v| match v {
                Value::String(s) => s,
                _ => unreachable!(),
            })
            .collect();
        prop_assert_eq!(from_prefix, from_query, "values mismatch for key {:?}", key);
    }
}
