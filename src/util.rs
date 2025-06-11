use crate::{
    bvh::{Bvh, MeshBvh},
    tlas::{BvhInstance, TNType, Tlas, TlasNode},
    tri::Tri,
};
use bevy::{
    ecs::system::{SystemParam, lifetimeless::Read},
    math::bounding::RayCast3d,
    prelude::*,
    ui::NodeType,
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

pub trait RayCastExt {
    fn intersect_triangle(&self, tri: &Tri, tri_index: usize, entity: Entity) -> Option<Hit>;

    //fn intersect_tlas(&self, tlas: &Tlas) -> Option<Hit>;

    fn intersect_bvh(&self, bvh: &Bvh, entity: Entity) -> Option<Hit>;

    // fn intersect_bvh_instance(&self, bvh_instance: &BvhInstance, bvhs: &[Bvh]) -> Option<Hit>;
}

impl RayCastExt for RayCast3d {
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

    fn intersect_bvh(&self, bvh: &Bvh, entity: Entity) -> Option<Hit> {
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
                        ray.intersect_triangle(&bvh.tris[tri_index], tri_index, entity)
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

    // fn intersect_tlas(&self, tlas: &Tlas) -> Option<Hit> {
    //     // PERF: clone the ray so we can update max distance as we find hits to tighten our search,
    //     // more complex the scene the bigger the performance win
    //     let mut ray = self.clone();

    //     if tlas.tlas_nodes.is_empty() || tlas.blas.is_empty() {
    //         return None;
    //     }
    //     let mut stack = Vec::<&TlasNode>::with_capacity(64);
    //     let mut node = &tlas.tlas_nodes[0];
    //     let mut best_hit: Option<Hit> = None;

    //     loop {
    //         if node.is_leaf() {
    //             if let Some(hit) =
    //                 ray.intersect_bvh_instance(&tlas.blas[node.blas as usize], &tlas.bvhs)
    //             {
    //                 if let Some(best) = best_hit {
    //                     if hit.distance < best.distance {
    //                         best_hit = Some(hit);
    //                         ray.max = hit.distance; // tighten the ray
    //                     }
    //                 } else {
    //                     best_hit = Some(hit);
    //                     ray.max = hit.distance; // tighten the ray
    //                 }
    //             }
    //             if stack.is_empty() {
    //                 break;
    //             } else {
    //                 node = stack.pop().unwrap();
    //             }
    //             continue;
    //         }
    //         let mut child1 = &tlas.tlas_nodes[(node.left_right & 0xffff) as usize];
    //         let mut child2 = &tlas.tlas_nodes[(node.left_right >> 16) as usize];
    //         let mut dist1 = ray.aabb_intersection_at(&child1.aabb);
    //         let mut dist2 = ray.aabb_intersection_at(&child2.aabb);
    //         if dist1.unwrap_or(f32::MAX) > dist2.unwrap_or(f32::MAX) {
    //             swap(&mut dist1, &mut dist2);
    //             swap(&mut child1, &mut child2);
    //         }
    //         if dist1.is_none() {
    //             if stack.is_empty() {
    //                 break;
    //             } else {
    //                 node = stack.pop().unwrap();
    //             }
    //         } else {
    //             node = child1;
    //             if dist2.is_some() {
    //                 stack.push(child2);
    //             }
    //         }
    //     }
    //     best_hit
    // }
}
