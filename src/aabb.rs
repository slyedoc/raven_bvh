use bevy::{math::bounding::Aabb3d, prelude::*};

pub trait Aabb3dExt {
    fn init() -> Self;

    fn area(&self) -> f32;

    fn expand(&mut self, point: Vec3A);

    fn expand_aabb(&mut self, aabb: &Aabb3d);
}

impl Aabb3dExt for Aabb3d {
    /// Initializes an Aabb3d with impossibly values, always set after init to this
    // TODO: remove this
    #[inline]
    fn init() -> Self {
        Aabb3d {
            min: Vec3A::splat(1e30f32),
            max: Vec3A::splat(-1e30f32),
        }
    }

    #[inline]
    fn area(&self) -> f32 {
        let e = self.max - self.min;
        e.x * e.y + e.y * e.z + e.z * e.x
    }

    #[inline]
    fn expand(&mut self, point: Vec3A) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    #[inline]
    fn expand_aabb(&mut self, aabb: &Aabb3d) {
        self.expand(aabb.min);
        self.expand(aabb.max);
    }
}
