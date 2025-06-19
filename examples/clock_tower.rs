mod helpers;
use helpers::*;

use bevy::{color::palettes::tailwind, prelude::*};
use raven_bvh::prelude::*;

use crate::helpers::camera_free::CameraFree;

// Example using SpawnSceneBvhs for oddly scaled gltf

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HelperPlugin, BvhPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // add tlas for camera
    let tlas_id = commands.spawn((Name::new("TLAS"), Tlas::default())).id();

    commands.spawn((
        Name::new("Main Camera"),
        CameraFree, // Helper to move the camera around with WASD and mouse look with right mouse button
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        BvhCamera::new(256, 256, tlas_id),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.5, 0.0)),
    ));

    // ground
    commands.spawn((
        Name::new("Ground"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: tailwind::GREEN_900.into(),
            ..default()
        })),
        SpawnBvh, // This will just create the bvh for the mesh
        TlasTarget(tlas_id), // Will make the tlas track this entity
                  // Could replace the last two components with for the same effect
                  // SpawnBvhForTlas(tlas_id), // This will just create the bvh for the mesh and add it to the tlas
    ));

    // Clock Tower
    commands.spawn((
        Name::new("Clock Tower"),
        Transform::from_xyz(0.0, 4.0, -10.0).with_scale(Vec3::splat(0.001)), // scale it to miniture size
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/clock-tower/scene.glb")),
        ),
        // This marker tells the BVH system to build nested children
        // for this entity, the handle is used to wait till asset is loaded
        SpawnSceneBvhForTlas(tlas_id), // This will just create the bvh for the meshes and add them to the tlas
    ));
}
