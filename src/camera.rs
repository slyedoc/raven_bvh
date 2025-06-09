use crate::util::TlasIntersect;
use crate::{BvhSystems};

use crate::tlas::Tlas;
use bevy::math::bounding::RayCast3d;
use bevy::{
    asset::RenderAssetUsages,
    math::vec3,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    tasks::{ComputeTaskPool, ParallelSliceMut},
};

pub struct BvhCameraPlugin;

impl Plugin for BvhCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tlas>().add_systems(
            PostUpdate,
            (
                init_camera_image,
                update_camera,
                render_camera,
                display_camera,
            )
                .chain()
                .in_set(BvhSystems::Camera)
                .after(BvhSystems::Update),
        );
    }
}

// TODO: Make this projection based
#[derive(Component)]
pub struct BvhCamera {
    pub width: u32,
    pub height: u32,
    pub origin: Vec3,
    viewport_height: f32,
    viewport_width: f32,
    lower_left_corner: Vec3,
    focus_dist: f32,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    pub samples: u32,
    pub image: Option<Handle<Image>>,
}

impl BvhCamera {
    pub fn new(width: u32, height: u32) -> Self {
        // TODO: after messing the params I am defualting more
        let vfov: f32 = 45.0; // vertical field of view
        let focus_dist: f32 = 1.0; // TODO: not using this yet
        let samples: u32 = 1;

        let aspect_ratio = width as f32 / height as f32;
        let theta = vfov * std::f32::consts::PI / 180.0;
        let half_height = (theta / 2.0).tan();
        let viewport_height = 2.0 * half_height;
        let viewport_width = aspect_ratio * viewport_height;

        Self {
            width,
            height,
            viewport_height,
            viewport_width,
            focus_dist,
            samples,
            // Rest will be updated every frame for now
            origin: Vec3::ZERO,
            lower_left_corner: Vec3::ZERO,
            horizontal: Vec3::ZERO,
            vertical: Vec3::ZERO,
            u: Vec3::ZERO,
            v: Vec3::ZERO,
            w: Vec3::ONE,
            image: None,
        }
    }

    pub fn update(&mut self, trans: &Transform) {        
        self.origin = trans.translation;

        self.w = -trans.forward().as_vec3();
        self.u = trans.right().as_vec3();
        self.v = trans.up().as_vec3();

        self.horizontal = self.focus_dist * self.viewport_width * self.u;
        self.vertical = self.focus_dist * self.viewport_height * self.v;

        self.lower_left_corner =
            self.origin - self.horizontal / 2.0 - self.vertical / 2.0 - self.focus_dist * self.w;
    }

    pub fn get_ray(&self, u: f32, v: f32) -> RayCast3d {
        let direction = (self.lower_left_corner + u * self.horizontal + v * self.vertical
            - self.origin)
            .normalize();
        RayCast3d::new(self.origin, Dir3A::new_unchecked(direction.into()), 1e30f32) 
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

pub fn update_camera(mut camera_query: Query<(&mut BvhCamera, &GlobalTransform)>) {
    for (mut camera, trans) in camera_query.iter_mut() {
        camera.update(&trans.compute_transform());
    }
}

pub fn render_camera(
    camera: Single<&BvhCamera>,
    mut images: ResMut<Assets<Image>>,
    tlas: Res<Tlas>,
) {
    if let Some(image) = &camera.image {
        let image = images.get_mut(image).unwrap();

        // TODO: Make this acutally tilings, currenty this just takes a slice pixels in a row
        const PIXEL_TILE_COUNT: usize = 64;
        const PIXEL_TILE: usize = 4 * PIXEL_TILE_COUNT;

        if let Some(data) = &mut image.data {
            data.par_chunk_map_mut(ComputeTaskPool::get(), PIXEL_TILE, |i, pixels| {
                for pixel_offset in 0..(pixels.len() / 4) {
                    let index = i * PIXEL_TILE_COUNT + pixel_offset;
                    let offset = pixel_offset * 4;

                    let x = index as u32 % camera.width;
                    let y = index as u32 / camera.width;
                    let u = x as f32 / camera.width as f32;
                    let v = y as f32 / camera.height as f32;
                    // TODO: Revisit multiple samples later
                    // if samples > 0 {
                    //     u += rng.gen::<f32>() / camera.width as f32;
                    //     v += rng.gen::<f32>() / camera.height as f32;
                    // }                    
                    let ray = camera.get_ray(u, 1.0 - v);
                    let color = if let Some(hit) = ray.intersect_tlas(&tlas) {
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


pub fn display_camera(mut commands: Commands, camera: Query<&BvhCamera, Added<BvhCamera>>) {
    for camera in camera.iter() {
        if let Some(image) = &camera.image {
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
                    (Name::new("Title"), Text::new("BVH Render")),
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
                    )
                ],
            ));
        }
    }
}