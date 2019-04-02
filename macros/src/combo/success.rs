use super::Cursor;

pub struct Success<'a, Input, Output> {
    pub value: Output,
    pub start: Cursor<'a, Input>,
    pub next: Cursor<'a, Input>,
}

impl<'a, Input, Output> Success<'a, Input, Output> {
    pub fn new(value: Output, start: &Cursor<'a, Input>, next: Cursor<'a, Input>) -> Self {
        Success {
            value,
            start: start.clone(),
            next,
        }
    }

    pub fn matched(&self) -> Vec<&'a Input> {
        let mut cursor = self.start.clone();
        let mut out = Vec::new();
        while cursor < self.next {
            if let Some(item) = cursor.get() {
                out.push(item);
            }
            cursor = cursor.next();
        }
        out
    }

    pub fn with_value(mut self, value: Output) -> Self {
        self.value = value;
        self
    }

    pub fn with_start(mut self, start: &Cursor<'a, Input>) -> Self {
        self.start = start.clone();
        self
    }

    pub fn map<F, Output2>(self, f: F) -> Success<'a, Input, Output2>
    where
        F: FnOnce(Output) -> Output2,
    {
        Success {
            value: f(self.value),
            start: self.start,
            next: self.next,
        }
    }
}
