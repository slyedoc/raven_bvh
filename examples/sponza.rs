mod helpers;
use helpers::*;

use bevy::prelude::*;
use raven_bvh::prelude::*;
use crate::helpers::camera_free::CameraFree;

/// Example using [`BvhScene`] for a scene load, this is pushing the limits of this example but it works
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            HelperPlugin,
            BvhPlugin
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // camera
    commands.spawn((
        Name::new("Main Camera"),
        CameraFree, // Helper to move the camera around with WASD and mouse look with right mouse button
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        BvhCamera::new(128, 128),
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));    

    commands.spawn((
        Name::new("Sponza"),
        Transform::from_xyz(0.0, 1.0, 0.0),
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/sponza/sponza.gltf")),
        ),
        // This marker tells the BVH system to build nested children
        // for this entity, waits till asset is loaded
        BvhScene,
    ));
}
