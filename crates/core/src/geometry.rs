use serde::{Deserialize, Serialize};

/// A rectangle of physical pixels in desktop coordinates, with the origin at the top left.
///
/// `right` and `bottom` are exclusive, so two rectangles that share an edge value touch
/// without overlapping.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}
