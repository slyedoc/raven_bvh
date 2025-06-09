use crate::{
    bvh::{Bvh, BvhInstance},
    tlas::{Tlas, TlasNode},
    tri::Tri,
};
use bevy::{
    math::bounding::{Aabb3d, RayCast3d},
    prelude::*,
};
use std::mem::swap;

#[derive(Debug, Clone, Copy)]
pub struct Hit {
    pub distance: f32, // intersection distance along ray, often seen as t
    pub u: f32,        // barycentric coordinates of the intersection
    pub v: f32,
    // We are using more bits here than in tutorial
    pub tri_index: usize,
    pub entity: Entity,
}

impl Default for Hit {
    fn default() -> Self {
        Self {
            distance: 1e30f32,
            u: Default::default(),
            v: Default::default(),
            tri_index: Default::default(),
            // TODO: Yes this isnt ideal, should be an option, will come back to this
            entity: Entity::from_raw(0),
        }
    }
}

pub trait RayCast3dToSpace {
    /// Converting ray into another space
    fn to_space(&self, transform: &GlobalTransform) -> RayCast3d;

    /// Get the point at a given distance along the ray.    
    fn get_point(&self, distance: f32) -> Vec3A;
}

impl RayCast3dToSpace for RayCast3d {
    // TODO: figured this be built in to bevy
    #[inline]
    fn to_space(&self, transform: &GlobalTransform) -> RayCast3d {
        let world_to = transform.affine().inverse();
        RayCast3d::new(
            world_to.transform_point3a(self.origin),
            Dir3A::new(world_to.transform_vector3a(self.direction.as_vec3a())).unwrap(),
            self.max,
        )
    }

    #[inline]
    fn get_point(&self, distance: f32) -> Vec3A {
        self.origin + *self.direction * distance
    }
}

pub trait TlasIntersect {
    fn intersect_triangle(&self, tri: &Tri, tri_index: usize, entity: Entity) -> Option<Hit>;

    fn intersect_tlas(&self, tlas: &Tlas) -> Option<Hit>;

    fn intersect_aabb(&self, aabb: &Aabb3d) -> f32;

    fn intersect_bvh(&self, bvh: &Bvh, entity: Entity) -> Option<Hit>;

    fn intersect_bvh_instance(&self, bvh_instance: &BvhInstance, bvhs: &[Bvh]) -> Option<Hit>;
}

