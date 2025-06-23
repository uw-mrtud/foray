use crate::StableMap;
use derive_more::derive::Debug;
use iced::advanced::layout;

use crate::math::Point;

pub type ShapeId = u32;
#[derive(Debug)]
pub struct Shape<T> {
    pub position: Point,
    #[debug(skip)]
    pub state: T,
}

impl<T> Shape<T> {
    pub fn new(position: Point, content: T) -> Self {
        Self {
            position,
            state: content,
        }
    }
}

#[derive(Debug)]
pub struct Shapes<T>(pub StableMap<ShapeId, Shape<T>>);

impl<T> Default for Shapes<T> {
    fn default() -> Self {
        Self(StableMap::new())
    }
}

impl<T> Shapes<T> {
    pub fn find_shape(&self, point: Point, layout: layout::Layout) -> Option<(ShapeId, Point)> {
        self.0
            .iter()
            .zip(layout.children())
            .find_map(|((id, shape), layout)| {
                let bounds = layout.bounds();
                if bounds.contains(point.into()) {
                    Some((*id, shape.position))
                } else {
                    None
                }
            })
    }
}
