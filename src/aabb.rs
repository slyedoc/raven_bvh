use bevy::{math::bounding::Aabb3d, prelude::* };

pub trait Aabb3dExt {
    fn init() -> Self;

    fn area(&self) -> f32;

    fn expand(&mut self, point: Vec3A);

    fn expand_aabb(&mut self, aabb: &Aabb3d);
}

impl Aabb3dExt for Aabb3d {
    
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

// #[derive(Debug, Copy, Clone)]
// pub struct Aabb {
//     pub bmin: Vec3,
//     pub bmax: Vec3,
// }

// impl Default for Aabb {
//     fn default() -> Self {
//         Self {
//             bmin: Vec3::splat(1e30f32),
//             bmax: Vec3::splat(-1e30f32),
//         }
//     }
// }

// impl Aabb {
//     pub fn grow(&mut self, p: Vec3) {
//         self.bmin = self.bmin.min(p);
//         self.bmax = self.bmax.max(p);
//     }

//     pub fn grow_aabb(&mut self, b: &Aabb) {
//         self.grow(b.bmin);
//         self.grow(b.bmax);
//     }

//     pub fn area(&self) -> f32 {
//         let e = self.bmax - self.bmin; // box extent
//         e.x * e.y + e.y * e.z + e.z * e.x
//     }
// }
