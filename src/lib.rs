use bevy::{math::vec3, prelude::*, transform::TransformSystem};

mod aabb;
mod bvh;
use bvh::*;
#[cfg(feature = "camera")]
mod camera;
mod ray;
mod tlas;
use tlas::*;
mod tri;
use tri::*;

pub mod prelude {
    #[cfg(feature = "camera")]
    pub use crate::camera::*;
    pub use crate::{
        BvhMesh, BvhPlugin, BvhScene, BvhSystems, bvh::*, ray::*, tlas::*, tri::*,
    };
}

const BIN_COUNT: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum BvhSystems {
    Update,
    #[cfg(feature = "camera")]
    Camera,
}

pub struct BvhPlugin;

impl Plugin for BvhPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tlas>().add_systems(
            PostUpdate,
            (spawn_bvh_mesh, spawn_bvh_scene, update_bvh, update_tlas)
                .chain()
                .in_set(BvhSystems::Update)
                .after(TransformSystem::TransformPropagate),
        );

        #[cfg(feature = "camera")]
        app.add_plugins(camera::BvhCameraPlugin);
    }
}

fn spawn_bvh_mesh(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    query: Query<(Entity, &Mesh3d), With<BvhMesh>>,
    mut tlas: ResMut<Tlas>,
) {
    for (e, handle) in query.iter() {
        let mesh = meshes.get(handle).expect("Mesh not found");
        let tris = parse_mesh(mesh);
        let bvh_index = tlas.add_bvh(Bvh::new(tris));
        tlas.add_instance(BvhInstance::new(e, bvh_index));
        commands.entity(e).remove::<BvhMesh>();
    }
}

fn spawn_bvh_scene(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    query: Query<(Entity, &SceneRoot), With<BvhScene>>,
    children: Query<(Entity, Option<&Children>, Option<&Mesh3d>)>,
    server: Res<AssetServer>,
    mut tlas: ResMut<Tlas>,
) {
    for (root, scene) in query.iter() {
        if let Some(load_state) = server.get_load_state(scene.0.id()) {
            if load_state.is_loading() {
                continue;
            }
        }

        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            let (e, opt_children, opt_mesh) = children.get(e).unwrap();
            if let Some(children) = opt_children {
                for child in children.iter() {
                    stack.push(child);
                }
            }
            if let Some(h_mesh) = opt_mesh {
                let mesh = meshes.get(h_mesh).expect("Mesh not found");
                let tris = parse_mesh(mesh);
                let bvh_index = tlas.add_bvh(Bvh::new(tris));
                tlas.add_instance(BvhInstance::new(e, bvh_index));
            }
        }

        commands.entity(root).remove::<BvhScene>();
    }
}

// TODO: both of these update system are incomplete, for now we are rebuilding every frame
// for now working on speeding up ray intersection
// will come back to this
pub fn update_bvh(query: Query<&GlobalTransform>, mut tlas: ResMut<Tlas>) {
    // moved fn into tlas self to since it needed 2 mutable refs within the tlas
    tlas.update_bvh_instances(&query);
}

pub fn update_tlas(mut tlas: ResMut<Tlas>) {
    tlas.build();
}

// Added to Entity with Mesh3d to enable parsing once mesh is loaded, will be removed after parsing
#[derive(Component)]
pub struct BvhMesh;
#[derive(Component)]

/// Added to SceneRoot to enable parsing scene once loaded, will be removed after parsing
#[require(SceneRoot)]
pub struct BvhScene;

// TODO: We dont really want to copy the all tris, find better way
pub fn parse_mesh(mesh: &Mesh) -> Vec<Tri> {
    use bevy::render::mesh::Indices;
    match mesh.primitive_topology() {
        bevy::render::mesh::PrimitiveTopology::TriangleList => {
            let indexes = match mesh.indices().expect("Mesh should have Indices") {
                Indices::U32(vec) => vec,
                Indices::U16(vec) => &vec.iter().map(|i| *i as u32).collect::<Vec<_>>(),
            };

            let verts = match mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .expect("Mesh should have Position Attribute")
            {
                bevy::render::mesh::VertexAttributeValues::Float32x3(vec) => {
                    vec.iter().map(|vec| vec3(vec[0], vec[1], vec[2]))
                }
                _ => todo!(),
            }
            .collect::<Vec<_>>();

            let mut triangles = Vec::with_capacity(indexes.len() / 3);
            for tri_indexes in indexes.chunks(3) {
                let v0 = verts[tri_indexes[0] as usize];
                let v1 = verts[tri_indexes[1] as usize];
                let v2 = verts[tri_indexes[2] as usize];
                triangles.push(Tri::new(
                    vec3(v0[0], v0[1], v0[2]),
                    vec3(v1[0], v1[1], v1[2]),
                    vec3(v2[0], v2[1], v2[2]),
                ));
            }
            triangles
        }
        _ => todo!(),
    }
}
