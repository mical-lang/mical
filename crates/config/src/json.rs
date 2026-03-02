use crate::{Config, KeyGroups, Value, Values};
use compact_str::CompactString;
use num_bigint::BigUint;
use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq};

pub struct JsonView<T>(pub T);

fn serialize_integer<S: serde::Serializer>(s: &str, serializer: S) -> Result<S::Ok, S::Error> {
    if let Ok(n) = s.parse::<i64>() {
        return serializer.serialize_i64(n);
    }

    let (is_negative, s) = match s.as_bytes().first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s),
    };
    let (radix, s) = match s.as_bytes() {
        [b'0', b'x', ..] => (16, &s[2..]),
        [b'0', b'o', ..] => (8, &s[2..]),
        [b'0', b'b', ..] => (2, &s[2..]),
        _ => (10, s),
    };

    let clean = s.bytes().filter(|&b| b != b'_').map(|b| b as char).collect::<CompactString>();

    if let Ok(n) = u64::from_str_radix(&clean, radix) {
        if is_negative {
            if n <= i64::MIN.unsigned_abs() {
                return serializer.serialize_i64((n as i64).wrapping_neg());
            }
            return serde_json::value::RawValue::from_string(format!("-{n}"))
                .map_err(serde::ser::Error::custom)?
                .serialize(serializer);
        } else {
            return serializer.serialize_u64(n);
        }
    }

    let parsed = BigUint::parse_bytes(clean.as_bytes(), radix).expect("valid digits should parse");
    let dec = if is_negative { format!("-{parsed}") } else { format!("{parsed}") };
    serde_json::value::RawValue::from_string(dec)
        .map_err(serde::ser::Error::custom)?
        .serialize(serializer)
}

impl Serialize for JsonView<&Value<'_>> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Integer(s) => serialize_integer(s, serializer),
            Value::String(s) => serializer.serialize_str(s),
        }
    }
}

impl Serialize for JsonView<&[Value<'_>]> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for v in self.0 {
            seq.serialize_element(&JsonView(v))?;
        }
        seq.end()
    }
}

impl Serialize for JsonView<&Values<'_>> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let KeyGroups { config, lo, hi, .. } = self.0.groups;
        let groups = KeyGroups::new(config, lo, hi);
        let mut map = serializer.serialize_map(None)?;
        for (key, idxs) in groups {
            if idxs.len() == 1 {
                let val = config.entries[idxs[0] as usize].1.to_value(&config.arena);
                map.serialize_entry(key, &JsonView(&val))?;
            } else {
                struct Array<'a> {
                    config: &'a Config,
                    idxs: &'a [u32],
                }
                impl Serialize for Array<'_> {
                    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
                        let mut seq = ser.serialize_seq(Some(self.idxs.len()))?;
                        for &i in self.idxs {
                            let val =
                                self.config.entries[i as usize].1.to_value(&self.config.arena);
                            seq.serialize_element(&JsonView(&val))?;
                        }
                        seq.end()
                    }
                }
                map.serialize_entry(key, &Array { config, idxs: &idxs })?;
            }
        }
        map.end()
    }
}

