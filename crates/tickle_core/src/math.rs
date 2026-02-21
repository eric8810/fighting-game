/// Fixed-point coordinate type (1/100 pixel precision)
/// Example: 100 = 1 pixel, 10000 = 100 pixels
pub type LogicCoord = i32;

/// 2D vector using fixed-point coordinates for deterministic physics
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LogicVec2 {
    pub x: LogicCoord,
    pub y: LogicCoord,
}

impl LogicVec2 {
    pub const ZERO: Self = Self { x: 0, y: 0 };
    pub const ONE: Self = Self { x: 100, y: 100 };
    pub const UP: Self = Self { x: 0, y: 100 };
    pub const DOWN: Self = Self { x: 0, y: -100 };
    pub const LEFT: Self = Self { x: -100, y: 0 };
    pub const RIGHT: Self = Self { x: 100, y: 0 };

    #[inline]
    pub const fn new(x: LogicCoord, y: LogicCoord) -> Self {
        Self { x, y }
    }

    /// Convert from pixels to logic coordinates
    #[inline]
    pub const fn from_pixels(x: i32, y: i32) -> Self {
        Self {
            x: x * 100,
            y: y * 100,
        }
    }

    /// Convert to render coordinates (f32 pixels)
    #[inline]
    pub fn to_render(self) -> [f32; 2] {
        [self.x as f32 / 100.0, self.y as f32 / 100.0]
    }

    /// Squared magnitude (avoids sqrt for performance)
    #[inline]
    pub fn magnitude_squared(self) -> i64 {
        (self.x as i64) * (self.x as i64) + (self.y as i64) * (self.y as i64)
    }

    /// Magnitude (uses integer sqrt approximation)
    pub fn magnitude(self) -> LogicCoord {
        let mag_sq = self.magnitude_squared();
        // Integer square root approximation
        isqrt(mag_sq) as LogicCoord
    }

    /// Normalize to unit vector (magnitude = 100)
    pub fn normalize(self) -> Self {
        let mag = self.magnitude();
        if mag == 0 {
            return Self::ZERO;
        }
        Self {
            x: (self.x as i64 * 100 / mag as i64) as LogicCoord,
            y: (self.y as i64 * 100 / mag as i64) as LogicCoord,
        }
    }

    /// Dot product
    #[inline]
    pub fn dot(self, other: Self) -> i64 {
        (self.x as i64) * (other.x as i64) + (self.y as i64) * (other.y as i64)
    }

    /// Distance to another point
    #[inline]
    pub fn distance(self, other: Self) -> LogicCoord {
        (self - other).magnitude()
    }
}

// Arithmetic operations
impl std::ops::Add for LogicVec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for LogicVec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<LogicCoord> for LogicVec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: LogicCoord) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Div<LogicCoord> for LogicVec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: LogicCoord) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::ops::Neg for LogicVec2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

/// AABB rectangle using fixed-point coordinates
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LogicRect {
    pub x: LogicCoord,
    pub y: LogicCoord,
    pub w: LogicCoord,
    pub h: LogicCoord,
}

impl LogicRect {
    #[inline]
    pub const fn new(x: LogicCoord, y: LogicCoord, w: LogicCoord, h: LogicCoord) -> Self {
        Self { x, y, w, h }
    }

    /// Create from center point and size
    #[inline]
    pub fn from_center(center: LogicVec2, w: LogicCoord, h: LogicCoord) -> Self {
        Self {
            x: center.x - w / 2,
            y: center.y - h / 2,
            w,
            h,
        }
    }

    /// Get center point
    #[inline]
    pub fn center(self) -> LogicVec2 {
        LogicVec2 {
            x: self.x + self.w / 2,
            y: self.y + self.h / 2,
        }
    }

    /// Get top-left corner
    #[inline]
    pub fn top_left(self) -> LogicVec2 {
        LogicVec2 {
            x: self.x,
            y: self.y,
        }
    }

    /// Get bottom-right corner
    #[inline]
    pub fn bottom_right(self) -> LogicVec2 {
        LogicVec2 {
            x: self.x + self.w,
            y: self.y + self.h,
        }
    }

    /// AABB intersection test
    #[inline]
    pub fn intersects(self, other: Self) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }

    /// Point containment test
    #[inline]
    pub fn contains_point(self, point: LogicVec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.w
            && point.y >= self.y
            && point.y <= self.y + self.h
    }

    /// Translate rectangle by offset
    #[inline]
    pub fn translate(self, offset: LogicVec2) -> Self {
        Self {
            x: self.x + offset.x,
            y: self.y + offset.y,
            w: self.w,
            h: self.h,
        }
    }

    /// Flip horizontally (for facing direction)
    #[inline]
    pub fn flip_x(self, pivot_x: LogicCoord) -> Self {
        Self {
            x: 2 * pivot_x - self.x - self.w,
            y: self.y,
            w: self.w,
            h: self.h,
        }
    }
}

