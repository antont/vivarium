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

    pub fn insert(&mut self, _entity: Entity, _position: Vec3) {
        // NOT IMPLEMENTED
    }

    fn cell_key(&self, pos: Vec3) -> (i32, i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }

    /// Returns all entities in the 3x3x3 neighborhood of cells around the given position.
    pub fn get_nearby(&self, _position: Vec3) -> Vec<Entity> {
        Vec::new() // NOT IMPLEMENTED
    }
}