impl Serialize for JsonView<&Config> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        JsonView(&self.0.entries()).serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::JsonView;
    use crate::Value;
    use proptest::{prelude::*, property_test};

    struct JsonOfArg(String);
    impl<'a> From<&'a str> for JsonOfArg {
        fn from(s: &'a str) -> Self {
            JsonOfArg(s.to_string())
        }
    }
    impl From<String> for JsonOfArg {
        fn from(s: String) -> Self {
            JsonOfArg(s)
        }
    }
    impl From<u128> for JsonOfArg {
        fn from(u: u128) -> Self {
            JsonOfArg(u.to_string())
        }
    }
    fn json_of(s: impl Into<JsonOfArg>) -> String {
        let JsonOfArg(s) = s.into();
        let v = Value::Integer(&s);
        serde_json::to_string(&JsonView(&v)).unwrap()
    }

    #[test]
    fn decimal() {
        assert_eq!(json_of("42"), "42");
        assert_eq!(json_of("0"), "0");
        assert_eq!(json_of("1_000"), "1000");
        assert_eq!(json_of("999_999_999_999_999_999_999"), "999999999999999999999");
    }

    #[test]
    fn signed() {
        assert_eq!(json_of("+7"), "7");
        assert_eq!(json_of("-10"), "-10");
        assert_eq!(json_of("+0"), "0");
        assert_eq!(json_of("-0"), "0");
    }

    #[test]
    fn hex() {
        assert_eq!(json_of("0xFF"), "255");
        assert_eq!(json_of("0x0"), "0");
        assert_eq!(json_of("0xFFFFFFFFFFFFFFFF"), "18446744073709551615");
    }

    #[test]
    fn signed_hex() {
        assert_eq!(json_of("-0xFF"), "-255");
        assert_eq!(json_of("-0x8000000000000000"), "-9223372036854775808");
    }

    #[test]
    fn binary() {
        assert_eq!(json_of("0b1010"), "10");
        assert_eq!(json_of("0b0"), "0");
        assert_eq!(
            json_of("0b1_0000000000000000000000000000000000000000000000000000000000000000"),
            "18446744073709551616"
        );
    }

    #[test]
    fn octal() {
        assert_eq!(json_of("0o77"), "63");
        assert_eq!(json_of("0o0"), "0");
        assert_eq!(json_of("0o7777777777777777777777"), "73786976294838206463");
    }

    #[test]
    fn large_with_separator() {
        assert_eq!(json_of("0x1_0000_0000_0000_0000"), "18446744073709551616");
        assert_eq!(json_of("1_000_000_000_000_000_000_000"), "1000000000000000000000");
    }

    #[property_test]
    fn prop_decimal_128(u: u128) {
        let expected = u.to_string();
        prop_assert_eq!(&json_of(u), &expected);
        if u > 0 {
            prop_assert_eq!(json_of(format!("-{u}")), format!("-{u}"));
        }
    }

    #[property_test]
    fn prop_hex_128(u: u128) {
        prop_assert_eq!(json_of(format!("0x{u:x}")), u.to_string());
        if u > 0 {
            prop_assert_eq!(json_of(format!("-0x{u:x}")), format!("-{u}"));
        }
    }

    #[property_test]
    fn prop_octal_128(u: u128) {
        prop_assert_eq!(json_of(format!("0o{u:o}")), u.to_string());
        if u > 0 {
            prop_assert_eq!(json_of(format!("-0o{u:o}")), format!("-{u}"));
        }
    }

    #[property_test]
    fn prop_binary_128(u: u128) {
        prop_assert_eq!(json_of(format!("0b{u:b}")), u.to_string());
        if u > 0 {
            prop_assert_eq!(json_of(format!("-0b{u:b}")), format!("-{u}"));
        }
    }

    #[property_test]
    fn prop_separator_128(u: u128, mask: u64) {
        let e = u.to_string();
        prop_assert_eq!(&json_of(seped(&e, mask)), &e);
        prop_assert_eq!(json_of(format!("-{}", seped(&e, mask))), format!("-{e}"));
        prop_assert_eq!(&json_of(format!("0x{}", seped(&format!("{u:x}"), mask))), &e);
        prop_assert_eq!(&json_of(format!("0o{}", seped(&format!("{u:o}"), mask))), &e);
        prop_assert_eq!(&json_of(format!("0b{}", seped(&format!("{u:b}"), mask))), &e);
        prop_assert_eq!(json_of(format!("-0x{}", seped(&format!("{u:x}"), mask))), format!("-{e}"));
        prop_assert_eq!(json_of(format!("-0o{}", seped(&format!("{u:o}"), mask))), format!("-{e}"));
        prop_assert_eq!(json_of(format!("-0b{}", seped(&format!("{u:b}"), mask))), format!("-{e}"));
        fn seped(digits: &str, mask: u64) -> String {
            let mut s = String::with_capacity(digits.len() * 2);
            for (i, ch) in digits.chars().enumerate() {
                s.push(ch);
                if i < digits.len() - 1 && (mask >> (i % 64)) & 1 == 1 {
                    s.push('_');
                }
            }
            s
        }
    }
}
