use crate::pos::Pos;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Select {
    None,
    SingleSelect,
    MultiSelect,
}

#[derive(Debug, Clone, Eq)]
pub struct Input {
    pub pos: Pos,
    pub length: u16,
    pub name: String,
    pub value: String,
    pub default_value: String,
    pub allowed_characters: Option<Vec<char>>,
    pub mask_char: Option<char>,
    pub select: Select,
    pub select_static: Vec<(String, String)>,
}

impl Ord for Input {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pos.cmp(&other.pos)
    }
}

impl PartialEq for Input {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos // TODO: && self.widget_type == other.widget_type
    }
}

impl PartialOrd for Input {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Input {
    pub fn has_focus(&self, cursor: Pos) -> bool {
        cursor.within(self.pos, self.length).is_some()
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
            name: name.into(),
            value: value.into(),
            default_value: default_value.into(),
            allowed_characters: allowed_characters.map(|a| a.into()),
            mask_char,
            select,
            select_static: Default::default(),
        }
    }
}
