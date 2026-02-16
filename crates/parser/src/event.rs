use mical_syntax::SyntaxKind;
use std::{borrow::Cow, mem, num::NonZeroU32};

pub(crate) enum Event {
    StartNode { kind: SyntaxKind, forward_parent: Option<NonZeroU32> },
    FinishNode,
    Token { kind: SyntaxKind, len: u32 },
    Error { message: Cow<'static, str> },
}

#[derive(Debug)]
enum EventRaw {
    StartNode { kind: SyntaxKind, forward_parent: Option<NonZeroU32> },
    FinishNode,
    Token { kind: SyntaxKind, len: u32 },
    Error { message_index: u32 },
    Tombstone,
}

const _: () = const {
    assert!(size_of::<EventRaw>() == 8);
};

#[derive(Debug)]
pub(crate) struct EventContainer {
    events: Vec<EventRaw>,
    errors: Vec<Cow<'static, str>>,
}

impl EventContainer {
    pub(crate) fn new() -> Self {
        EventContainer { events: Vec::new(), errors: Vec::new() }
    }

    pub(crate) fn push(&mut self, event: Event) {
        let event_raw = self.convert_event(event);
        self.events.push(event_raw);
    }

    pub(crate) fn push_tombstone(&mut self) {
        self.events.push(EventRaw::Tombstone);
    }

    pub(crate) fn replace_tombstone(&mut self, index: usize, event: Event) {
        match &self.events[index] {
            EventRaw::Tombstone => {
                let event_raw = self.convert_event(event);
                self.events[index] = event_raw;
            }
            _ => panic!("Expected a tombstone at index {index}"),
        }
    }

    pub(crate) fn set_forward_parent(&mut self, index: usize, forward_parent: u32) {
        let Some(forward_parent) = NonZeroU32::new(forward_parent) else {
            panic!("Forward parent must be a positive integer");
        };
        match &mut self.events[index] {
            EventRaw::StartNode { forward_parent: fp, .. } => {
                assert!(fp.is_none(), "Forward parent is already set for event at index {index}");
                *fp = Some(forward_parent);
            }
            _ => panic!("Expected a StartNode event at index {}", index),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.events.len()
    }

    fn convert_event(&mut self, event: Event) -> EventRaw {
        match event {
            Event::StartNode { kind, forward_parent } => {
                EventRaw::StartNode { kind, forward_parent }
            }
            Event::FinishNode => EventRaw::FinishNode,
            Event::Token { kind, len } => EventRaw::Token { kind, len },
            Event::Error { message } => {
                let message_index = self.errors.len() as u32;
                self.errors.push(message);
                EventRaw::Error { message_index }
            }
        }
    }

    pub(crate) fn take(&mut self, index: usize) -> Event {
        match mem::replace(&mut self.events[index], EventRaw::Tombstone) {
            EventRaw::StartNode { kind, forward_parent } => {
                Event::StartNode { kind, forward_parent }
            }
            EventRaw::FinishNode => Event::FinishNode,
            EventRaw::Token { kind, len } => Event::Token { kind, len },
            EventRaw::Error { message_index } => {
                let message = self.errors[message_index as usize].clone();
                Event::Error { message }
            }
            EventRaw::Tombstone => panic!("Tombstone should be replaced before taking"),
        }
    }
}

// impl IntoIterator for EventContainer {
//     type Item = Event;
//     type IntoIter = IntoIter;
//
//     fn into_iter(self) -> Self::IntoIter {
//         IntoIter { iter: self.events.into_iter(), errors: self.errors }
//     }
// }
//
// pub(crate) struct IntoIter {
//     iter: std::vec::IntoIter<EventRaw>,
//     errors: Vec<Cow<'static, str>>,
// }
//
// impl Iterator for IntoIter {
//     type Item = Event;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next().map(|event_raw| match event_raw {
//             EventRaw::StartNode { kind } => Event::StartNode { kind },
//             EventRaw::FinishNode => Event::FinishNode,
//             EventRaw::Token { kind, len } => Event::Token { kind, len },
//             EventRaw::Error { message_index } => {
//                 let message = self.errors[message_index as usize].clone();
//                 Event::Error { message }
//             }
//             EventRaw::Tombstone => panic!("Tombstone should be replaced before iteration"),
//         })
//     }
// }
