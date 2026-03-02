use mical_cli_syntax::ast;
use smallvec::{SmallVec, ToSmallVec};

mod text_arena;
use text_arena::{TextArena, TextId};

mod error;
pub use error::Error;

mod eval;
mod json;
pub use json::JsonView;

pub struct Config {
    arena: TextArena,
    /// Entry list in insertion order
    entries: Vec<(TextId, ValueRaw)>,
    /// Sorted list of indices into `entries` by key string (for binary search)
    sorted_indices: Vec<u32>,
    /// Group information for unique keys (sorted by first occurrence order).
    /// element (group_start, count) means sorted_indices[group_start..group_start+count] are the indices of entries with the same key.
    group_order: Vec<(u32, u32)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value<'s> {
    Bool(bool),
    Integer(&'s str),
    String(&'s str),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum ValueRaw {
    Bool(bool),
    Integer(TextId),
    String(TextId),
}

impl ValueRaw {
    fn to_value<'s>(self, arena: &'s TextArena) -> Value<'s> {
        match self {
            ValueRaw::Bool(b) => Value::Bool(b),
            ValueRaw::Integer(id) => Value::Integer(&arena[id]),
            ValueRaw::String(id) => Value::String(&arena[id]),
        }
    }
}

/// Iterates over groups of entries within [lo, hi) range in first-occurrence order.
/// Each item is (key, sorted_indices) where sorted_indices contains insertion-ordered entry indices.
pub(crate) struct KeyGroups<'a> {
    pub(crate) config: &'a Config,
    pub(crate) lo: usize,
    pub(crate) hi: usize,
    pos: usize,
}

impl<'a> KeyGroups<'a> {
    pub(crate) fn new(config: &'a Config, lo: usize, hi: usize) -> Self {
        KeyGroups { config, lo, hi, pos: 0 }
    }
}

impl<'a> Iterator for KeyGroups<'a> {
    type Item = (&'a str, SmallVec<[u32; 2]>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.config.group_order.len() {
            let (gs, count) = self.config.group_order[self.pos];
            self.pos += 1;
            if (gs as usize) >= self.lo && (gs as usize) < self.hi {
                let (gs, count) = (gs as usize, count as usize);
                let mut idxs: SmallVec<[u32; 2]> =
                    self.config.sorted_indices[gs..(gs + count)].to_smallvec();
                idxs.sort_unstable();
                let key = &self.config.arena[self.config.entries[idxs[0] as usize].0];
                return Some((key, idxs));
            }
        }
        None
    }
}

pub struct Values<'a> {
    groups: KeyGroups<'a>,
    current_idxs: SmallVec<[u32; 2]>,
    idx_pos: usize,
}
// Most cases have 1 entry per key, use SmallVec to optimize for that case.
// SmallVec<[u32; 2]> is used because it has the same size as SmallVec<[u32; 1]>.
const _: () = {
    assert!(size_of::<SmallVec<[u32; 2]>>() == size_of::<SmallVec<[u32; 1]>>());
    assert!(size_of::<SmallVec<[u32; 3]>>() > size_of::<SmallVec<[u32; 1]>>());
};

impl<'a> Iterator for Values<'a> {
    type Item = (&'a str, Value<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.idx_pos < self.current_idxs.len() {
                let i = self.current_idxs[self.idx_pos];
                self.idx_pos += 1;
                let config = self.groups.config;
                let (key_id, raw) = config.entries[i as usize];
                return Some((&config.arena[key_id], raw.to_value(&config.arena)));
            }
            let (_, idxs) = self.groups.next()?;
            self.current_idxs = idxs;
            self.idx_pos = 0;
        }
    }
}

impl Config {
    pub fn from_source_file(source_file: ast::SourceFile) -> (Self, Vec<Error>) {
        let eval::Output { arena, entries, errors } = eval::eval_source_file(&source_file);
        let (sorted_indices, group_order) = Self::build_indices(&arena, &entries);
        (Config { arena, entries, sorted_indices, group_order }, errors)
    }

