use mical_config::{Config, Value};
use proptest::{prelude::*, property_test};

fn q<'a>(config: &'a Config, key: &str) -> Vec<Value<'a>> {
    config.query(key).collect()
}

#[test]
fn empty_config_returns_nothing() {
    let config = Config::from_kv_entries(std::iter::empty());
    assert!(q(&config, "any").is_empty());
    assert!(q(&config, "").is_empty());
}

#[test]
fn single_entry_exact_match() {
    let config = Config::from_kv_entries([("key", Value::String("val"))]);
    assert_eq!(q(&config, "key"), [Value::String("val")]);
}

#[test]
fn single_entry_no_match() {
    let config = Config::from_kv_entries([("key", Value::String("val"))]);
    assert!(q(&config, "other").is_empty());
    assert!(q(&config, "ke").is_empty());
    assert!(q(&config, "keys").is_empty());
}

#[test]
fn multiple_distinct_keys() {
    let config = Config::from_kv_entries([
        ("a", Value::String("1")),
        ("b", Value::Integer("2")),
        ("c", Value::Bool(true)),
    ]);
    assert_eq!(q(&config, "a"), [Value::String("1")]);
    assert_eq!(q(&config, "b"), [Value::Integer("2")]);
    assert_eq!(q(&config, "c"), [Value::Bool(true)]);
}

#[test]
fn duplicate_keys_return_in_insertion_order() {
    let config = Config::from_kv_entries([
        ("k", Value::String("first")),
        ("other", Value::Bool(false)),
        ("k", Value::String("second")),
        ("k", Value::String("third")),
    ]);
    assert_eq!(
        q(&config, "k"),
        [Value::String("first"), Value::String("second"), Value::String("third"),]
    );
    assert_eq!(q(&config, "other"), [Value::Bool(false)]);
}

#[test]
fn interleaved_duplicates_preserve_per_key_order() {
    let config = Config::from_kv_entries([
        ("x", Value::String("x1")),
        ("y", Value::String("y1")),
        ("x", Value::String("x2")),
        ("y", Value::String("y2")),
        ("x", Value::String("x3")),
    ]);
    assert_eq!(q(&config, "x"), [Value::String("x1"), Value::String("x2"), Value::String("x3")]);
    assert_eq!(q(&config, "y"), [Value::String("y1"), Value::String("y2")]);
}

#[test]
fn empty_string_key() {
    let config = Config::from_kv_entries([
        ("", Value::String("empty_key")),
        ("a", Value::String("a")),
        ("", Value::Integer("42")),
    ]);
    assert_eq!(q(&config, ""), [Value::String("empty_key"), Value::Integer("42")]);
}

#[test]
fn all_value_types() {
    let config = Config::from_kv_entries([
        ("b", Value::Bool(true)),
        ("b", Value::Bool(false)),
        ("i", Value::Integer("0")),
        ("i", Value::Integer("-99")),
        ("s", Value::String("")),
        ("s", Value::String("hello world")),
    ]);
    assert_eq!(q(&config, "b"), [Value::Bool(true), Value::Bool(false)]);
    assert_eq!(q(&config, "i"), [Value::Integer("0"), Value::Integer("-99")]);
    assert_eq!(q(&config, "s"), [Value::String(""), Value::String("hello world")]);
}

#[test]
fn prefix_of_key_does_not_match() {
    let config = Config::from_kv_entries([("abc", Value::String("1")), ("ab", Value::String("2"))]);
    assert!(q(&config, "a").is_empty());
    assert_eq!(q(&config, "ab"), [Value::String("2")]);
    assert_eq!(q(&config, "abc"), [Value::String("1")]);
}

#[test]
fn many_duplicates_preserve_insertion_order() {
    let entries: Vec<(&str, Value)> = (0..100)
        .map(|i| ("key", Value::Integer(if i % 2 == 0 { "even" } else { "odd" })))
        .collect();
    let config = Config::from_kv_entries(entries);
    let result = q(&config, "key");
    assert_eq!(result.len(), 100);
    for (i, v) in result.iter().enumerate() {
        let expected = if i % 2 == 0 { Value::Integer("even") } else { Value::Integer("odd") };
        assert_eq!(*v, expected, "mismatch at index {}", i);
    }
}

#[test]
fn many_keys_interleaved_query_each() {
    let keys = ["alpha", "beta", "gamma"];
    let entries: Vec<(&str, Value)> = (0..90)
        .map(|i| {
            let k = keys[i % 3];
            let v = Value::String(k);
            (k, v)
        })
        .collect();
    let config = Config::from_kv_entries(entries);
    for k in &keys {
        let result = q(&config, k);
        assert_eq!(result.len(), 30);
        assert!(result.iter().all(|v| *v == Value::String(k)));
    }
    assert!(q(&config, "delta").is_empty());
}

fn reference_query<'a>(entries: &'a [(String, String)], key: &str) -> Vec<&'a str> {
    entries.iter().filter(|(k, _)| k == key).map(|(_, v)| v.as_str()).collect()
}

#[property_test]
fn matches_linear_scan(entries: Vec<(String, String)>, query_key: String) {
    let config = Config::from_kv_entries(
        entries.iter().map(|(k, v)| (k.as_str(), Value::String(v.as_str()))),
    );
    let actual: Vec<&str> = config
        .query(&query_key)
        .map(|v| match v {
            Value::String(s) => s,
            _ => unreachable!(),
        })
        .collect();
    let expected = reference_query(&entries, &query_key);
    prop_assert_eq!(actual, expected);
}

#[property_test]
fn high_collision_matches_linear_scan(
    #[strategy = prop::collection::vec(("[a-c]{1,2}", "[a-z]{0,8}"), 0..100)] entries: Vec<(
        String,
        String,
    )>,
    #[strategy = "[a-c]{1,2}"] query_key: String,
) {
    let config = Config::from_kv_entries(
        entries.iter().map(|(k, v)| (k.as_str(), Value::String(v.as_str()))),
    );
    let actual: Vec<&str> = config
        .query(&query_key)
        .map(|v| match v {
            Value::String(s) => s,
            _ => unreachable!(),
        })
        .collect();
    let expected = reference_query(&entries, &query_key);
    prop_assert_eq!(actual, expected);
}

#[property_test]
fn absent_key_returns_empty(
    #[strategy = prop::collection::vec(("[a-c]{1,3}", "[a-z]{0,5}"), 0..15)] entries: Vec<(
        String,
        String,
    )>,
    #[strategy = "[x-z]{1,3}"] query_key: String,
) {
    let config = Config::from_kv_entries(
        entries.iter().map(|(k, v)| (k.as_str(), Value::String(v.as_str()))),
    );
    prop_assert_eq!(config.query(&query_key).count(), 0);
}
