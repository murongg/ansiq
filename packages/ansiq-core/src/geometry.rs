use crate::Padding;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }

        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        Some(Self::new(
            x,
            y,
            right.saturating_sub(x),
            bottom.saturating_sub(y),
        ))
    }

    pub const fn contains(self, other: Self) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.right() >= other.right()
            && self.bottom() >= other.bottom()
    }

    pub const fn can_merge_rect(self, other: Self) -> bool {
        self.intersects(other)
            || (self.x == other.x
                && self.width == other.width
                && (self.bottom() == other.y || other.bottom() == self.y))
            || (self.y == other.y
                && self.height == other.height
                && (self.right() == other.x || other.right() == self.x))
    }

    pub fn union(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self::new(x, y, right.saturating_sub(x), bottom.saturating_sub(y))
    }

    pub fn shrink(self, amount: u16) -> Self {
        let next_x = self.x.saturating_add(amount.min(self.width));
        let next_y = self.y.saturating_add(amount.min(self.height));

        if self.width <= amount.saturating_mul(2) || self.height <= amount.saturating_mul(2) {
            return Self::new(next_x, next_y, 0, 0);
        }

        Self::new(
            self.x.saturating_add(amount),
            self.y.saturating_add(amount),
            self.width.saturating_sub(amount.saturating_mul(2)),
            self.height.saturating_sub(amount.saturating_mul(2)),
        )
    }

    pub fn inset(self, padding: Padding) -> Self {
        let x = self.x.saturating_add(padding.left.min(self.width));
        let y = self.y.saturating_add(padding.top.min(self.height));
        let horizontal = padding.left.saturating_add(padding.right);
        let vertical = padding.top.saturating_add(padding.bottom);

        Self::new(
            x,
            y,
            self.width.saturating_sub(horizontal),
            self.height.saturating_sub(vertical),
        )
    }
}