/// Integer square root using Newton's method
fn isqrt(n: i64) -> i64 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic_vec2_constants() {
        assert_eq!(LogicVec2::ZERO, LogicVec2::new(0, 0));
        assert_eq!(LogicVec2::ONE, LogicVec2::new(100, 100));
        assert_eq!(LogicVec2::UP, LogicVec2::new(0, 100));
    }

    #[test]
    fn test_from_pixels() {
        let v = LogicVec2::from_pixels(10, 20);
        assert_eq!(v, LogicVec2::new(1000, 2000));
    }

    #[test]
    fn test_to_render() {
        let v = LogicVec2::new(1000, 2000);
        let [x, y] = v.to_render();
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
    }

    #[test]
    fn test_add() {
        let a = LogicVec2::new(100, 200);
        let b = LogicVec2::new(300, 400);
        assert_eq!(a + b, LogicVec2::new(400, 600));
    }

    #[test]
    fn test_sub() {
        let a = LogicVec2::new(500, 600);
        let b = LogicVec2::new(100, 200);
        assert_eq!(a - b, LogicVec2::new(400, 400));
    }

    #[test]
    fn test_mul() {
        let v = LogicVec2::new(100, 200);
        assert_eq!(v * 2, LogicVec2::new(200, 400));
    }

    #[test]
    fn test_div() {
        let v = LogicVec2::new(400, 600);
        assert_eq!(v / 2, LogicVec2::new(200, 300));
    }

    #[test]
    fn test_neg() {
        let v = LogicVec2::new(100, -200);
        assert_eq!(-v, LogicVec2::new(-100, 200));
    }

    #[test]
    fn test_magnitude() {
        let v = LogicVec2::new(300, 400); // 3-4-5 triangle
        assert_eq!(v.magnitude(), 500);
    }

    #[test]
    fn test_normalize() {
        let v = LogicVec2::new(300, 400);
        let n = v.normalize();
        // Should be approximately (60, 80) for unit vector
        assert!((n.magnitude() - 100).abs() < 2); // Allow small error
    }

    #[test]
    fn test_dot() {
        let a = LogicVec2::new(100, 200);
        let b = LogicVec2::new(300, 400);
        assert_eq!(a.dot(b), 100 * 300 + 200 * 400);
    }

    #[test]
    fn test_distance() {
        let a = LogicVec2::new(0, 0);
        let b = LogicVec2::new(300, 400);
        assert_eq!(a.distance(b), 500);
    }

    #[test]
    fn test_rect_intersects() {
        let a = LogicRect::new(0, 0, 100, 100);
        let b = LogicRect::new(50, 50, 100, 100);
        let c = LogicRect::new(200, 200, 100, 100);

        assert!(a.intersects(b));
        assert!(b.intersects(a));
        assert!(!a.intersects(c));
        assert!(!c.intersects(a));
    }

    #[test]
    fn test_rect_contains_point() {
        let rect = LogicRect::new(100, 100, 200, 200);

        assert!(rect.contains_point(LogicVec2::new(150, 150)));
        assert!(rect.contains_point(LogicVec2::new(100, 100))); // Edge
        assert!(rect.contains_point(LogicVec2::new(300, 300))); // Edge
        assert!(!rect.contains_point(LogicVec2::new(50, 50)));
        assert!(!rect.contains_point(LogicVec2::new(350, 350)));
    }

    #[test]
    fn test_rect_center() {
        let rect = LogicRect::new(100, 200, 400, 600);
        assert_eq!(rect.center(), LogicVec2::new(300, 500));
    }

    #[test]
    fn test_rect_from_center() {
        let center = LogicVec2::new(300, 500);
        let rect = LogicRect::from_center(center, 400, 600);
        assert_eq!(rect, LogicRect::new(100, 200, 400, 600));
    }

    #[test]
    fn test_rect_translate() {
        let rect = LogicRect::new(100, 200, 300, 400);
        let offset = LogicVec2::new(50, 100);
        let translated = rect.translate(offset);
        assert_eq!(translated, LogicRect::new(150, 300, 300, 400));
    }

    #[test]
    fn test_rect_flip_x() {
        let rect = LogicRect::new(100, 200, 50, 100);
        let flipped = rect.flip_x(200);
        // Flipped around x=200: new_x = 2*200 - 100 - 50 = 250
        assert_eq!(flipped, LogicRect::new(250, 200, 50, 100));
    }

    #[test]
    fn test_isqrt() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(9), 3);
        assert_eq!(isqrt(16), 4);
        assert_eq!(isqrt(100), 10);
        assert_eq!(isqrt(250000), 500); // 500^2 = 250000
    }
}
