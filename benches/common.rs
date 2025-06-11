use bevy::{color::palettes::tailwind, prelude::*, render::mesh::MeshPlugin};
use raven_bvh::prelude::*;

#[test]
fn camera() {
    // Setup app
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        AssetPlugin::default(),
        ImagePlugin::default(),
        MeshPlugin,
        //AssetPlugin::default(),
        BvhPlugin,
    ))
    .add_systems(Startup, setup);

    // Setup test entities
    let camera_id = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            Camera {
                hdr: true,
                ..default()
            },
            Transform::from_xyz(0.0, 2.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            BvhCamera::new(256, 256),
        ))
        .id();

    // Run systems
    app.update();
    app.update();

    // Check resulting changes
    let image = {
        let handle = app
            .world()
            .get::<BvhCamera>(camera_id)
            .expect("Camera image not found")
            .image
            .clone()
            .expect("Image not found");
        app.world()
            .resource::<Assets<Image>>()
            .get(&handle)
            .expect("Camera image asset not found")
            .clone()
            .try_into_dynamic()
            .expect("Failed to convert image to dynamic")
    };
    let file_path = "tmp/bevy.png";
    image.save(file_path).unwrap();
    println!("Camera image saved to: {}", file_path);

    //assert_eq!(app.world().get::<Enemy>(enemy_id).unwrap().hit_points, 4);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
        // MeshMaterial3d(materials.add(StandardMaterial {
        //     base_color: tailwind::GREEN_900.into(),
        //     ..default()
        // })),
        SpawnMeshBvh, // This Marker will have our mesh added
    ));

    //     commands.spawn((
    //     Name::new("Box"),
    //     Transform::from_xyz(1., 1., 0.),
    //     Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0).mesh())),
    //     // MeshMaterial3d(materials.add(StandardMaterial {
    //     //     base_color: tailwind::GRAY_700.into(),
    //     //     ..default()
    //     // })),
    //     SpawnMeshBvh,
    // ));

    for (position, size, _color) in [
        (vec3(-3.0, 1.0, 0.0), 2.0, tailwind::YELLOW_400),
        (vec3(3.0, 1.0, 0.0), 2.0, tailwind::BLUE_400),
    ] {
        commands.spawn((
            Name::new("Target"),
            Transform::from_translation(position),
            Mesh3d(meshes.add(Sphere { radius: size }.mesh())),
            SpawnMeshBvh,
        ));
    }
}
