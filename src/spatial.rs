use bevy::prelude::*;
use std::collections::HashMap;

/// 3D spatial index for efficient neighbor queries.
#[derive(Resource, Default)]
pub struct SpatialIndex {
    cell_size: f32,
    map: HashMap<(i32, i32, i32), Vec<Entity>>,
}

impl SpatialIndex {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            map: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn insert(&mut self, entity: Entity, position: Vec3) {
        let key = self.cell_key(position);
        self.map.entry(key).or_default().push(entity);
    }

    fn cell_key(&self, pos: Vec3) -> (i32, i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }

    /// Returns all entities in the 3x3x3 neighborhood of cells around the given position.
    pub fn get_nearby(&self, position: Vec3) -> Vec<Entity> {
        let center = self.cell_key(position);
        let mut result = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let key = (center.0 + dx, center.1 + dy, center.2 + dz);
                    if let Some(entities) = self.map.get(&key) {
                        result.extend(entities);
                    }
                }
            }
        }
        result
    }
}
