use bevy::prelude::*;

// TODO: Will be replaced by bevy mesh data
//, stop gap to get things working
#[derive(Default, Debug, Copy, Clone)]
pub struct Tri {
    pub vertex0: Vec3A,
    pub vertex1: Vec3A,
    pub vertex2: Vec3A,
    pub centroid: Vec3A,
}

impl Tri {
    pub fn new(v0: Vec3A, v1: Vec3A, v2: Vec3A) -> Self {
        Tri {
            vertex0: v0,
            vertex1: v1,
            vertex2: v2,
            centroid: (v0 + v1 + v2) / 3.0,
        }
    }
}
