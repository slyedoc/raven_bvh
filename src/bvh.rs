use crate::{BIN_COUNT, aabb::Aabb3dExt};
use bevy::{math::bounding::Aabb3d, prelude::*, render::mesh::*};

/// Note: we really want this to be 32 bytes, so things layout in on nice 64 bytes pages in memory, using Vec3A instead of Vec3 in
/// aabb, puts us at 48, instead of 32
// pub struct Aabb {
//     pub min: Vec3,
//     pub max: Vec3,
// }

/// A BVH node, which is a node in the bounding volume hierarchy (BVH).
#[derive(Debug, Clone, Copy)]
pub struct BvhNode {
    pub aabb: Aabb3d,
    pub left_first: u32,
    pub tri_count: u32,
}

// TODO: makes this more rusty
//  pub enum BvhNodeType {
//      Leaf { tri_count: u32},
//      Branch {
//          left_first: u32,
//          // right will be left_first + 1
//      },
//  }

impl Default for BvhNode {
    fn default() -> Self {
        BvhNode {
            aabb: Aabb3d::init(),
            left_first: 0,
            tri_count: 0,
        }
    }
}

impl BvhNode {
    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.tri_count > 0
    }

    #[inline]
    pub fn calculate_cost(&self) -> f32 {
        self.tri_count as f32 * self.aabb.area()
    }
}

/// A handle to a BVH asset
#[derive(Component, Default, Clone, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct MeshBvh(pub Handle<Bvh>);

/// Bounded Volume Hierarchy (BVH) spatial data structure used for efficient ray casting
#[derive(Asset, Default, TypePath, Debug)]
pub struct Bvh {
    pub nodes: Vec<BvhNode>,
    pub tris: Vec<Tri>,
    pub triangle_indexs: Vec<usize>,
}

impl From<&Mesh> for Bvh {
    fn from(mesh: &Mesh) -> Self {
        let triangles = match mesh.primitive_topology() {
            PrimitiveTopology::TriangleList => {
                let indexes = match mesh.indices().expect("Mesh should have Indices") {
                    Indices::U32(vec) => &vec.iter().map(|i| *i as usize).collect::<Vec<_>>(),
                    Indices::U16(vec) => &vec.iter().map(|i| *i as usize).collect::<Vec<_>>(),
                };
                let verts = match mesh
                    .attribute(Mesh::ATTRIBUTE_POSITION)
                    .expect("Mesh should have Position Attribute")
                {
                    VertexAttributeValues::Float32x3(vec) => {
                        vec.iter().map(|vec| vec3a(vec[0], vec[1], vec[2]))
                    }
                    _ => todo!(),
                }
                .collect::<Vec<_>>();

                let mut triangles = Vec::with_capacity(indexes.len() / 3);
                for tri_indexes in indexes.chunks(3) {
                    triangles.push(Tri::new(
                        verts[tri_indexes[0]],
                        verts[tri_indexes[1]],
                        verts[tri_indexes[2]],
                    ));
                }
                triangles
            }
            _ => unimplemented!(),
        };
        Self::new(triangles)
    }
}

impl Bvh {
    pub fn new(triangles: Vec<Tri>) -> Bvh {
        let count = triangles.len() as u32;
        let mut nodes = Vec::with_capacity(64);

        // reserve a root node
        nodes.push(BvhNode {
            left_first: 0,
            tri_count: count,
            aabb: Aabb3d::init(),
        });

        // Note: Due to no longer being 32 bytes, we can no longer add dummy nodes to align the tree to 64 bytes
        // nodes.push(BvhNode {
        //     left_first: 0,
        //     tri_count: 0,
        //     aabb: Aabb3d::init(),
        // });

        let mut bvh = Bvh {
            tris: triangles,
            nodes,
            triangle_indexs: (0..count as usize).collect::<Vec<_>>(),
        };

        // build the BVH
        bvh.update_node_bounds(0);
        bvh.subdivide_node(0);
        bvh
    }

    // pub fn refit(&mut self, triangles: &[Tri]) {
    //     for i in (0..(self.open_node - 1)).rev() {
    //         if i != 1 {
    //             let node = &mut self.nodes[i];
    //             if node.is_leaf() {
    //                 // leaf node: adjust bounds to contained triangles
    //                 self.update_node_bounds(i, triangles);
    //                 continue;
    //             }
    //             // interior node: adjust bounds to child node bounds

    //             let leftChild = &self.nodes[node.left_first as usize];
    //             let rightChild = &self.nodes[(node.left_first + 1) as usize];

    //             node.aabb_min = leftChild.aabb_min.min(rightChild.aabb_min);
    //             node.aabb_max = leftChild.aabb_max.max(rightChild.aabb_max);
    //         }
    //     }
    // }

    fn update_node_bounds(&mut self, node_idx: usize) {
        let node = &mut self.nodes[node_idx];
        node.aabb.min = Vec3A::splat(1e30f32);
        node.aabb.max = Vec3A::splat(-1e30f32);
        for i in 0..node.tri_count {
            let leaf_tri_index = self.triangle_indexs[(node.left_first + i) as usize];
            let leaf_tri = self.tris[leaf_tri_index];
            node.aabb.expand(leaf_tri.vertex0);
            node.aabb.expand(leaf_tri.vertex1);
            node.aabb.expand(leaf_tri.vertex2);
        }
    }

