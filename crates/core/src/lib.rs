//! Engine-independent logic: config, Zone geometry, Rule matching.
//!
//! Nothing in here touches an OS, so the crate compiles on any target.
//! Ticket 02 fills in config parsing, Zone -> pixel rectangle, and Rule matching.

/// A pixel rectangle in a Monitor's work area.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[cfg(test)]
mod tests {
    use super::Rect;

    #[test]
    fn rect_measures_its_pixels() {
        let rect = Rect {
            x: 0,
            y: 0,
            width: 3,
            height: 4,
        };
        assert_eq!(rect.width * rect.height, 12);
    }
}
