use std::cmp::{max, min};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Pos {
    pub x: u16,
    pub y: u16,
}

impl Pos {
    pub fn constrain<I: Into<Self> + Copy>(self, other: I) -> Self {
        Self {
            x: max(min(self.x, other.into().x - 1), 0),
            y: max(min(self.y, other.into().y - 1), 0),
        }
    }

    pub fn within(self, other: Self, length: u16) -> Option<usize> {
        if self.x >= other.x && self.x <= other.x + length && self.y == other.y {
            Some((self.x - other.x) as usize)
        } else {
            None
        }
    }

    pub fn move_x(self, by: i16, max: u16) -> Self {
        Pos {
            x: (self.x as i16 + by) as u16,
            y: self.y,
        }
        .constrain((max, u16::MAX))
    }
}

impl From<(u16, u16)> for Pos {
    fn from((x, y): (u16, u16)) -> Self {
        Self { x, y }
    }
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.y.cmp(&other.y) {
            core::cmp::Ordering::Equal => self.x.cmp(&other.x),
            ord => ord,
        }
    }
}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn within_1() {
        let pos: Pos = (2, 2).into();
        let field: Pos = (0, 2).into();

        let res = pos.within(field, 4);

        assert_eq!(res, Some(2));
    }

    #[test]
    fn within_2() {
        let pos: Pos = (6, 2).into();
        let field: Pos = (2, 2).into();

        let res = pos.within(field, 4);

        assert_eq!(res, Some(4));
    }

    #[test]
    fn outside_x() {
        let pos: Pos = (2, 2).into();
        let field: Pos = (7, 2).into();

        let res = pos.within(field, 4);

        assert_eq!(res, None);
    }

    #[test]
    fn outside_y() {
        let pos: Pos = (2, 2).into();
        let field: Pos = (2, 0).into();

        let res = pos.within(field, 4);

        assert_eq!(res, None);
    }

    #[test]
    fn ordering() {
        let test_pos: Pos = (2, 2).into();

        assert_eq!(test_pos.cmp(&(0, 0).into()), Ordering::Greater);
        assert_eq!(test_pos.cmp(&(2, 0).into()), Ordering::Greater);
        assert_eq!(test_pos.cmp(&(0, 2).into()), Ordering::Greater);
        assert_eq!(test_pos.cmp(&(2, 2).into()), Ordering::Equal);
        assert_eq!(test_pos.cmp(&(2, 4).into()), Ordering::Less);
        assert_eq!(test_pos.cmp(&(0, 4).into()), Ordering::Less);
        assert_eq!(test_pos.cmp(&(4, 4).into()), Ordering::Less);
    }
}
