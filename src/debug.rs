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

#[derive(Resource, Default)]
pub struct BvhDebug {
    pub enabled: bool,
}

#[cfg(feature = "debug_draw")]
pub fn debug_gimos(
    mut _tlas: ResMut<Tlas>,
    query: Query<(&MeshBvh, &GlobalTransform)>,
    bvhs: Res<Assets<Bvh>>,
    mut gizmos: Gizmos,
    bvh_debug: Res<BvhDebug>,
) {
    use bevy::color::palettes::tailwind;
    for (b, global_trans) in query.iter() {
        let bvh = bvhs.get(&b.0).expect("Bvh not found");
        if bvh_debug.enabled {
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
}
