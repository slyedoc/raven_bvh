mod helpers;
use helpers::*;

use bevy::{color::palettes::tailwind, prelude::*};
use raven_bvh::prelude::*;

use crate::helpers::camera_free::CameraFree;

// !!!!!!!!!!
// TODO: This is broken, we dont scale distnce at some point

// Example using BvhInitWithChildren for a scene load
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
    commands.spawn((
        Name::new("Main Camera"),
        CameraFree, // Helper to move the camera around with WASD and mouse look with right mouse button
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        BvhCamera::new(256, 256),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.5, 0.0)),
    ));

    /// ground
    commands.spawn((
        Name::new("Ground"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: tailwind::GREEN_900.into(),
            ..default()
        })),
        SpawnMeshBvh, // This Marker will have our mesh added
    ));

    /// This is to test when our Transform has odd scaling
    commands.spawn((
        Name::new("Clock Tower"),
        Transform::from_xyz(0.0, 4.0, -10.0).with_scale(Vec3::splat(0.001)), // scale it to miniture size
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/clock-tower/scene.glb")),
        ),
        // This marker tells the BVH system to build nested children
        // for this entity, the handle is used to wait till asset is loaded
        SpawnSceneBvhs,
    ));
}
