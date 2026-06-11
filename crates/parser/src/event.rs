use mical_cli_syntax::SyntaxKind;

#[derive(Debug)]
pub(crate) enum Event {
    StartNode { kind: SyntaxKind },
    FinishNode,
    Token { kind: SyntaxKind, len: u32 },
    Error { message: &'static str, len: u32 },
}

// Errors are rare, so their payload lives in a side table and the stream
// element carries only an index: the stream stays at 8 bytes per event
// instead of paying the message pointer on every element.
#[derive(Debug)]
enum EventRaw {
    StartNode { kind: SyntaxKind },
    FinishNode,
    Token { kind: SyntaxKind, len: u32 },
    Error { index: u32 },
}

// A tombstone is a free niche: `None` must not grow the element size.
const _: () = const {
    assert!(size_of::<Option<EventRaw>>() == 8);
};

#[derive(Debug)]
pub(crate) struct EventContainer {
    events: Vec<Option<EventRaw>>,
    errors: Vec<(&'static str, u32)>,
}

impl EventContainer {
    pub(crate) fn new() -> Self {
        EventContainer { events: Vec::new(), errors: Vec::new() }
    }

    pub(crate) fn push(&mut self, event: Event) {
        let raw = self.convert_event(event);
        self.events.push(Some(raw));
    }

    pub(crate) fn push_tombstone(&mut self) {
        self.events.push(None);
    }

    pub(crate) fn replace_tombstone(&mut self, index: usize, event: Event) {
        let raw = self.convert_event(event);
        let slot = &mut self.events[index];
        assert!(slot.is_none(), "expected a tombstone at index {index}");
        *slot = Some(raw);
    }

    pub(crate) fn len(&self) -> usize {
        self.events.len()
    }

    fn convert_event(&mut self, event: Event) -> EventRaw {
        match event {
            Event::StartNode { kind } => EventRaw::StartNode { kind },
            Event::FinishNode => EventRaw::FinishNode,
            Event::Token { kind, len } => EventRaw::Token { kind, len },
            Event::Error { message, len } => {
                let index = self.errors.len() as u32;
                self.errors.push((message, len));
                EventRaw::Error { index }
            }
        }
    }
}

impl IntoIterator for EventContainer {
    type Item = Event;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { iter: self.events.into_iter(), errors: self.errors }
    }
}

pub(crate) struct IntoIter {
    iter: std::vec::IntoIter<Option<EventRaw>>,
    errors: Vec<(&'static str, u32)>,
}

impl Iterator for IntoIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.iter.next()?;
        let raw = raw.expect("tombstone should be replaced before iteration");
        Some(match raw {
            EventRaw::StartNode { kind } => Event::StartNode { kind },
            EventRaw::FinishNode => Event::FinishNode,
            EventRaw::Token { kind, len } => Event::Token { kind, len },
            EventRaw::Error { index } => {
                let (message, len) = self.errors[index as usize];
                Event::Error { message, len }
            }
        })
    }
}
