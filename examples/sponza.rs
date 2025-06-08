mod helpers;
use bevy::{color::palettes::tailwind, prelude::*, window::PresentMode};
use helpers::*;
use raven_bvh::prelude::*;

use crate::helpers::camera_free::CameraFree;

// Example using BvhInitWithChildren for a scene load
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
        BvhCamera::new(256, 256),
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // for (position, size, roughness, color) in [
    //     (vec3(-3.0, 1.0, 0.0), 2.0, 1.0, tailwind::YELLOW_400),
    //     (vec3(3.0, 1.0, 0.0), 2.0, 0.0, tailwind::BLUE_400),
    // ] {
    //     commands.spawn((
    //         Name::new("Target"),
    //         Transform::from_translation(position),
    //         Mesh3d(meshes.add(Sphere { radius: size }.mesh())),
    //         MeshMaterial3d(materials.add(StandardMaterial {
    //             base_color: color.into(),
    //             perceptual_roughness: roughness,
    //             ..default()
    //         })),
    //         BvhInit,
    //     ));
    // }

    commands.spawn((
        Name::new("Sponza"),
        Transform::from_xyz(0.0, 1.0, 0.0),
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/sponza/sponza.gltf")),
        ),
        // This marker tells the BVH system to build nested children
        // for this entity, the handle is used to wait till asset is loaded
        BvhInitWithChildren,
    ));
}
