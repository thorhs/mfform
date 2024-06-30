use crate::pos::Pos;

#[derive(Debug, Clone, Eq)]
pub struct Widget {
    pub pos: Pos,
    pub widget_type: WidgetType,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub enum WidgetType {
    Text {
        value: String,
    },
    Generic {
        length: u16,
        name: String,
        value: String,
        default_value: String,
        allowed_characters: Option<Vec<char>>,
        mask_char: Option<char>,
    },
}

impl Ord for Widget {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pos.cmp(&other.pos)
    }
}

impl PartialEq for Widget {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos // TODO: && self.widget_type == other.widget_type
    }
}

impl PartialOrd for Widget {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Widget {
    pub fn is_input(&self) -> bool {
        matches!(self.widget_type, WidgetType::Generic { .. })
    }

    pub fn new_label(pos: impl Into<Pos>, text: impl Into<String>) -> Self {
        Self {
            pos: pos.into(),
            widget_type: WidgetType::Text { value: text.into() },
        }
    }

    pub fn new_generic(
        pos: impl Into<Pos>,
        length: u16,
        name: impl Into<String>,
        value: impl Into<String>,
        default_value: impl Into<String>,
        allowed_characters: Option<impl Into<Vec<char>>>,
        mask_char: Option<char>,
    ) -> Self {
        Self {
            pos: pos.into(),
            widget_type: WidgetType::Generic {
                length,
                name: name.into(),
                value: value.into(),
                default_value: default_value.into(),
                allowed_characters: allowed_characters.map(|a| a.into()),
                mask_char,
            },
        }
    }
}
