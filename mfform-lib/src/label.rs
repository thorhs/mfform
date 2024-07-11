use crate::pos::Pos;

/// Text label
#[derive(Debug, Clone, Eq)]
pub struct Label {
    pub pos: Pos,
    pub text: String,
}

impl Ord for Label {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pos.cmp(&other.pos)
    }
}

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl PartialOrd for Label {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Label {
    /// Create a new label at the specified position
    pub fn new_label(pos: impl Into<Pos>, text: impl Into<String>) -> Self {
        let text: String = text.into();
        Self {
            pos: pos.into(),
            text,
        }
    }
}
