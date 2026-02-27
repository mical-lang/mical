use std::ops;

pub(super) trait Joined {
    fn joined(&mut self, text: &str) -> JoinedStr<'_>;
}

impl Joined for String {
    fn joined(&mut self, text: &str) -> JoinedStr<'_> {
        JoinedStr::new(self, text)
    }
}

pub(super) struct JoinedStr<'a> {
    buffer: &'a mut String,
    original_len: usize,
}

impl<'a> JoinedStr<'a> {
    pub(super) fn new(buffer: &'a mut String, text: &str) -> Self {
        let original_len = buffer.len();
        buffer.push_str(text);
        JoinedStr { buffer, original_len }
    }
}

impl ops::Deref for JoinedStr<'_> {
    type Target = str;

    fn deref(&self) -> &str {
        self.buffer
    }
}

impl Drop for JoinedStr<'_> {
    fn drop(&mut self) {
        self.buffer.truncate(self.original_len);
    }
}
