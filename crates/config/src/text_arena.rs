use std::ops::Index;

pub(crate) struct TextArena {
    buffer: String,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TextId {
    offset: u32,
    length: u32,
}

impl TextArena {
    #[inline]
    pub(crate) fn new() -> Self {
        Self { buffer: String::new() }
    }

    #[inline]
    pub(crate) fn alloc(&mut self, text: &str) -> TextId {
        let offset = self.buffer.len() as u32;
        let length = text.len() as u32;
        self.buffer.push_str(text);
        TextId { offset, length }
    }
}

impl Index<TextId> for TextArena {
    type Output = str;

    #[inline]
    fn index(&self, id: TextId) -> &Self::Output {
        let start = id.offset as usize;
        let end = start + id.length as usize;
        &self.buffer[start..end]
    }
}