    fn subdivide_node(&mut self, node_idx: usize) {
        let node = &self.nodes[node_idx];

        // determine split axis using SAH
        let (axis, split_pos, split_cost) = self.find_best_split_plane(node);
        let nosplit_cost = node.calculate_cost();
        if split_cost >= nosplit_cost {
            return;
        }

        // in-place partition
        let mut i = node.left_first;
        let mut j = i + node.tri_count - 1;
        while i <= j {
            if self.tris[self.triangle_indexs[i as usize]].centroid[axis] < split_pos {
                i += 1;
            } else {
                self.triangle_indexs.swap(i as usize, j as usize);
                j -= 1;
            }
        }

        // abort split if one of the sides is empty
        let left_count = i - node.left_first;
        if left_count == 0 || left_count == node.tri_count {
            return;
        }

        // create child nodes
        self.nodes.push(BvhNode::default());
        let left_child_idx = self.nodes.len() as u32 - 1;
        self.nodes.push(BvhNode::default());
        let right_child_idx = self.nodes.len() as u32 - 1;

        self.nodes[left_child_idx as usize].left_first = self.nodes[node_idx].left_first;
        self.nodes[left_child_idx as usize].tri_count = left_count;
        self.nodes[right_child_idx as usize].left_first = i;
        self.nodes[right_child_idx as usize].tri_count =
            self.nodes[node_idx].tri_count - left_count;

        self.nodes[node_idx].left_first = left_child_idx;
        self.nodes[node_idx].tri_count = 0;

        self.update_node_bounds(left_child_idx as usize);
        self.update_node_bounds(right_child_idx as usize);

        // recurse
        self.subdivide_node(left_child_idx as usize);
        self.subdivide_node(right_child_idx as usize);
    }

    fn find_best_split_plane(&self, node: &BvhNode) -> (usize, f32, f32) {
        // determine split axis using SAH
        let mut best_axis = 0;
        let mut split_pos = 0.0f32;
        let mut best_cost = 1e30f32;

        for a in 0..3 {
            let mut bounds_min = 1e30f32;
            let mut bounds_max = -1e30f32;
            for i in 0..node.tri_count {
                let triangle = &self.tris[self.triangle_indexs[(node.left_first + i) as usize]];
                bounds_min = bounds_min.min(triangle.centroid[a]);
                bounds_max = bounds_max.max(triangle.centroid[a]);
            }
            if bounds_min == bounds_max {
                continue;
            }
            // populate bins
            let mut bin = [Bin::default(); BIN_COUNT];
            let mut scale = BIN_COUNT as f32 / (bounds_max - bounds_min);
            for i in 0..node.tri_count {
                let triangle = &self.tris[self.triangle_indexs[(node.left_first + i) as usize]];
                let bin_idx =
                    (BIN_COUNT - 1).min(((triangle.centroid[a] - bounds_min) * scale) as usize);
                bin[bin_idx].tri_count += 1;
                bin[bin_idx].bounds.expand(triangle.vertex0);
                bin[bin_idx].bounds.expand(triangle.vertex1);
                bin[bin_idx].bounds.expand(triangle.vertex2);
            }

            // gather data for the BINS - 1 planes between the bins
            let mut left_area = [0.0f32; BIN_COUNT - 1];
            let mut right_area = [0.0f32; BIN_COUNT - 1];
            let mut left_count = [0u32; BIN_COUNT - 1];
            let mut right_count = [0u32; BIN_COUNT - 1];
            let mut left_box = Aabb3d::init();
            let mut right_box = Aabb3d::init();
            let mut left_sum = 0u32;
            let mut right_sum = 0u32;
            for i in 0..(BIN_COUNT - 1) {
                left_sum += bin[i].tri_count;
                left_count[i] = left_sum;
                left_box.expand_aabb(&bin[i].bounds);
                left_area[i] = left_box.area();
                right_sum += bin[BIN_COUNT - 1 - i].tri_count;
                right_count[BIN_COUNT - 2 - i] = right_sum;
                right_box.expand_aabb(&bin[BIN_COUNT - 1 - i].bounds);
                right_area[BIN_COUNT - 2 - i] = right_box.area();
            }

            // calculate SAH cost for the 7 planes
            scale = (bounds_max - bounds_min) / BIN_COUNT as f32;
            for i in 0..BIN_COUNT - 1 {
                let plane_cost =
                    left_count[i] as f32 * left_area[i] + right_count[i] as f32 * right_area[i];
                if plane_cost < best_cost {
                    best_axis = a;
                    split_pos = bounds_min + scale * (i + 1) as f32;
                    best_cost = plane_cost;
                }
            }
        }
        (best_axis, split_pos, best_cost)
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Tri {
    pub vertex0: Vec3A,
    pub vertex1: Vec3A,
    pub vertex2: Vec3A,
    pub centroid: Vec3A,
}

impl Tri {
    pub fn new(v0: Vec3A, v1: Vec3A, v2: Vec3A) -> Self {
        Tri {
            vertex0: v0,
            vertex1: v1,
            vertex2: v2,
            centroid: (v0 + v1 + v2) / 3.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Bin {
    bounds: Aabb3d,
    tri_count: u32,
}

impl Default for Bin {
    fn default() -> Self {
        Bin {
            bounds: Aabb3d::init(),
            tri_count: 0,
        }
    }
}
