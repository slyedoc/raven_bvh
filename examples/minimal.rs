// Minimal example of using raven_bvh without bevy
// generates a image based on ray tracing a random triangle scene

mod helpers;
#[cfg(feature = "trace")]
use bevy::log::info_span;
use rand::prelude::*;
use rand_chacha::{rand_core::SeedableRng, ChaChaRng};

use std::{f32::consts::PI, time::Instant};
use bevy::{
    
    ecs::entity::Entity, math::{vec3, vec3a, Quat, Vec3, Vec3A}, prelude::GlobalTransform, transform::components::Transform, utils::default
};
use raven_bvh::prelude::*;
use image::{Rgb, RgbImage};

fn main() {
    
    println!("BvhNode size: {}", std::mem::size_of::<BvhNode>());
    println!("TlasNode size: {}", std::mem::size_of::<TlasNode>());
    
    let build_time = Instant::now();
    let tlas = build_random_tri_scene();
    println!("Tlas build time: {:?}", build_time.elapsed());

    // render out the scene in a few sizes    
    for size in [256, 512] {
        let render_time = Instant::now();
        #[cfg(feature = "trace")]
        let _span = info_span!("render image - {}", size).entered();

        let mut camera = BvhCamera::new(size, size);
        // Bench: update camera with trans, since we dont get updated by a service here
        camera.update(&Transform {
            translation: vec3(0.0, 40.0, 100.0),
            rotation: Quat::from_axis_angle(Vec3::X, -PI / 6.0),
            ..Default::default()
        });

        let mut img = RgbImage::new(camera.width, camera.height);
        for y in 0..camera.height {
            for x in 0..camera.width {
                let ray = camera.get_ray(
                    x as f32 / camera.width as f32,                    
                    y as f32 / camera.height as f32,
                );
                let color = if let Some(hit) = ray.intersect_tlas(&tlas) {
                    let c = vec3(hit.u, hit.v, 1.0 - (hit.u + hit.v)) * 255.0;
                    Rgb([c.x as u8, c.y as u8, c.z as u8])
                } else {
                    Rgb([0, 0, 0])
                };

                img[(x, camera.height - 1 - y)] = color;
            }
            
        }
        println!("Render time {}x{}: {:?}", camera.width, camera.height, render_time.elapsed());

        img.save(format!("tmp/img_{}x{}.png", size, size)).unwrap();
    }
}



pub fn build_random_tri_scene() -> Tlas {
    #[cfg(feature = "trace")]
    let _span = info_span!("build_random_tri_scene").entered();

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
    let mut tlas = Tlas::default();
    let enity_count = 100;
    let tri_per_entity = 1000;
    // create a scene
    let side_count = (enity_count as f32).sqrt().ceil() as u32;
    let offset = 12.0;
    let side_offset = side_count as f32 * offset * 0.5;
    for i in 0..side_count {
        for j in 0..side_count {
            let id = i * side_count + j;
            let tris = gen_random_triangles(tri_per_entity, 4.0, &mut rng);
            let bvh_index = tlas.add_bvh(Bvh::new(tris));
            let e = Entity::from_raw(id);
            let mut blas = BvhInstance::new(e, bvh_index);

            // Bench: Go ahead and update the bvh instance, since we dont get updated by a service here
            blas.update(
                &GlobalTransform::from(Transform { 
                    translation: vec3(
                        i as f32 * offset - side_offset + (offset * 0.5),
                        0.0,
                        j as f32 * offset - side_offset + (offset * 0.5),
                    ),
                    ..default()
                }),
                &tlas.bvhs[blas.bvh_index].nodes[0],
            );

            // Add to tlas
            tlas.add_instance(blas);
        }
    }
    // Bench: Build the tlas, since we dont get updated by a service here
    tlas.build();
    tlas
}
