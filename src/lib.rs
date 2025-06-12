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
mod util;
use bvh::*;
#[cfg(feature = "camera")]
mod camera;
#[cfg(feature = "tlas")]
mod tlas;

mod debug;

#[cfg(feature = "tlas")]
use tlas::*;

use crate::{aabb::Aabb3dExt, debug::BvhDebugMode};

#[allow(unused_imports)]
#[cfg(feature = "debug_draw")]
use crate::debug::*;

pub mod prelude {
    #[cfg(feature = "camera")]
    pub use crate::camera::*;
    pub use crate::{
        BvhPlugin, BvhSystems, bvh::*, debug::*, util::*,
    };

    #[cfg(feature = "tlas")]
    pub use crate::tlas::*;

    #[cfg(feature = "helpers")]
    pub use crate::{ SpawnMeshBvh, SpawnSceneBvhs};
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
        app
            .init_resource::<BvhDebugMode>()
            .init_asset::<Bvh>();

        #[cfg(feature = "helpers")]
        app.add_systems(
            PostUpdate,
            (
                // Helpers to spawn BVH from Mesh3d and SceneRoot
                spawn_mesh_bvh,
                spawn_scene_bvhs,           
            )
                .chain()
                .before(BvhSystems::Update)                    
        );

        #[cfg(feature = "tlas")]
        app
            .init_resource::<Tlas>()
            .add_systems(
                PostUpdate,
                    build_tlas.in_set(BvhSystems::Update)
                    .after(TransformSystem::TransformPropagate),
            );
        
        #[cfg(feature = "debug_draw")]
        app.add_systems(PostUpdate, debug::debug_gimos.after(BvhSystems::Update));

        // Creates camera from tlas, used for testing BVH and TLAS and benchmarks
        #[cfg(feature = "camera")]
        app.add_plugins(camera::BvhCameraPlugin);
    }
}

/// Marker to convert mesh3d's mesh to a bvh
#[cfg(feature = "helpers")]
#[derive(Component)]
pub struct SpawnMeshBvh;

/// add MeshBvh component to Mesh3d entities that have SpawnMeshBvh
#[cfg(feature = "helpers")]
fn spawn_mesh_bvh(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut bvhs: ResMut<Assets<Bvh>>,
    query: Query<(Entity, &Mesh3d), With<SpawnMeshBvh>>,
) {
    for (e, handle) in query.iter() {
        let mesh = meshes.get(handle).expect("Mesh not found");
        let bvh = bvhs.add(Bvh::from(mesh));
        commands
            .entity(e)
            .insert(MeshBvh(bvh))
            .remove::<SpawnMeshBvh>();
    }
}

/// Added to SceneRoot to add Bvhs from Meshes in scene
#[cfg(feature = "helpers")]
#[derive(Component)]
pub struct SpawnSceneBvhs;

/// add MeshBvh components to all Mesh3d children of SceneRoot
#[cfg(feature = "helpers")]
fn spawn_scene_bvhs(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut bvhs: ResMut<Assets<Bvh>>,
    query: Query<(Entity, &SceneRoot), With<SpawnSceneBvhs>>,
    children: Query<(Entity, Option<&Children>, Option<&Mesh3d>)>,
    server: Res<AssetServer>,
    mut stack: Local<Vec<Entity>>,
) {
    for (root, scene) in query.iter() {
        if let Some(load_state) = server.get_load_state(scene.0.id()) {
            if load_state.is_loading() {
                continue;
            }
        }

        stack.push(root);
        while let Some(e) = stack.pop() {
            let (e, opt_children, opt_mesh) = children.get(e).unwrap();
            if let Some(children) = opt_children {
                for child in children.iter() {
                    stack.push(child);
                }
            }
            if let Some(h_mesh) = opt_mesh {
                let mesh = meshes.get(h_mesh).expect("Mesh not found");
                let bvh = bvhs.add(Bvh::from(mesh));
                commands.entity(e).insert(MeshBvh(bvh));
            }
        }

        commands.entity(root).remove::<SpawnSceneBvhs>();
    }
}

/// Builds the TLAS from the MeshBvh components in the scene
/// Should not be called every frame, but for now it for debugging purposes
#[cfg(feature = "tlas")]
pub fn build_tlas(
    mut tlas: ResMut<Tlas>,
    query: Query<(Entity, &MeshBvh, &GlobalTransform)>,
    bvhs: Res<Assets<Bvh>>,
) {
    let count = query.iter().count();
    let mut node_index = vec![0u32; count + 1];
    let mut node_indices = count as i32;

    tlas.tlas_nodes.clear();

    // reserve a root node
    tlas.tlas_nodes.push(TlasNode::default());

    // fill the tlas all the leaf nodes
    for (i, (e, b, global_trans)) in query.iter().enumerate() {
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

        node_index[i] = i as u32 + 1;
        tlas.tlas_nodes.push(TlasNode {
            aabb: world_aabb,
            node_type: TlasNodeType::Leaf(e),
        });
    }

    // use agglomerative clustering to build the TLAS
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
}
