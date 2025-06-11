#![feature(test)]
extern crate test;

// use std::f32::consts::PI;

use std::{
    f32::consts::PI,
    hash::{DefaultHasher, Hash},
};

use bevy::{prelude::*, render::mesh::MeshPlugin};
use image::{ImageBuffer, Rgb, RgbImage};
use rand::prelude::*;
use rand_chacha::{ChaChaRng, rand_core::SeedableRng};
use raven_bvh::prelude::*;
use test::{Bencher, black_box};

#[test]
fn scene_1k_1024() {
    // Setup app
    let (mut app, camera_id) = setup_app::<10, 100, 1024>();

    // Run systems
    app.update();

    // Check resulting changes
    let image = get_image(app, camera_id);

    let file_path = "tmp/bevy_1k_1024x1024.png";
    image.save(file_path).unwrap();
    println!("Camera image saved to: {}", file_path);

    // Check against a reference image
    let mut hasher = DefaultHasher::new();
    let ref_image =
        image::load_from_memory(include_bytes!("../assets/tests/bevy_1k_1024x1024.png"))
            .unwrap()
            .into_rgb8();
    assert_eq!(
        image.hash(&mut hasher),
        ref_image.hash(&mut hasher),
        "Rendered image does not match reference image"
    );
}

#[test]
fn scene_100k_1024() {
    let (mut app, camera_id) = setup_app::<100, 1000, 1024>();
    app.update();
    let image = get_image(app, camera_id);


    let file_path = "tmp/bevy_100k_1024x1024.png";
    image.save(file_path).unwrap();
    println!("Camera image saved to: {}", file_path);

    // Check against a reference image
    let mut hasher = DefaultHasher::new();
    let ref_image =
        image::load_from_memory(include_bytes!("../assets/tests/bevy_100k_1024x1024.png"))
            .unwrap()
            .into_rgb8();
    assert_eq!(
        image.hash(&mut hasher),
        ref_image.hash(&mut hasher),
        "Rendered image does not match reference image"
    );
}



#[bench]
fn random_scene_1k_256(b: &mut Bencher) {
    b.iter(|| {
        let (mut app, camera_id) = setup_app::<10, 100, 256>();

        app.update();

        // Check resulting changes
        let image = get_image(app, camera_id);

        // let file_path = "tmp/bench_1k_256.png";
        // image.save(file_path).unwrap();
        // println!("Camera image saved to: {}", file_path);
        black_box(image);
    });
}

#[bench]
pub fn random_scene_100k_256(b: &mut Bencher) {
    b.iter(|| {
        let (mut app, camera_id) = setup_app::<100, 1000, 256>();

        app.update();

        let image = get_image(app, camera_id);

        // let file_path = "tmp/bench_100k_256.png";
        //  image.save(file_path).unwrap();
        //  println!("Camera image saved to: {}", file_path);
        black_box(image);
    });
}



fn setup_app<const GROUP_COUNT: usize, const TRI_PER_GROUP: usize, const RESOLUTION: u32>()
-> (App, Entity) {
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
    .add_systems(
        Startup,
        build_random_tri_scene::<GROUP_COUNT, TRI_PER_GROUP>,
    );

    // Setup test entities
    let camera_id = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            Camera {
                hdr: true,
                ..default()
            },
            Transform {
                translation: vec3(0.0, 40.0, 100.0),
                rotation: Quat::from_axis_angle(Vec3::X, -PI / 6.0),
                ..Default::default()
            },
            BvhCamera::new(RESOLUTION, RESOLUTION),
        ))
        .id();

    // Run systems

    (app, camera_id)
}

fn get_image(app: App, camera_id: Entity) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
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
        .to_rgb8()
    
}


/// This is odd but its from non-`bevy` code, kept here so I could benchmark vs old code
fn build_random_tri_scene<const GROUP_COUNT: usize, const TRI_PER_GROUP: usize>(
    mut commands: Commands,
    mut bvhs: ResMut<Assets<Bvh>>,
) {
    fn random_vec3(rng: &mut impl Rng) -> Vec3A {
        vec3a(
            rng.random_range(-1.0..=1.0),
            rng.random_range(-1.0..=1.0),
            rng.random_range(-1.0..=1.0),
        )
    }

    fn gen_random_triangles(size: u32, scale: f32, rng: &mut impl Rng) -> Vec<Tri> {
        (0..size)
            .map(|_| {
                // TODO: there should already be a random vec3 impl somewhere
                let r0 = random_vec3(rng);
                let r1 = random_vec3(rng);
                let r2 = random_vec3(rng);

                let v0 = r0 * scale;
                Tri::new(v0, v0 + r1, v0 + r2)
            })
            .collect::<Vec<_>>()
    }

    let mut rng = ChaChaRng::seed_from_u64(0);

    let side_count = (GROUP_COUNT as f32).sqrt().ceil() as u32;
    let offset = 12.0;
    let side_offset = side_count as f32 * offset * 0.5;
    for i in 0..side_count {
        for j in 0..side_count {
            // let id = i * side_count + j;
            let tris = gen_random_triangles(TRI_PER_GROUP as u32, 4.0, &mut rng);
            commands.spawn((
                Transform::from_xyz(
                    i as f32 * offset - side_offset + (offset * 0.5),
                    0.0,
                    j as f32 * offset - side_offset + (offset * 0.5),
                ),
                MeshBvh(bvhs.add(Bvh::new(tris))),
            ));
        }
    }
}
