use bevy::{
    math::bounding::{Aabb3d, BoundingVolume},
    prelude::*,
};

#[cfg(feature = "debug_draw")]
use crate::{
    bvh::{Bvh, MeshBvh},
    tlas::Tlas,
};

pub fn aabb3d_global(bounding: &Aabb3d) -> GlobalTransform {
    GlobalTransform::from(
        Transform::from_translation(bounding.center().into())
            .with_scale((bounding.max - bounding.min).into()),
    )
}

pub fn aabb3d_transform(bounding: &Aabb3d, transform: &GlobalTransform) -> GlobalTransform {
    *transform
        * GlobalTransform::from(
            Transform::from_translation(bounding.center().into())
                .with_scale((bounding.max - bounding.min).into()),
        )
}

#[derive(Resource, Default, Debug)]
pub enum BvhDebugMode {
    #[default]
    Disabled,
    Bvhs,    
    Tlas,
}

#[cfg(feature = "debug_draw")]
pub fn debug_gimos(
    mut _tlas: ResMut<Tlas>,
    query: Query<(&MeshBvh, &GlobalTransform)>,
    bvhs: Res<Assets<Bvh>>,
    mut gizmos: Gizmos,
    bvh_debug: Res<BvhDebugMode>,
) {
    use bevy::color::palettes::tailwind;

    match bvh_debug.as_ref() {
        BvhDebugMode::Disabled => (),
        BvhDebugMode::Bvhs => {
            for (b, global_trans) in query.iter() {
                let bvh = bvhs.get(&b.0).expect("Bvh not found");

                for node in &bvh.nodes {
                    let color = if node.is_leaf() {
                        tailwind::GREEN_500
                    } else {
                        tailwind::YELLOW_500
                    };
                    gizmos.cuboid(aabb3d_transform(&node.aabb, global_trans), color);
                }
            }
        }
        BvhDebugMode::Tlas => {
            for node in _tlas.tlas_nodes.iter() {
                let color = if node.is_leaf() {
                    tailwind::GREEN_500
                } else {
                    tailwind::YELLOW_500
                };
                gizmos.cuboid(aabb3d_global(&node.aabb), color);
            }
        }
    }
    
    // gizmos.cuboid(
    //     aabb3d_global(&Aabb3d {
    //         min: Vec3A::splat(-1.0),
    //         max: Vec3A::splat(1.0),
    //     }),
    //     tailwind::RED_500,
    // );
}
