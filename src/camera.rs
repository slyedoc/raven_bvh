use crate::BvhSystems;

use crate::tlas::{Tlas, TlasCast};

use bevy::{
    asset::RenderAssetUsages,
    math::{bounding::RayCast3d, vec3},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    tasks::{ComputeTaskPool, ParallelSliceMut},
};

/// Not something you would use in production, but great for debugging ray casting
/// and benchmarking against [`Bvh`] and [`Tlas`].
pub struct BvhCameraPlugin;

impl Plugin for BvhCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tlas>().add_systems(
            PostUpdate,
            (init_camera_image, render_camera, camera_ui)
                .chain()
                .after(BvhSystems::Update)
                .in_set(BvhSystems::Camera),
        );
    }
}

#[derive(Component)]
pub struct BvhCamera {
    pub width: u32,
    pub height: u32,
    pub image: Option<Handle<Image>>,
}

impl BvhCamera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            image: None,
        }
    }
}

pub fn init_camera_image(
    mut query: Query<&mut BvhCamera, Added<BvhCamera>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Create image for camera to render to
    for mut camera in query.iter_mut() {
        camera.image = Some(images.add(Image::new(
            Extent3d {
                width: camera.width as u32,
                height: camera.height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![0; (camera.width * camera.height) as usize * 4],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )));
    }
}

pub fn render_camera(
    cameras: Query<(&BvhCamera, &GlobalTransform)>,
    mut images: ResMut<Assets<Image>>,
    tlas_cast: TlasCast,
) {
    for (bvh_camera, trans) in cameras.iter() {
        if let Some(image) = &bvh_camera.image {
            let image = images.get_mut(image).unwrap();

            let vfov: f32 = 45.0; // vertical field of view
            let focus_dist: f32 = 1.0; // TODO: not using this yet

            let aspect_ratio = bvh_camera.width as f32 / bvh_camera.height as f32;
            let theta = vfov * std::f32::consts::PI / 180.0;
            let half_height = (theta / 2.0).tan();
            let viewport_height = 2.0 * half_height;
            let viewport_width = aspect_ratio * viewport_height;
            let origin = trans.translation();
            let w = -trans.forward().as_vec3();
            let u = trans.right().as_vec3();
            let v = trans.up().as_vec3();

            let horizontal = focus_dist * viewport_width * u;
            let vertical = focus_dist * viewport_height * v;

            let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

            // TODO: Make this acutally tilings, currenty this just takes a slice pixels in a row
            const PIXEL_TILE_COUNT: usize = 64;
            const PIXEL_TILE: usize = 4 * PIXEL_TILE_COUNT;
            if let Some(data) = &mut image.data {
                data.par_chunk_map_mut(ComputeTaskPool::get(), PIXEL_TILE, |i, pixels| {
                    for pixel_offset in 0..(pixels.len() / 4) {
                        // generate pixel offset and ray for this pixel
                        let index = i * PIXEL_TILE_COUNT + pixel_offset;
                        let offset = pixel_offset * 4;

                        let x = index as u32 % bvh_camera.width;
                        let y = index as u32 / bvh_camera.width;

                        let u = x as f32 / bvh_camera.width as f32;
                        let v = 1.0 - (y as f32 / bvh_camera.height as f32);

                        let direction = lower_left_corner + u * horizontal + v * vertical - origin;
                        let ray =
                            RayCast3d::new(origin, Dir3A::new(direction.into()).unwrap(), 1e30f32);

                        // intersect the ray with the TLAS
                        let color = if let Some((_e, hit)) = tlas_cast.intersect_tlas(&ray) {
                            vec3(hit.u, hit.v, 1.0 - (hit.u + hit.v)) * 255.0
                        } else {
                            Vec3::ZERO
                        };

                        pixels[offset] = color.x as u8;
                        pixels[offset + 1] = color.y as u8;
                        pixels[offset + 2] = color.z as u8;
                        pixels[offset + 3] = 255;
                    }
                });
            }
        }
    }
}

pub fn camera_ui(mut commands: Commands, camera: Query<&BvhCamera, Added<BvhCamera>>) {
    for camera in camera.iter() {
        if let Some(image) = &camera.image {
            let ray_count = camera.width * camera.height;
            let ray_text = if ray_count >= 1_000_000 {
                format!("{:.1}m rays", ray_count / 1_000_000)
            } else if ray_count >= 1_000 {
                format!("{:.1}k rays", ray_count / 1_000)
            } else {
                format!("{} rays", ray_count)
            };
            commands.spawn((
                Name::new("UI - BVH"),
                Node {
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    top: Val::Px(50.0),
                    right: Val::Px(50.0),
                    ..default()
                },
                children![
                    (Name::new("Title"), Text::new("Tlas Render")),
                    (
                        Name::new("Image"),
                        Node {
                            width: Val::Px(camera.width as f32),
                            ..default()
                        },
                        ImageNode {
                            image: image.clone(),
                            //image_mode: NodeImageMode::Auto,
                            ..default()
                        }
                    ),
                    (Text::new(ray_text)),
                ],
            ));
        }
    }
}
