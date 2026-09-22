use iced::Point;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn from_points(first: Point, second: Point) -> Self {
        Self {
            x: first.x.min(second.x),
            y: first.y.min(second.y),
            width: (first.x - second.x).abs(),
            height: (first.y - second.y).abs(),
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangles_are_normalized_and_intersected() {
        let rect = Rect::from_points(Point::new(100.0, 100.0), Point::new(50.0, 50.0));
        assert_eq!(
            rect,
            Rect {
                x: 50.0,
                y: 50.0,
                width: 50.0,
                height: 50.0
            }
        );
        assert!(rect.intersects(&Rect {
            x: 75.0,
            y: 75.0,
            width: 20.0,
            height: 20.0
        }));
        assert!(rect.contains(Point::new(50.0, 50.0)));
    }
}
