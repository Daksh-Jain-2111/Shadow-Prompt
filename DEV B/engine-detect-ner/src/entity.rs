#[derive(Debug, Clone, PartialEq)]
pub struct EntitySpan {
    pub entity_type: String,
    pub start: usize,
    pub end: usize,
    pub value: String,
    pub confidence: f32,
    pub source: DetectionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectionSource {
    Regex,
    NER,
    UserRule,
    Keyword,
}

impl EntitySpan {
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

