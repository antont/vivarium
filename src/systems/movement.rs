use bevy::prelude::*;
use crate::components::{Bird, Insect, Velocity};
use crate::config::Config;
use crate::wind::Wind;

pub fn movement_system(
    time: Res<Time>,
    wind: Res<Wind>,
    mut query: Query<(&Velocity, &mut Transform, Option<&Insect>, Option<&Bird>)>,
) {
    let dt = time.delta_secs();
    let wind_vec = wind.vector();

    query.par_iter_mut().for_each(|(velocity, mut transform, insect, bird)| {
        let wind_factor = if insect.is_some() {
            Config::WIND_INSECT_FACTOR
        } else if bird.is_some() {
            Config::WIND_BIRD_FACTOR
        } else {
            0.0
        };
        transform.translation += (velocity.0 + wind_vec * wind_factor) * dt;
    });
}
