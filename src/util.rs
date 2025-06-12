use crate::{
    bvh::{Bvh, Tri},    
};
use bevy::{
    math::bounding::RayCast3d,
    prelude::*,
};
use std::mem::swap;

#[derive(Debug, Clone, Copy)]
pub struct Hit {
    pub distance: f32, // intersection distance along ray, often seen as t
    pub u: f32,        // barycentric coordinates of the intersection
    pub v: f32,
    pub tri_index: usize,    
}

impl Default for Hit {
    fn default() -> Self {
        Self {
            distance: 1e30f32,
            u: Default::default(),
            v: Default::default(),
            tri_index: Default::default(),                        
        }
    }
}

pub trait RayCastExt {
    /// Converting ray into another space, and how much the range was scaled by
    fn to_local(&self, transform: &GlobalTransform) -> (RayCast3d, f32);

    /// Get the point at a given distance along the ray.    
    fn get_point(&self, distance: f32) -> Vec3A;

    fn intersect_triangle(&self, tri: &Tri, tri_index: usize) -> Option<Hit>;

    //fn intersect_tlas(&self, tlas: &Tlas) -> Option<Hit>;

    fn intersect_bvh(&self, bvh: &Bvh) -> Option<Hit>;

    // fn intersect_bvh_instance(&self, bvh_instance: &BvhInstance, bvhs: &[Bvh]) -> Option<Hit>;
}

impl RayCastExt for RayCast3d {
    #[inline]
    fn to_local(&self, transform: &GlobalTransform) -> (RayCast3d, f32) {
        let to_local = transform.affine().inverse();
        let local_origin = to_local.transform_point3a(self.origin);
        let local_dir = to_local.transform_vector3a(self.direction.as_vec3a());
        // Compute how much the direction vector changed length
        let dir_scale = local_dir.length() / self.direction.as_vec3a().length();

        (RayCast3d::new(
            local_origin,
            Dir3A::new(local_dir).unwrap(),
            self.max * dir_scale
        ), dir_scale)
    }

    #[inline]
    fn get_point(&self, distance: f32) -> Vec3A {
        self.origin + *self.direction * distance
    }

    #[inline(always)]
    fn intersect_triangle(&self, tri: &Tri, tri_index: usize) -> Option<Hit> {
        #[cfg(feature = "trace")]
        let _span = info_span!("intersect_triangle").entered();
        let edge1 = tri.vertex1 - tri.vertex0;
        let edge2 = tri.vertex2 - tri.vertex0;
        let h = self.direction.as_vec3a().cross(edge2);
        let a = edge1.dot(h);
        if a.abs() < 0.00001 {
            return None;
        }

        // ray parallel to triangle
        let f = 1.0 / a;
        let s = self.origin - tri.vertex0;
        let u = f * s.dot(h);
        if !(0.0..=1.0).contains(&u) {
            return None;
        }
        let q = s.cross(edge1);
        let v = f * self.direction.dot(q);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        let t = f * edge2.dot(q);

        if t > 0.0001 {
            return Some(Hit {
                distance: t,
                u,
                v,
                tri_index,                
            });            
        }
        None
    }

    fn intersect_bvh(&self, bvh: &Bvh) -> Option<Hit> {
        #[cfg(feature = "trace")]
        let _span = info_span!("intersect_bvh").entered();
        let mut node = &bvh.nodes[0];
        let mut stack = Vec::with_capacity(64);
        let mut best_hit: Option<Hit> = None;

        // PERF: clone the ray so we can update max distance as we find hits to tighten our search,
        // more complex the scene the big the performance win
        let mut ray = self.clone();

        loop {
            if node.is_leaf() {
                for i in 0..node.tri_count {
                    let tri_index = bvh.triangle_indexs[(node.left_first + i) as usize];
                    if let Some(hit) =
                        ray.intersect_triangle(&bvh.tris[tri_index], tri_index)
                    {
                        if let Some(best) = best_hit {
                            if hit.distance < best.distance {
                                best_hit = Some(hit);
                                ray.max = hit.distance; // tighten the ray
                            }
                        } else {
                            best_hit = Some(hit);
                            ray.max = hit.distance; // tighten the ray
                        }
                    }
                }
                if stack.is_empty() {
                    break;
                }
                node = stack.pop().unwrap();
                continue;
            }
            let mut child1 = &bvh.nodes[node.left_first as usize];
            let mut child2 = &bvh.nodes[(node.left_first + 1) as usize];

            let mut dist1 = ray.aabb_intersection_at(&child1.aabb);
            let mut dist2 = ray.aabb_intersection_at(&child2.aabb);

            // Sort the children by distance
            if dist1.unwrap_or(f32::MAX) > dist2.unwrap_or(f32::MAX) {
                swap(&mut dist1, &mut dist2);
                swap(&mut child1, &mut child2);
            }

            if dist1.is_none() {
                if stack.is_empty() {
                    break;
                }
                node = stack.pop().unwrap();
            } else {
                node = child1;
                if dist2.is_some() {
                    stack.push(child2);
                }
            }
        }
        best_hit
    }    
}
