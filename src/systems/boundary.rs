use bevy::prelude::*;
use crate::components::BoundaryWrap;
use crate::config::Config;

pub fn boundary_wrap_system(
    mut query: Query<&mut Transform, With<BoundaryWrap>>,
) {
    let half = Config::WORLD_HALF_SIZE;
    let size = half * 2.0;
    for mut transform in &mut query {
        let pos = &mut transform.translation;
        if pos.x > half {
            pos.x -= size;
        } else if pos.x < -half {
            pos.x += size;
        }
        if pos.y > half {
            pos.y -= size;
        } else if pos.y < -half {
            pos.y += size;
        }
        if pos.z > half {
            pos.z -= size;
        } else if pos.z < -half {
            pos.z += size;
        }
    }
}