impl TlasIntersect for RayCast3d {
    #[inline(always)]
    fn intersect_triangle(&self, tri: &Tri, tri_index: usize, entity: Entity) -> Option<Hit> {
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
                entity,
            });
            // TODO: The option part here feels sloppy
            // if let Some(hit) = self.hit {
            //     if t < hit.distance {
            //         self.hit = Some(Hit {
            //             distance: t,
            //             u,
            //             v,
            //             tri_index,
            //             entity,
            //         });
            //     }
            // } else {
            //     self.hit = Some(Hit {
            //         distance: t,
            //         u,
            //         v,
            //         tri_index,
            //         entity,
            //     });
            // }
        }
        None
    }

    #[inline(always)]
    fn intersect_aabb(&self, aabb: &Aabb3d) -> f32 {
        #[cfg(feature = "trace")]
        let _span = info_span!("intersect_aabb").entered();
        let direction_inv = self.direction_recip();
        let tx1 = (aabb.min.x - self.origin.x) * direction_inv.x;
        let tx2 = (aabb.max.x - self.origin.x) * direction_inv.x;
        let tmin = tx1.min(tx2);
        let tmax = tx1.max(tx2);
        let ty1 = (aabb.min.y - self.origin.y) * direction_inv.y;
        let ty2 = (aabb.max.y - self.origin.y) * direction_inv.y;
        let tmin = tmin.max(ty1.min(ty2));
        let tmax = tmax.min(ty1.max(ty2));
        let tz1 = (aabb.min.z - self.origin.z) * direction_inv.z;
        let tz2 = (aabb.max.z - self.origin.z) * direction_inv.z;
        let tmin = tmin.max(tz1.min(tz2));
        let tmax = tmax.min(tz1.max(tz2));

        // Most intersect test would return here with a tmax and min test
        // but we are also sorting
        // let t_hit = if let Some(hit) = self.hit {
        //     hit.distance
        // } else {
        //     1e30f32
        // };

        if tmax >= tmin && tmin < self.max && tmax > 0.0 {
            tmin
        } else {
            1e30f32
        }
    }

    fn intersect_bvh(&self, bvh: &Bvh, entity: Entity) -> Option<Hit> {
        #[cfg(feature = "trace")]
        let _span = info_span!("intersect_bvh").entered();
        let mut node = &bvh.nodes[0];
        let mut stack = Vec::with_capacity(64);
        let mut best_hit: Option<Hit> = None;
        loop {
            if node.is_leaf() {
                for i in 0..node.tri_count {
                    let tri_index = bvh.triangle_indexs[(node.left_first + i) as usize];
                    if let Some(hit) =
                        self.intersect_triangle(&bvh.tris[tri_index], tri_index, entity)
                    {
                        if let Some(best) = best_hit {
                            if best.distance < hit.distance {
                                best_hit = Some(hit);
                            }
                        } else {
                            best_hit = Some(hit);
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

            let mut dist1 = self.aabb_intersection_at(&child1.aabb);
            let mut dist2 = self.aabb_intersection_at(&child2.aabb);

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

    fn intersect_bvh_instance(&self, bvh_instance: &BvhInstance, bvhs: &[Bvh]) -> Option<Hit> {
        #[cfg(feature = "trace")]
        let _span = info_span!("intersect_bvh_instance").entered();

        let world_to = bvh_instance.inv_trans;
        let ray = RayCast3d::new(
            world_to.transform_point3a(self.origin),
            Dir3A::new_unchecked(world_to.transform_vector3a(self.direction.as_vec3a())),
            self.max,
        );

        let bvh = &bvhs[bvh_instance.bvh_index];
        ray.intersect_bvh(bvh, bvh_instance.entity)
    }

    fn intersect_tlas(&self, tlas: &Tlas) -> Option<Hit> {
        // clone the ray so we can update max distance as we find hits to tighten our search,
        // more complex the scene the big the performance win
        let mut ray = self.clone();

        if tlas.tlas_nodes.is_empty() || tlas.blas.is_empty() {
            return None;
        }
        let mut stack = Vec::<&TlasNode>::with_capacity(64);
        let mut node = &tlas.tlas_nodes[0];
        let mut best_hit: Option<Hit> = None;

        loop {
            if node.is_leaf() {
                if let Some(hit) =
                    ray.intersect_bvh_instance(&tlas.blas[node.blas as usize], &tlas.bvhs)
                {
                    if let Some(best) = best_hit {
                        if best.distance < hit.distance {
                            best_hit = Some(hit);
                            ray.max = hit.distance; // tighten the ray
                        }
                    } else {
                        best_hit = Some(hit);
                        ray.max = hit.distance; // tighten the ray
                    }
                }
                if stack.is_empty() {
                    break;
                } else {
                    node = stack.pop().unwrap();
                }
                continue;
            }
            let mut child1 = &tlas.tlas_nodes[(node.left_right & 0xffff) as usize];
            let mut child2 = &tlas.tlas_nodes[(node.left_right >> 16) as usize];
            let mut dist1 = ray.aabb_intersection_at(&child1.aabb);
            let mut dist2 = ray.aabb_intersection_at(&child2.aabb);
            if dist1.unwrap_or(f32::MAX) > dist2.unwrap_or(f32::MAX) {
                swap(&mut dist1, &mut dist2);
                swap(&mut child1, &mut child2);
            }
            if dist1.is_none() {
                if stack.is_empty() {
                    break;
                } else {
                    node = stack.pop().unwrap();
                }
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
