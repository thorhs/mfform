use crate::pos::Pos;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Select {
    None,
    SingleSelect,
    MultiSelect,
}

#[derive(Debug, Clone, Eq)]
pub struct Widget {
    pub pos: Pos,
    pub length: u16,
    pub widget_type: WidgetType,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub enum WidgetType {
    Text {
        value: String,
    },
    Generic {
        name: String,
        value: String,
        default_value: String,
        allowed_characters: Option<Vec<char>>,
        mask_char: Option<char>,
        select: Select,
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

    pub fn has_focus(&self, other: &Pos) -> bool {
        self.is_input() && self.pos.within(other, self.length).is_some()
    }

    pub fn new_label(pos: impl Into<Pos>, text: impl Into<String>) -> Self {
        let text: String = text.into();
        Self {
            pos: pos.into(),
            length: text.len() as u16,
            widget_type: WidgetType::Text { value: text },
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
        select: Select,
    ) -> Self {
        Self {
            pos: pos.into(),
            length,
            widget_type: WidgetType::Generic {
                name: name.into(),
                value: value.into(),
                default_value: default_value.into(),
                allowed_characters: allowed_characters.map(|a| a.into()),
                mask_char,
                select,
            },
        }
    }
}
