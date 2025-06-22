//#![feature(test)]
//#extern crate test;

#[allow(unused_imports)]
#[cfg(feature = "debug_draw")]
use bevy::color::palettes::tailwind;
use bevy::{
    math::bounding::{Aabb3d, BoundingVolume},
    prelude::*,
};

mod aabb;
mod bvh;
mod helpers;
mod util;
use bvh::*;
#[cfg(feature = "camera")]
mod camera;
mod tlas;

mod debug;

use crate::{aabb::Aabb3dExt, debug::BvhDebugMode, tlas::*};

#[allow(unused_imports)]
#[cfg(feature = "debug_draw")]
use crate::debug::*;

pub mod prelude {
    pub use crate::{BvhPlugin, BvhSystems, bvh::*, debug::*, helpers::*, tlas::*, util::*};

    #[cfg(feature = "camera")]
    pub use crate::camera::*;
}

const BIN_COUNT: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum BvhSystems {
    Update,
    //#[cfg(feature = "camera")]
    Camera,
}

pub struct BvhPlugin;

impl Plugin for BvhPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BvhDebugMode>().init_asset::<Bvh>();

        app.add_systems(
            PostUpdate,
            (
                // Helpers to spawn BVH from Mesh3d and SceneRoot
                helpers::spawn_bvh,
                helpers::spawn_bvh_for_tlas,
                helpers::spawn_scene_bvh_for_tlas,
            )
                .before(BvhSystems::Update),                
        );

        app.add_systems(
            PostUpdate,
            build_tlas
                .in_set(BvhSystems::Update)
                .after(TransformSystem::TransformPropagate),
        )
        .register_type::<Tlas>()
        .register_type::<TlasMembers>()
        .register_type::<TlasRebuildStrategy>()
        .register_type::<TlasNodeType>();

        #[cfg(feature = "debug_draw")]
        app.add_systems(PostUpdate, debug::debug_gimos.after(BvhSystems::Update));

        // Creates camera from tlas, used for testing BVH and TLAS and benchmarks
        #[cfg(feature = "camera")]
        app.add_plugins(camera::BvhCameraPlugin);
    }
}

/// Builds the TLAS from the MeshBvh components in the scene
/// Should not be called every frame, but for now it for debugging purposes
pub fn build_tlas(
    mut tlas: Query<(&mut Tlas, &TlasMembers, &mut TlasRebuildStrategy)>,
    query: Query<(Entity, &MeshBvh, &GlobalTransform)>,
    bvhs: Res<Assets<Bvh>>,
) {
    for (mut tlas, children, mut strat) in tlas.iter_mut() {
        if let TlasRebuildStrategy::Mannual(false) = *strat {
            continue;
        }

        // clear the tlas
        tlas.tlas_nodes.clear();

        let count = children.collection().len();
        if count == 0 {
            // if there are no children, we will not build the tlas
            continue;
        }

        // reserve a root node as 0
        tlas.tlas_nodes.push(TlasNode::default());

        // fill the tlas all the leaf nodes
        for e in children.iter() {
            let (e, b, global_trans) = query.get(e).unwrap();
            let bvh = bvhs.get(&b.0).expect("Bvh not found");

            // convert the AABB to world space
            let local_aabb = bvh.nodes[0].aabb.clone(); // root node AABB

            // This would be ideal, but the scale only works if the aabb is centered local space, saidly not always the case
            // let world_aabb = local_aabb
            //     .scale_around_center(global_trans.scale())
            //     .transformed_by(global_trans.translation(), global_trans.rotation());

            // instead we will project the corners of the local AABB to world space
            let mut world_aabb = Aabb3d::init();
            for i in 0..8 {
                let corner = Vec3A::new(
                    if i & 1 == 0 {
                        local_aabb.min.x
                    } else {
                        local_aabb.max.x
                    },
                    if i & 2 == 0 {
                        local_aabb.min.y
                    } else {
                        local_aabb.max.y
                    },
                    if i & 4 == 0 {
                        local_aabb.min.z
                    } else {
                        local_aabb.max.z
                    },
                );

                let world_pos = global_trans.affine().transform_point3a(corner);
                world_aabb.expand(world_pos);
            }
            tlas.tlas_nodes.push(TlasNode {
                aabb: world_aabb,
                node_type: TlasNodeType::Leaf(e),
            });
        }

        // use agglomerative clustering to build the TLAS

        let mut node_index = (0..count as u32 + 1) // this builds a list of nodes to
            .map(|i| i + 1)
            .chain(std::iter::once(0))
            .collect::<Vec<u32>>();

        let mut node_indices = count as i32;

        let mut a = 0i32;
        let mut b = tlas.find_best_match(&node_index, node_indices, a);
        while node_indices > 1 {
            let c = tlas.find_best_match(&node_index, node_indices, b);
            if a == c {
                let node_index_a = node_index[a as usize];
                let node_index_b = node_index[b as usize];
                let node_a = tlas.tlas_nodes[node_index_a as usize];
                let node_b = tlas.tlas_nodes[node_index_b as usize];
                tlas.tlas_nodes.push(TlasNode {
                    aabb: node_a.aabb.merge(&node_b.aabb),
                    node_type: TlasNodeType::Branch {
                        left: node_index_a as u16,
                        right: node_index_b as u16,
                    },
                });
                node_index[a as usize] = tlas.tlas_nodes.len() as u32 - 1;
                node_index[b as usize] = node_index[node_indices as usize - 1];
                node_indices -= 1;
                b = tlas.find_best_match(&node_index, node_indices, a);
            } else {
                a = b;
                b = c;
            }
        }
        tlas.tlas_nodes[0] = tlas.tlas_nodes[node_index[a as usize] as usize];

        // if in manual mode clear the flag
        if let TlasRebuildStrategy::Mannual(true) = *strat {
            *strat = TlasRebuildStrategy::Mannual(false);
        }
    }
}
