use std::mem::swap;

use bevy::{
    ecs::system::{SystemParam, lifetimeless::Read},
    math::bounding::{Aabb3d, BoundingVolume, RayCast3d},
    prelude::*,
};

use crate::{
    Bvh,
    aabb::Aabb3dExt,
    bvh::MeshBvh,
    util::{Hit, RayCastExt},
};

/// Note: we really want this to be 32 bytes, so things layout in on nice 64 bytes cache lines, but using Vec3A instead of Vec3 in
/// aabb, puts us at 48 instead of 32, need to test this impact more
// pub struct Aabb {
//     pub min: Vec3,
//     pub max: Vec3,
// }

/// A TLAS node, which is a node in the top-level acceleration structure (TLAS).
#[derive(Debug, Copy, Clone)]
pub enum TlasNodeType {
    Leaf(Entity),
    Branch {
        left: u16,  // index of left child in TLAS nodes
        right: u16, // index of right child in TLAS nodes
    },
}

#[derive(Debug, Copy, Clone)]
pub struct TlasNode {
    pub aabb: Aabb3d,
    pub node_type: TlasNodeType,
}

// TODO: This is left in a invade state,
impl Default for TlasNode {
    fn default() -> Self {
        TlasNode {
            aabb: Aabb3d::init(),
            node_type: TlasNodeType::Branch { left: 0, right: 0 },
        }
    }
}

impl TlasNode {
    pub fn is_leaf(&self) -> bool {
        matches!(self.node_type, TlasNodeType::Leaf { .. })
    }
}

#[derive(Debug, Default, Resource)]
pub struct Tlas {
    pub tlas_nodes: Vec<TlasNode>,
}

/// A TLAS is a top-level acceleration structure that contains instances of bottom-level acceleration structures (BLAS).
impl Tlas {
    pub fn find_best_match(&self, list: &[u32], n: i32, a: i32) -> i32 {
        let mut smallest = 1e30f32;
        let mut best_b = -1i32;
        for b in 0..n {
            if b != a {
                let node_a = &self.tlas_nodes[list[a as usize] as usize];
                let node_b = &self.tlas_nodes[list[b as usize] as usize];
                let surface_area = node_a.aabb.merge(&node_b.aabb).area();
                if surface_area < smallest {
                    smallest = surface_area;
                    best_b = b;
                }
            }
        }
        best_b
    }
}

#[derive(SystemParam)]
pub struct TlasCast<'w, 's> {
    pub tlas: Res<'w, Tlas>,
    pub bvhs: Res<'w, Assets<Bvh>>,
    pub query: Query<'w, 's, (Entity, Read<MeshBvh>, Read<GlobalTransform>)>,
}

impl<'w, 's> TlasCast<'w, 's> {
    pub fn intersect_tlas(&self, ray: &RayCast3d) -> Option<(Entity, Hit)> {
        // PERF: clone the ray so we can update max distance as we find hits to tighten our search,
        // more complex the scene the bigger the performance win
        let mut ray = ray.clone();

        if self.tlas.tlas_nodes.is_empty() || self.query.iter().count() == 0 {
            return None;
        }
        let mut stack = Vec::<&TlasNode>::with_capacity(64);
        let mut node = &self.tlas.tlas_nodes[0];
        let mut best_hit: Option<Hit> = None;
        let mut best_entity: Option<Entity> = None;

        loop {
            match node.node_type {
                TlasNodeType::Leaf(e) => {
                    let (_e, mesh_bvh, global_trans) = self.query.get(e).unwrap();
                    // convert the ray to local space of the e
                    let (local_ray, dir_scale) = ray.to_local(global_trans);

                    // test vs bvh
                    let bvh = self.bvhs.get(&mesh_bvh.0).unwrap();
                    if let Some(mut hit) = local_ray.intersect_bvh(bvh) {
                        hit.distance /= dir_scale; // Convert back to world-space distance                        
                        if let Some(best) = best_hit {
                            if hit.distance < best.distance {
                                best_hit = Some(hit);
                                best_entity = Some(e);
                                ray.max = hit.distance; // tighten the ray
                            }
                        } else {
                            best_hit = Some(hit);
                            best_entity = Some(e);
                            ray.max = hit.distance; // tighten the ray
                        }
                    }
                    if let Some(n) = stack.pop() {
                        node = n;
                    } else {
                        break;
                    }
                }
                TlasNodeType::Branch { left, right } => {
                    let mut child1 = &self.tlas.tlas_nodes[right as usize];
                    let mut child2 = &self.tlas.tlas_nodes[left as usize];
                    let mut dist1 = ray.aabb_intersection_at(&child1.aabb);
                    let mut dist2 = ray.aabb_intersection_at(&child2.aabb);
                    if dist1.unwrap_or(f32::MAX) > dist2.unwrap_or(f32::MAX) {
                        swap(&mut dist1, &mut dist2);
                        swap(&mut child1, &mut child2);
                    }
                    if dist1.is_none() {
                        if let Some(n) = stack.pop() {
                            node = n;
                        } else {
                            break;
                        }
                    } else {
                        node = child1;
                        if dist2.is_some() {
                            stack.push(child2);
                        }
                    }
                }
            }
        }
        if let Some(hit) = best_hit {
            Some((best_entity.unwrap(), hit))
        } else {
            None
        }
    }
}
