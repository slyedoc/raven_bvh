// #![feature(test)]
// extern crate test;

// use std::f32::consts::PI;

// use bevy::prelude::*;
// use image::{ImageBuffer, Rgb, RgbImage};
// use rand::prelude::*;
// use rand_chacha::{ChaChaRng, rand_core::SeedableRng};
// use raven_bvh::prelude::*;
// use test::{Bencher, black_box};

// #[test]
// fn scene_1k_1024() {
//     let tlas = build_random_tri_scene(10, 100);
//     let mut camera = BvhCamera::new(1024, 1024);
//     camera.update(&Transform {
//         translation: vec3(0.0, 40.0, 100.0),
//         rotation: Quat::from_axis_angle(Vec3::X, -PI / 6.0),
//         ..Default::default()
//     });

//     let img = render_camera(&tlas, &camera);
//     img.save(format!(
//         "tmp/random_{:0}k_{}x{}.png",
//         tlas.triangle_count() / 1000,
//         camera.width, 
//         camera.height
//     ))
//     .unwrap();

//     // check against a reference image
//     let ref_load = image::load_from_memory(include_bytes!("../assets/tests/random_1k_1024x1024.png"));
//     assert!(ref_load.is_ok(), "Failed to load reference image");
//     let ref_image = ref_load.unwrap().into_rgb8();
//     assert_eq!(img, ref_image, "Rendered image does not match reference image");
// }

// #[test]
// fn scene_100k_1024() {
//     let tlas = build_random_tri_scene(100, 1000);
//     let mut camera = BvhCamera::new(1024, 1024);

//     camera.update(&Transform {
//         translation: vec3(0.0, 40.0, 100.0),
//         rotation: Quat::from_axis_angle(Vec3::X, -PI / 6.0),
//         ..Default::default()
//     });
    
//     let img = render_camera(&tlas, &camera);
//     img.save(format!(
//         "tmp/random_{:0}k_{}x{}.png",
//         tlas.triangle_count() / 1000,
//         camera.width,
//         camera.height
//     ))
//     .unwrap();

//     let ref_data = include_bytes!("../assets/tests/random_100k_1024x1024.png");
//     let ref_image = RgbImage::from_raw(camera.width, camera.height, ref_data.to_vec())
//         .expect("Failed to create image from reference bytes");
    
//     // Compare the rendered image with the reference image
//     assert_eq!(img, ref_image, "Rendered image does not match reference image");
// }

// #[bench]
// fn random_scene_1k_256(b: &mut Bencher) {
//     b.iter(|| {
//         let tlas = build_random_tri_scene(10, 100);
//         let mut camera = BvhCamera::new(256, 256);
//         // Bench: update camera with trans, since we dont get updated by a service here
//         camera.update(&Transform {
//             translation: vec3(0.0, 40.0, 100.0),
//             rotation: Quat::from_axis_angle(Vec3::X, -PI / 6.0),
//             ..Default::default()
//         });
 
//         black_box(render_camera(&tlas, &camera));
//     });
// }

// #[bench]
// pub fn random_scene_100k_256(b: &mut Bencher) {
//     b.iter(|| {
//         let tlas = build_random_tri_scene(100, 1000);
//         let mut camera = BvhCamera::new(256, 256);
//         // Bench: update camera with trans, since we dont get updated by a service here
//         camera.update(&Transform {
//             translation: vec3(0.0, 40.0, 100.0),
//             rotation: Quat::from_axis_angle(Vec3::X, -PI / 6.0),
//             ..Default::default()
//         });

//         black_box(render_camera(&tlas, &camera));
//     });
// }

// pub fn render_camera(tlas: &Tlas, camera: &BvhCamera) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
//     let mut img = RgbImage::new(camera.width, camera.height);
//     // TODO: this tiling doesnt work all resolutions, but its faster, so leaving it in for now
//     let grid_edge_divisions: u32 = camera.width / 8;
//     for grid_x in 0..grid_edge_divisions {
//         for grid_y in 0..grid_edge_divisions {
//             for u in 0..(camera.width / grid_edge_divisions) {
//                 for v in 0..(camera.height / grid_edge_divisions) {
//                     // PERF: calculating an offset 2 loops up is slower than doing it in the inter loop
//                     let x = (grid_x * camera.width / grid_edge_divisions) + u;
//                     let y = (grid_y * camera.height / grid_edge_divisions) + v;
//                     let ray = camera.get_ray(
//                         x as f32 / camera.width as f32,
//                         // TODO: image still reversed
//                         y as f32 / camera.height as f32,
//                     );
//                     let color = if let Some(hit) = ray.intersect_tlas(&tlas) {
//                         let c = vec3(hit.u, hit.v, 1.0 - (hit.u + hit.v)) * 255.0;
//                         Rgb([c.x as u8, c.y as u8, c.z as u8])
//                     } else {
//                         Rgb([0, 0, 0])
//                     };
//                     img[(x, camera.height - 1 - y)] = color;
//                 }
//             }
//         }
//     }
//     img
// }

// pub fn build_random_tri_scene(group_count: u32, tri_per_group: u32) -> Tlas {
//     fn random_vec3(rng: &mut impl Rng) -> Vec3A {
//         vec3a(
//             rng.random_range(-1.0..=1.0),
//             rng.random_range(-1.0..=1.0),
//             rng.random_range(-1.0..=1.0),
//         )
//     }

//     fn gen_random_triangles(size: u32, scale: f32, rng: &mut impl Rng) -> Vec<Tri> {
//         (0..size)
//             .map(|_| {
//                 // TODO: there should already be a random vec3 impl somewhere
//                 let r0 = random_vec3(rng);
//                 let r1 = random_vec3(rng);
//                 let r2 = random_vec3(rng);

//                 let v0 = r0 * scale;
//                 Tri::new(v0, v0 + r1, v0 + r2)
//             })
//             .collect::<Vec<_>>()
//     }

//     let mut rng = ChaChaRng::seed_from_u64(0);
//     let mut tlas = Tlas::default();

//     // create a scene
//     let side_count = (group_count as f32).sqrt().ceil() as u32;
//     let offset = 12.0;
//     let side_offset = side_count as f32 * offset * 0.5;
//     for i in 0..side_count {
//         for j in 0..side_count {
//             let id = i * side_count + j;
//             let tris = gen_random_triangles(tri_per_group, 4.0, &mut rng);
//             let bvh_index = tlas.add_bvh(Bvh::new(tris));
//             let e = Entity::from_raw(id);
//             let mut blas = BvhInstance::new(e, bvh_index);

//             // Bench: Go ahead and update the bvh instance, since we dont get updated by a service here
//             blas.update(
//                 &GlobalTransform::from(Transform {
//                     translation: vec3(
//                         i as f32 * offset - side_offset + (offset * 0.5),
//                         0.0,
//                         j as f32 * offset - side_offset + (offset * 0.5),
//                     ),
//                     ..Default::default()
//                 }),
//                 &tlas.bvhs[blas.bvh_index].nodes[0],
//             );

//             // Add to tlas
//             tlas.add_instance(blas);
//         }
//     }
//     // Bench: Build the tlas, since we dont get updated by a service here
//     tlas.build();
//     tlas
// }
