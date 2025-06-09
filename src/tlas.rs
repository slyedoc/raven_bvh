use bevy::{math::bounding::Aabb3d, prelude::*};

use crate::{ aabb::Aabb3dExt, Bvh, BvhInstance};

#[derive(Debug, Copy, Clone)]
pub struct TlasNode {
    pub aabb: Aabb3d,
    pub left_right: u32, // 2x16 bits    
    pub blas: u32,
}

impl Default for TlasNode {
    fn default() -> Self {
        TlasNode {
            aabb: Aabb3d::init(),
            left_right: 0,
            blas: 0,
        }
    }
}

impl TlasNode {
    pub fn is_leaf(&self) -> bool {
        self.left_right == 0
    }
}

#[derive(Debug, Resource)]
pub struct Tlas {
    pub tlas_nodes: Vec<TlasNode>,    
    pub blas: Vec<BvhInstance>,
    pub bvhs: Vec<Bvh>,
}

impl Default for Tlas {
    fn default() -> Self {
        Tlas {
            tlas_nodes: Vec::with_capacity(0),
            blas: Default::default(),
            bvhs: Default::default(),
        }
    }
}

impl Tlas {
    pub fn triangle_count(&self) -> usize {
        self.bvhs.iter().map(|bvh| bvh.tris.len()).sum()
    }

    pub fn add_bvh(&mut self, bvh: Bvh) -> usize {
        self.bvhs.push(bvh);
        self.bvhs.len() - 1
    }

    pub fn add_instance(&mut self, instnace: BvhInstance) {
        self.blas.push(instnace);
    }

    pub fn build(&mut self) {
        self.tlas_nodes = Vec::with_capacity(self.blas.len() + 1);
        // reserve root node
        self.tlas_nodes.push(TlasNode::default());

        let mut node_index = vec![0u32; self.blas.len() + 1];
        let mut node_indices = self.blas.len() as i32;
        
        // assign a TLASleaf node to each BLAS
        // and index
        for (i, b) in self.blas.iter().enumerate() {
            node_index[i] = i as u32 + 1;
            self.tlas_nodes.push(TlasNode {
                aabb: b.bounds,
                left_right: 0, // is leaf
                blas: i as u32,
            });
        }

        // use agglomerative clustering to build the TLAS
        let mut a = 0i32;
        let mut b = self.find_best_match(&node_index, node_indices, a);
        while node_indices > 1 {
            let c = self.find_best_match(&node_index, node_indices, b);
            if a == c {
                let node_index_a = node_index[a as usize];
                let node_index_b = node_index[b as usize];
                let node_a = &self.tlas_nodes[node_index_a as usize];
                let node_b = &self.tlas_nodes[node_index_b as usize];
                self.tlas_nodes.push(TlasNode {
                    aabb: Aabb3d {
                        min: node_a.aabb.min.min(node_b.aabb.min),
                        max: node_a.aabb.max.max(node_b.aabb.max),
                    },
                    left_right: node_index_a + (node_index_b << 16),
                    blas: 0,
                });
                node_index[a as usize] = self.tlas_nodes.len() as u32 - 1;
                node_index[b as usize] = node_index[node_indices as usize - 1];
                node_indices -= 1;
                b = self.find_best_match(&node_index, node_indices, a);
            } else {
                a = b;
                b = c;
            }
        }
        self.tlas_nodes[0] = self.tlas_nodes[node_index[a as usize] as usize];
    }

    pub fn find_best_match(&self, list: &[u32], n: i32, a: i32) -> i32 {
        let mut smallest = 1e30f32;
        let mut best_b = -1i32;
        for b in 0..n {
            if b != a {
                let bmax = self.tlas_nodes[list[a as usize] as usize]
                    .aabb.max
                    .max(self.tlas_nodes[list[b as usize] as usize].aabb.max);
                let bmin = self.tlas_nodes[list[a as usize] as usize]
                    .aabb.min
                    .min(self.tlas_nodes[list[b as usize] as usize].aabb.min);
                let e = bmax - bmin;
                let surface_area = e.x * e.y + e.y * e.z + e.z * e.x;
                if surface_area < smallest {
                    smallest = surface_area;
                    best_b = b;
                }
            }
        }
        best_b
    }

    pub fn update_bvh_instances(&mut self, query: &Query<&GlobalTransform>) {
        for instance in &mut self.blas {
            let bvh = &self.bvhs[instance.bvh_index];
            if let Ok(trans) = query.get(instance.entity) {
                instance.update(trans, &bvh.nodes[0]);
            }
        }
    }
}
