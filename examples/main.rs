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
        BvhCamera::new(256, 512),
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
        SpawnMeshBvh, // This Marker will have our mesh added
    ));

    // testing some nested meshes
    commands.spawn((
        Name::new("Box"),
        Transform::from_xyz(0., 1., 0.).with_scale(Vec3::splat(0.5)),
        children![
            (
                Name::new("Box - Inside1"),
                Transform::from_xyz(-2., 0., 0.),

                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0).mesh())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: tailwind::GRAY_700.into(),
                    ..default()
                })),
                SpawnMeshBvh,
            ),
            (
                Name::new("Box - Inside2"),
                Transform::from_xyz(2., 0., 0.),

                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0).mesh())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: tailwind::GRAY_700.into(),
                    ..default()
                })),
                SpawnMeshBvh,
            )
        ],
    ));

    // Spawn a circle of targets
    for i in 0..100 {
        let angle = i as f32 * (2.0 * PI / 10.0);
        let distance = 10.0 + (i as f32 * 0.7);
        let position = vec3(angle.cos() * distance, 1.0, angle.sin() * distance);
        let scale = (i as f32).cos().abs() * 2.0;

        commands.spawn(spawn_circle(
            i,
            position,
            Vec3::splat(scale),
            &mut meshes,
            &mut materials,
        ));
    }
}

fn spawn_circle(
    i: usize,
    pos: Vec3,
    scale: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> impl Bundle {
    ((
        Name::new(format!("Target{}", i)),
        Transform::from_translation(pos).with_scale(scale),
        Mesh3d(meshes.add(Sphere { radius: 2.0 }.mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: tailwind::AMBER_400.into(),
            ..default()
        })),
        SpawnMeshBvh,
    ))
}
