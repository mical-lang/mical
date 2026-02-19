use crate::{Config, Value};
use smallvec::ToSmallVec;

fn normalize_integer(s: &str) -> i64 {
    let s = if let Some(rest) = s.strip_prefix('+') { rest } else { s };
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i64::from_str_radix(hex, 16).unwrap_or_else(|_| {
            // TODO: handle integers that don't fit in i64
            panic!("unsupported: integer '{s}' does not fit in i64")
        })
    } else if let Some(hex) = s.strip_prefix("-0x").or_else(|| s.strip_prefix("-0X")) {
        let abs = i64::from_str_radix(hex, 16).unwrap_or_else(|_| {
            // TODO: handle integers that don't fit in i64
            panic!("unsupported: integer '{s}' does not fit in i64")
        });
        abs.checked_neg().unwrap_or_else(|| {
            // TODO: handle integers that don't fit in i64
            panic!("unsupported: integer '{s}' does not fit in i64")
        })
    } else {
        s.parse::<i64>().unwrap_or_else(|_| {
            // TODO: handle integers that don't fit in i64
            panic!("unsupported: integer '{s}' does not fit in i64")
        })
    }
}

impl<'s> Value<'s> {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Integer(s) => {
                let n = normalize_integer(s);
                serde_json::Value::Number(n.into())
            }
            Value::String(s) => serde_json::Value::String((*s).to_owned()),
        }
    }
}

impl Config {
    pub fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for &(group_start, count) in &self.group_order {
            let (group_start, count) = (group_start as usize, count as usize);
            let mut idxs: smallvec::SmallVec<[u32; 2]> =
                { self.sorted_indices[group_start..(group_start + count)].to_smallvec() };
            idxs.sort_unstable();
            let key = &self.arena[self.entries[idxs[0] as usize].0];
            if count == 1 {
                let val = self.entries[idxs[0] as usize].1.to_value(&self.arena);
                map.insert(key.to_owned(), val.to_json());
            } else {
                let arr: Vec<serde_json::Value> = idxs
                    .iter()
                    .map(|&i| {
                        let val = self.entries[i as usize].1.to_value(&self.arena);
                        val.to_json()
                    })
                    .collect();
                map.insert(key.to_owned(), serde_json::Value::Array(arr));
            }
        }
        serde_json::Value::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_integer;

    #[test]
    fn decimal() {
        assert_eq!(normalize_integer("42"), 42);
        assert_eq!(normalize_integer("0"), 0);
    }

    #[test]
    fn signed() {
        assert_eq!(normalize_integer("+7"), 7);
        assert_eq!(normalize_integer("-10"), -10);
        assert_eq!(normalize_integer("+0"), 0);
    }

    #[test]
    fn hex() {
        assert_eq!(normalize_integer("0xFF"), 255);
        assert_eq!(normalize_integer("0XFF"), 255);
        assert_eq!(normalize_integer("0x0"), 0);
    }

    #[test]
    fn signed_hex() {
        assert_eq!(normalize_integer("-0xFF"), -255);
        assert_eq!(normalize_integer("-0XA"), -10);
    }

    #[test]
    #[should_panic(expected = "unsupported")]
    fn overflow_panics() {
        normalize_integer("99999999999999999999");
    }
}
