use std::{f32::consts::E, mem::swap};

use bevy::{
    ecs::system::{lifetimeless::Read, SystemParam}, math::bounding::{Aabb3d, BoundingVolume, RayCast3d}, prelude::*
};

use crate::{aabb::Aabb3dExt, bvh::{BvhNode, MeshBvh}, util::{Hit, RayCastExt}, Bvh};

/// Note: we really want this to be 32 bytes, so things layout in on nice 64 bytes pages in memory, using Vec3A instead of Vec3 in
/// aabb, puts us at 48, instead of 32, need to test this impact more
// pub struct Aabb {
//     pub min: Vec3,
//     pub max: Vec3,
// }

// pub struct TlasNodeTest {
//     pub aabb: Aabb,
//     pub left_right: u32, // 2x16 bits
//     pub blas: u32,
// }

/// A TLAS node, which is a node in the top-level acceleration structure (TLAS).
#[derive(Debug, Copy, Clone)]
pub enum TNType {    
    Leaf(Entity), // Entity that this node represents, usually a BLAS
    Branch {
        left_right: u32,  // index of left child in TLAS nodes
    },
}


#[derive(Debug, Copy, Clone)]
pub struct TlasNode {
    pub aabb: Aabb3d,
    pub node_type: TNType,    
}

// TODO: This is left in a invade state, 
impl Default for TlasNode {
    fn default() -> Self {
        TlasNode {
            aabb: Aabb3d::init(),
            node_type: TNType::Branch { left_right: 0 },
        }
    }
}

impl TlasNode {
    pub fn is_leaf(&self) -> bool {
        matches!(self.node_type, TNType::Leaf { .. })
    }
}

#[derive(Debug, Default, Resource)]
pub struct Tlas {
    pub tlas_nodes: Vec<TlasNode>,
    //pub blas: Vec<BvhInstance>,
    //pub bvhs: Vec<Bvh>,
}


/// A TLAS is a top-level acceleration structure that contains instances of bottom-level acceleration structures (BLAS).
impl Tlas {
    /// Total triangle count in all bvhs known to the TLAS.
    pub fn triangle_count(&self) -> usize {
        0
        //self.bvhs.iter().map(|bvh| bvh.tris.len()).sum()
    }

    // pub fn add_bvh(&mut self, bvh: Bvh) -> usize {
    //     self.bvhs.push(bvh);
    //     self.bvhs.len() - 1
    // }

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
    pub hits: Local<'s, Vec<(f32, Entity)>>,
    //     // pub output: Local<'s, Vec<(Entity, RayNavHit)>>,
    //     // pub culled_list: Local<'s, Vec<(FloatOrd, Entity)>>,
    pub query: Query<'w, 's, (Entity, Read<MeshBvh>, Read<GlobalTransform>)>,
    //     // pub tile_query: Query<'w, 's, (Read<Tile>, Read<TileNavMesh>, Read<GlobalTransform>)>,
    //     // pub waymap_query: Query<'w, 's, (Read<Waymap>, Read<GlobalTransform>)>,
    //     // #[cfg(feature = "debug_draw")]
    //     // pub gizmos: Gizmos<'w, 's, RavenGizmos>,
}

impl<'w, 's> TlasCast<'w, 's> {
    pub fn intersect_tlas(&self, ray: &RayCast3d) -> Option<Hit> {
        // PERF: clone the ray so we can update max distance as we find hits to tighten our search,
        // more complex the scene the bigger the performance win
        let mut ray = ray.clone();

        if self.tlas.tlas_nodes.is_empty() || self.query.iter().count() == 0 {
            return None;
        }
        let mut stack = Vec::<&TlasNode>::with_capacity(64);
        let mut node = &self.tlas.tlas_nodes[0];
        let mut best_hit: Option<Hit> = None;

        loop {
            match node.node_type {
                TNType::Leaf(e) => {
                    let (_e, mesh_bvh, global_trans) = self.query.get(e).unwrap(); 
                    let inv_trans = global_trans.affine().inverse();        
                    let local_ray = RayCast3d::new(
                        inv_trans.transform_point3a(ray.origin),
                        Dir3A::new(inv_trans.transform_vector3a(ray.direction.as_vec3a())).unwrap(),
                        ray.max, // TODO: should scale this too
                    );
                    let bvh = self.bvhs.get(&mesh_bvh.0).unwrap();
                    if let Some(hit) = local_ray.intersect_bvh(bvh, e){
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
                    if let Some(n) = stack.pop() {
                        node = n;
                    } else {
                        break;
                    }
                }
                TNType::Branch { left_right } => {
                    let mut child1 = &self.tlas.tlas_nodes[(left_right & 0xffff) as usize];
                    let mut child2 = &self.tlas.tlas_nodes[(left_right >> 16) as usize];
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
        best_hit
    }
}


#[derive(Debug)]
pub struct BvhInstance {
    pub entity: Entity,    
    pub inv_trans: Mat4,
    pub bounds: Aabb3d,
}

impl BvhInstance {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,            
            inv_trans: Mat4::default(),
            bounds: Aabb3d::init(),
        }
    }

    pub fn update(&mut self, trans: &GlobalTransform, root: &BvhNode) {
        // Update inv transfrom matrix for faster intersections
        let trans_matrix = trans.compute_matrix();
        self.inv_trans = trans_matrix.inverse();

        // calculate world-space bounds using the new matrix
        let bmin = root.aabb.min;
        let bmax = root.aabb.max;
        for i in 0..8 {
            self.bounds.expand(trans_matrix.transform_point3a(vec3a(
                if i & 1 != 0 { bmax.x } else { bmin.x },
                if i & 2 != 0 { bmax.y } else { bmin.y },
                if i & 4 != 0 { bmax.z } else { bmin.z },
            )));
        }
    }
}
