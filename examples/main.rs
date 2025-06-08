#![allow(warnings)]
mod helpers;
use std::f32::consts::PI;

use bevy::{color::palettes::tailwind, math::vec3, prelude::*, window::PresentMode};
//use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use helpers::*;
use raven_bvh::prelude::*;

use crate::helpers::camera_free::CameraFree;

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
        BvhCamera::new(800, 600),
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    //ground
    commands.spawn((
        Name::new("Ground"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: tailwind::GREEN_900.into(),
            ..default()
        })),
        BvhInit, // This Marker will have our mesh added
    ));

    let mesh_complexity = 3;
    for (position, size, complexity, color) in [
        (vec3(-3.0, 1.0, 0.0), 2.0, 12, tailwind::YELLOW_400),
        (vec3(3.0, 1.0, 0.0), 2.0, 12, tailwind::BLUE_400),
    ] {
        commands.spawn((
            Name::new("Target"),
            Transform::from_translation(position),
            Mesh3d(meshes.add(Sphere { radius: size }.mesh())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color.into(),
                ..default()
            })),
            BvhInit,
        ));
    }
}

// #[allow(dead_code)]
// pub fn camera_gizmo(
//     mut commands: Commands,
//     mut lines: ResMut<DebugLines>,
//     camera_query: Query<(&BvhCamera)>,
// ) {
//     if let Ok(camera) = camera_query.get_single() {
//         let start = camera.origin;
//         let duration = 0.0;

//         // Draw frustum lines
//         Ray::default();
//         for i in 0..4 {
//             let u = if i % 2 == 0 { 0.0 } else { 1.0 };
//             let v = if i < 2 { 0.0 } else { 1.0 };
//             let mut ray = camera.get_ray(u, v);
//             let end = camera.origin + (ray.direction * ray.distance);
//             lines.line(start, end, duration);
//         }
//     }
// }
