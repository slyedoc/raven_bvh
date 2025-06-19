use bevy::prelude::*;

use crate::bvh::*;

use crate::tlas::*;

/// Marker to convert mesh3d's mesh to a bvh
#[derive(Component)]
pub struct SpawnBvh;

/// add MeshBvh component to Mesh3d entities that have SpawnMeshBvh

pub(crate) fn spawn_bvh(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut bvhs: ResMut<Assets<Bvh>>,
    query: Query<(Entity, &Mesh3d), With<SpawnBvh>>,
) {
    for (e, handle) in query.iter() {
        let mesh = meshes.get(handle).expect("Mesh not found");
        let bvh = bvhs.add(Bvh::from(mesh));
        commands
            .entity(e)
            .insert(
                MeshBvh(bvh)
            )
            .remove::<SpawnBvh>();
    }
}

/// Marker to convert mesh3d's mesh to a bvh and add it a tlas
#[derive(Component)]
pub struct SpawnBvhForTlas(pub Entity); // the tlas target

pub(crate) fn spawn_bvh_for_tlas(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut bvhs: ResMut<Assets<Bvh>>,
    query: Query<(Entity, &Mesh3d, &SpawnBvhForTlas)>,
) {
    for (e, handle, spawn) in query.iter() {
        let mesh = meshes.get(handle).expect("Mesh not found");
        let bvh = bvhs.add(Bvh::from(mesh));
        commands
            .entity(e)
            .insert((
                TlasTarget(spawn.0),
                MeshBvh(bvh)
            ))
            .remove::<SpawnBvhForTlas>();
    }
}

/// Added to SceneRoot to add Bvhs from Meshes in scene and add them to a tlas
#[derive(Component)]
pub struct SpawnSceneBvhForTlas(pub Entity);

/// add MeshBvh components to all Mesh3d children of SceneRoot

pub(crate) fn spawn_scene_bvh_for_tlas(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut bvhs: ResMut<Assets<Bvh>>,
    query: Query<(Entity, &SceneRoot, &SpawnSceneBvhForTlas)>,
    children: Query<(Entity, Option<&Children>, Option<&Mesh3d>)>,
    server: Res<AssetServer>,
    mut stack: Local<Vec<Entity>>,
) {
    for (root, scene, spawn) in query.iter() {
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
                commands.entity(e)
                    .insert((
                        MeshBvh(bvh),
                        TlasTarget(spawn.0)
                    ));
            }
        }

        commands.entity(root).remove::<SpawnSceneBvhForTlas>();
    }
}
