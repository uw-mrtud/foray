use derive_more::derive::{Add, Mul, Neg, Sub};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize, Add, Mul, Sub, Neg, PartialOrd)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Vector {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn to_point(self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl<T> From<iced::Vector<T>> for Vector
where
    T: std::convert::Into<f32>,
{
    fn from(value: iced::Vector<T>) -> Self {
        Self {
            x: value.x.into(),
            y: value.y.into(),
        }
    }
}

impl From<Vector> for iced::Vector<f32> {
    fn from(val: Vector) -> Self {
        iced::Vector { x: val.x, y: val.y }
    }
}

impl From<[f32; 2]> for Vector {
    fn from(value: [f32; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

#[derive(Default, PartialEq, derive_more::Debug, Clone, Copy, Serialize, Deserialize)]
#[debug("({x:.2},{y:.2})")]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn to_vector(self) -> Vector {
        Vector {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<iced::Point> for Point {
    fn from(value: iced::Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
impl From<Point> for iced::Vector {
    fn from(value: Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<Point> for iced::Point<f32> {
    fn from(val: Point) -> Self {
        iced::Point { x: val.x, y: val.y }
    }
}
impl From<(f32, f32)> for Point {
    fn from(value: (f32, f32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}
impl std::ops::Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl std::ops::Sub<Vector> for Point {
    type Output = Point;

    fn sub(self, rhs: Vector) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Add<Vector> for Point {
    type Output = Point;

    fn add(self, rhs: Vector) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl std::ops::Mul<f32> for Point {
    type Output = Point;

    fn mul(self, rhs: f32) -> Self::Output {
        (self.x * rhs, self.y * rhs).into()
    }
}
