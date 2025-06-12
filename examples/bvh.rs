#![allow(warnings)]
mod helpers;
use std::f32::consts::PI;

use bevy::{
    color::palettes::tailwind,
    math::{bounding::RayCast3d, vec3},
    prelude::*,
};
use helpers::*;
use raven_bvh::prelude::*;

use crate::helpers::camera_free::CameraFree;

// Example using a few Bvh with no helpers or tlas

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HelperPlugin, BvhPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, ray_cast)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut bvhs: ResMut<Assets<Bvh>>,
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
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Example of addings Bvh directly from a mesh
    let ground_mesh = Plane3d::new(Vec3::Y, Vec2::splat(50.)).mesh().build();
    let ground_bvh = Bvh::from(&ground_mesh);
    commands.spawn((
        Name::new("Ground"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mesh3d(meshes.add(ground_mesh)),
        MeshBvh(bvhs.add(ground_bvh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: tailwind::GREEN_900.into(),
            ..default()
        })),
    ));

    // Like other assets, the same bvh can be uses on multiple entities
    let box_mesh = Cuboid::new(0.5, 0.5, 0.5).mesh().build();
    let box_bvh = Bvh::from(&box_mesh);
    let box_mesh_handle = meshes.add(box_mesh);
    let box_bvh_handle = bvhs.add(box_bvh);
    let mat = materials.add(StandardMaterial {
        base_color: tailwind::BLUE_500.into(),
        ..default()
    });

    for i in 0..10 {
        commands.spawn((
            Name::new(format!("Box {}", i)),
            Transform::from_xyz(i as f32, 1.0, 0.0)
                .with_scale(Vec3::splat((i as f32 * 0.1) + 0.5))
                .with_rotation(Quat::from_rotation_y(i as f32 * PI / 5.0)),
            Mesh3d(box_mesh_handle.clone()),
            MeshBvh(box_bvh_handle.clone()),
            MeshMaterial3d(mat.clone()),
        ));
    }
}

// Simple example of ray castings cursor position vs bvhs in the scene
fn ray_cast(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
    query: Query<(Entity, &MeshBvh, &GlobalTransform)>,
    mut bvhs: Res<Assets<Bvh>>,
    mut gizmos: Gizmos,
) {
    // create a ray cursor position
    let (camera, camera_trans) = *camera_query;
    let Ok(window) = window.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok(camera_ray) = camera.viewport_to_world(camera_trans, cursor_position) else {
        return;
    };
    let ray = RayCast3d::from_ray(camera_ray, 10.0);

    // test against all bvhs in the scene
    let mut best_hit: Option<Hit> = None;
    for (e, mesh_bvh, transform) in query.iter() {
        let bvh = bvhs.get(&mesh_bvh.0).expect("Bvh not found");

        // Convert the ray to local space of the mesh
        let (local_ray, dir_scale) = ray.to_local(transform);
        if let Some(mut hit) = local_ray.intersect_bvh(&bvh) {
            // Scale the distance back to world space
            hit.distance /= dir_scale;
            if let Some(best) = best_hit {
                if hit.distance < best.distance {
                    best_hit = Some(hit);
                }
            } else {
                best_hit = Some(hit);
            }
        }
    }

    // draw line of the ray
    if let Some(hit) = best_hit {
        gizmos.sphere(
            Vec3::from(ray.get_point(hit.distance)),
            0.2,
            tailwind::GREEN_500,
        );
    }
}
