#![feature(test)]
extern crate test;

use bevy::{
    math::bounding::BoundingVolume,
    prelude::*,
};

mod aabb;
mod bvh;
mod util;
use bvh::*;
#[cfg(feature = "camera")]
mod camera;
mod tlas;
use tlas::*;
mod debug;

use crate::{
    debug::{BvhDebug},
};
mod tri;

pub mod prelude {
    #[cfg(feature = "camera")]
    pub use crate::camera::*;
    pub use crate::{
        BvhPlugin, BvhSystems, SpawnMeshBvh, SpawnSceneBvhs, bvh::*, debug::*, tlas::*, tri::*,
        util::*,
    };
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
        app.init_resource::<Tlas>()
            .init_resource::<BvhDebug>()
            .init_asset::<Bvh>()
            .configure_sets(
                PostUpdate,
                (BvhSystems::Update, BvhSystems::Camera)
                    .chain()
                    .after(TransformSystem::TransformPropagate)                    
            )
            .add_systems(
                PostUpdate,
                (spawn_mesh_bvh, spawn_scene_bvhs, build_tlas)
                    .chain()
                    .in_set(BvhSystems::Update),
            );

        #[cfg(feature = "debug_draw")]
        app.add_systems(PostUpdate, debug::debug_gimos.after(BvhSystems::Update));

        #[cfg(feature = "camera")]
        app.add_plugins(camera::BvhCameraPlugin);
    }
}

/// Helper to add BVH from a Mesh3d component with a marker component
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

/// Helper to MeshBvh to any Mesh3d in a SceneRoot component with a marker component
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

pub fn build_tlas(
    mut tlas: ResMut<Tlas>,
    query: Query<(Entity, &MeshBvh, &GlobalTransform)>,
    bvhs: Res<Assets<Bvh>>,
    //#[cfg(feature = "debug_draw")] mut gizmos: Gizmos,
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
        
        let aabb = bvh.nodes[0]
            .aabb
            .clone()
            .transformed_by(global_trans.translation(), global_trans.rotation());
        //let inv_trans = global_trans.affine().inverse();
        // calculate world-space aabb using global transform
        //let mut aabb = Aabb3d::init();
        // for i in 0..8 {
        //     let corner = vec3a(
        //         if i & 1 != 0 { local_aabb.max.x } else { local_aabb.min.x },
        //         if i & 2 != 0 { local_aabb.max.y } else { local_aabb.min.y },
        //         if i & 4 != 0 { local_aabb.max.z } else { local_aabb.min.z },
        //     );
        //     let world_corner = global_trans.affine().transform_point3a(corner);
        //     aabb.expand(world_corner);
        // }
        //#[cfg(feature = "debug_draw")]
        //gizmos.cuboid(aabb3d_global(&aabb), tailwind::GREEN_500);

        node_index[i] = i as u32 + 1;
        tlas.tlas_nodes.push(TlasNode {
            aabb,
            node_type: TNType::Leaf(e),
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
                node_type: TNType::Branch {
                    left_right: node_index_a + (node_index_b << 16),
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

// Added to Entity with Mesh3d to enable parsing once mesh is loaded, will be removed after parsing
#[derive(Component)]
pub struct SpawnMeshBvh;
#[derive(Component)]

/// Added to SceneRoot to enable parsing scene once loaded, will be removed after parsing
#[require(SceneRoot)]
pub struct SpawnSceneBvhs;
