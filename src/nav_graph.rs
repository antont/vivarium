use bevy::prelude::*;
use pathfinding::prelude::astar;
use crate::config::Config;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavNodeKind {
    Ground,
    TreeBase,
    Branch,
}

#[derive(Debug, Clone)]
pub struct NavNode {
    pub position: Vec3,       // static position (for ground) or initial position (for branches)
    pub kind: NavNodeKind,
    pub entity: Option<Entity>, // for branch nodes: the tip entity to track live position
}

#[derive(Resource)]
pub struct NavGraph {
    pub nodes: Vec<NavNode>,
    pub edges: Vec<Vec<usize>>, // adjacency list
}

impl NavGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node and return its index.
    pub fn add_node(&mut self, position: Vec3, kind: NavNodeKind) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(NavNode { position, kind, entity: None });
        self.edges.push(Vec::new());
        idx
    }

    /// Add a node linked to a live entity (for wind-bent tree branches).
    pub fn add_node_with_entity(&mut self, position: Vec3, kind: NavNodeKind, entity: Entity) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(NavNode { position, kind, entity: Some(entity) });
        self.edges.push(Vec::new());
        idx
    }

    /// Get the current world position of a node, using live entity transform if available.
    pub fn node_position(&self, index: usize, transforms: &Query<&GlobalTransform>) -> Vec3 {
        let node = &self.nodes[index];
        if let Some(entity) = node.entity {
            if let Ok(gt) = transforms.get(entity) {
                return gt.translation();
            }
        }
        node.position
    }

    /// Add a bidirectional edge between two nodes.
    pub fn add_edge(&mut self, a: usize, b: usize) {
        if !self.edges[a].contains(&b) {
            self.edges[a].push(b);
        }
        if !self.edges[b].contains(&a) {
            self.edges[b].push(a);
        }
    }

    /// Build a sparse grid of ground nodes and connect them.
    pub fn build_ground_nodes(&mut self) {
        let half = Config::WORLD_HALF_SIZE;
        let spacing = Config::NAV_GROUND_SPACING;
        let ground_y = -half;

        let mut ground_indices = Vec::new();
        let mut x = -half;
        while x <= half {
            let mut z = -half;
            while z <= half {
                let idx = self.add_node(
                    Vec3::new(x, ground_y, z),
                    NavNodeKind::Ground,
                );
                ground_indices.push((idx, x, z));
                z += spacing;
            }
            x += spacing;
        }

        // Connect adjacent ground nodes (4-connected grid)
        for i in 0..ground_indices.len() {
            for j in (i + 1)..ground_indices.len() {
                let (_, x1, z1) = ground_indices[i];
                let (_, x2, z2) = ground_indices[j];
                let dx = (x1 - x2).abs();
                let dz = (z1 - z2).abs();
                if (dx <= spacing + 0.1 && dz < 0.1) || (dz <= spacing + 0.1 && dx < 0.1) {
                    self.add_edge(ground_indices[i].0, ground_indices[j].0);
                }
            }
        }

        // Connect tree base nodes to nearest ground nodes
        let tree_bases: Vec<usize> = (0..self.nodes.len())
            .filter(|&i| self.nodes[i].kind == NavNodeKind::TreeBase)
            .collect();

        for &tree_idx in &tree_bases {
            let tree_pos = self.nodes[tree_idx].position;
            // Find closest ground node
            if let Some(&closest) = ground_indices
                .iter()
                .min_by(|a, b| {
                    let da = Vec3::new(a.1, tree_pos.y, a.2).distance(tree_pos);
                    let db = Vec3::new(b.1, tree_pos.y, b.2).distance(tree_pos);
                    da.partial_cmp(&db).unwrap()
                })
            {
                self.add_edge(tree_idx, closest.0);
            }
        }
    }

    /// Find nearest nav node to a world position.
    pub fn nearest_node(&self, pos: Vec3) -> Option<usize> {
        self.nodes
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.position.distance(pos)
                    .partial_cmp(&b.position.distance(pos))
                    .unwrap()
            })
            .map(|(i, _)| i)
    }

    /// A* pathfinding between two node indices.
    pub fn find_path(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        let result = astar(
            &from,
            |&node| {
                self.edges[node]
                    .iter()
                    .map(|&neighbor| {
                        let cost = (self.nodes[node].position.distance(self.nodes[neighbor].position) * 100.0) as u32;
                        (neighbor, cost)
                    })
                    .collect::<Vec<_>>()
            },
            |&node| {
                (self.nodes[node].position.distance(self.nodes[to].position) * 100.0) as u32
            },
            |&node| node == to,
        );
        result.map(|(path, _cost)| path)
    }
}