    pub fn from_kv_entries<'a>(items: impl IntoIterator<Item = (&'a str, Value<'a>)>) -> Self {
        let mut arena = TextArena::new();
        let mut entries = Vec::new();
        for (key, val) in items {
            let key_id = arena.alloc(key);
            let raw = match val {
                Value::Bool(b) => ValueRaw::Bool(b),
                Value::Integer(s) => ValueRaw::Integer(arena.alloc(s)),
                Value::String(s) => ValueRaw::String(arena.alloc(s)),
            };
            entries.push((key_id, raw));
        }
        let (sorted_indices, group_order) = Self::build_indices(&arena, &entries);
        Config { arena, entries, sorted_indices, group_order }
    }

    fn build_indices(
        arena: &TextArena,
        entries: &[(TextId, ValueRaw)],
    ) -> (Vec<u32>, Vec<(u32, u32)>) {
        let sorted_indices = {
            let mut indices = (0..entries.len() as u32).collect::<Vec<_>>();
            indices.sort_unstable_by(|&a, &b| {
                arena[entries[a as usize].0].cmp(&arena[entries[b as usize].0])
            });
            indices
        };
        let group_order = {
            let mut groups: Vec<(u32, u32, u32)> = Vec::new(); // (sorted_start, count, first_entry_idx)
            let mut i = 0;
            while i < sorted_indices.len() {
                let group_start = i;
                let cur_key_id = entries[sorted_indices[i] as usize].0;
                let mut min_idx = sorted_indices[i];
                i += 1;
                while i < sorted_indices.len() {
                    let next_key_id = entries[sorted_indices[i] as usize].0;
                    if arena[cur_key_id] != arena[next_key_id] {
                        break;
                    }
                    min_idx = min_idx.min(sorted_indices[i]); // chmin
                    i += 1;
                }
                groups.push((group_start as u32, (i - group_start) as u32, min_idx));
            }
            groups.sort_unstable_by_key(|&(_, _, first)| first);
            groups.into_iter().map(|(s, c, _)| (s, c)).collect()
        };
        (sorted_indices, group_order)
    }

    /// Return values that exactly match `key` in insertion order (grouped by first occurrence).
    pub fn query<'a>(&'a self, key: &str) -> impl Iterator<Item = Value<'a>> + 'a {
        let lo = self.sorted_indices.partition_point(|i| {
            let key_id = self.entries[*i as usize].0;
            &self.arena[key_id] < key
        });
        let hi = self.sorted_indices[lo..].partition_point(|i| {
            let key_id = self.entries[*i as usize].0;
            &self.arena[key_id] <= key
        }) + lo;
        let idxs: SmallVec<[u32; 2]> = {
            const _: () = {
                assert!(size_of::<SmallVec<[u32; 2]>>() == size_of::<SmallVec<[u32; 1]>>());
                assert!(size_of::<SmallVec<[u32; 3]>>() > size_of::<SmallVec<[u32; 1]>>());
            };
            let mut v = self.sorted_indices[lo..hi].to_smallvec();
            v.sort_unstable(); // insertion order
            v
        };
        idxs.into_iter().map(move |i| {
            let (_, raw) = self.entries[i as usize];
            raw.to_value(&self.arena)
        })
    }

    /// Return (key, value) pairs whose keys start with `prefix` in insertion order (grouped by first occurrence).
    pub fn query_prefix(&self, prefix: &str) -> Values<'_> {
        let lo = self.sorted_indices.partition_point(|i| {
            let key_id = self.entries[*i as usize].0;
            &self.arena[key_id] < prefix
        });
        let hi = self.sorted_indices.partition_point(|i| {
            let key_id = self.entries[*i as usize].0;
            let key = &self.arena[key_id];
            key.starts_with(prefix) || key < prefix
        });
        Values { groups: KeyGroups::new(self, lo, hi), current_idxs: SmallVec::new(), idx_pos: 0 }
    }

    /// Return all (key, value) pairs in the order they were inserted. (grouped by first occurrence)
    pub fn entries(&self) -> Values<'_> {
        let hi = self.sorted_indices.len();
        Values { groups: KeyGroups::new(self, 0, hi), current_idxs: SmallVec::new(), idx_pos: 0 }
    }
}
