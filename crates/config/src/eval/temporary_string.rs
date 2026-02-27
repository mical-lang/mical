pub(super) struct TemporaryString {
    buffer: String,
}

impl TemporaryString {
    pub(super) fn new() -> Self {
        Self { buffer: String::new() }
    }

    pub(super) fn get(&mut self) -> &mut String {
        self.buffer.clear();
        &mut self.buffer
    }
}
