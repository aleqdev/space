use std::{
    collections::HashMap,
    ops::Range,
    sync::{
        atomic::{AtomicU32, AtomicUsize, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

use bevy::{math::DVec3, prelude::*};

use super::SpaceSimulationParams;

#[derive(Default)]
pub struct SpaceBody {
    pub position: DVec3,
    pub velocity: DVec3,
    pub mass: f64,
    pub radius: f64,
}

#[derive(Debug, Default)]
pub struct SpaceBodies {
    positions: Vec<DVec3>,
    velocities: Vec<DVec3>,
    masses: Vec<f64>,
    radiuses: Vec<f64>,
    map: bevy::utils::HashMap<String, usize>,
}

#[allow(dead_code)]
impl SpaceBodies {
    pub fn insert(&mut self, name: String, body: SpaceBody) {
        self.map.insert(name, self.len());
        self.positions.push(body.position);
        self.velocities.push(body.velocity);
        self.masses.push(body.mass);
        self.radiuses.push(body.radius);
    }

    pub fn remove(&mut self, name: impl AsRef<str>) -> Option<usize> {
        let Some(index) = self.map.remove(name.as_ref()) else { return None };

        self.positions.swap_remove(index);
        self.velocities.swap_remove(index);
        self.masses.swap_remove(index);
        self.radiuses.swap_remove(index);

        if let Some(swapped) = self.map.values_mut().max() {
            *swapped = index;
        }

        Some(index)
    }

    pub fn positions(&self) -> &Vec<DVec3> {
        &self.positions
    }

    pub fn positions_mut(&mut self) -> &mut Vec<DVec3> {
        &mut self.positions
    }

    pub fn velocities(&self) -> &Vec<DVec3> {
        &self.velocities
    }

    pub fn velocities_mut(&mut self) -> &mut Vec<DVec3> {
        &mut self.velocities
    }

    pub fn masses(&self) -> &Vec<f64> {
        &self.masses
    }

    pub fn masses_mut(&mut self) -> &mut Vec<f64> {
        &mut self.masses
    }

    pub fn radiuses(&self) -> &Vec<f64> {
        &self.radiuses
    }

    pub fn radiuses_mut(&mut self) -> &mut Vec<f64> {
        &mut self.radiuses
    }

    pub fn get_index(&self, name: impl AsRef<str>) -> usize {
        self.map[name.as_ref()]
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }
}

#[allow(non_snake_case)]
#[derive(Resource, Default)]
pub struct SpaceSimulation {
    pub bodies: SpaceBodies,
    pub time: f64,
    pub G: f64,
}

impl SpaceSimulation {
    pub fn take_step_smooth(&mut self, delta: f64) {
        fn calculate_accerelation(
            distance2: f64,
            direction: DVec3,
            mass1: f64,
            mass2: f64,
            g: f64,
        ) -> (DVec3, DVec3) {
            let a1 = direction / distance2 * mass2 * g;
            let a2 = direction / distance2 * mass1 * g;

            (a1, a2)
        }

        fn from_end<T>(vec: &Vec<T>, n: usize) -> &T {
            &vec[vec.len() - n]
        }

        fn from_end_mut<T>(vec: &mut Vec<T>, n: usize) -> &mut T {
            let index = vec.len() - n;
            &mut vec[index]
        }

        use itertools::Itertools;
        use rayon::prelude::*;

        const MAX_DEPTH_POWER: u32 = 8;

        let mut positions_lerp: Vec<_> = self.bodies.positions.iter().map(|e| vec![*e, *e]).collect();
        let mut velocities_lerp: Vec<_> = self.bodies.velocities.iter().map(|e| vec![*e, *e]).collect();

        let iteration_cap = 2u32.pow(MAX_DEPTH_POWER - 1);
        let mut iteration = iteration_cap;
        let mut step = iteration;
        let mut depths_lerp = vec![iteration; self.bodies.len()];

        while iteration > 0 {
            info!("Iter[{iteration}], Step[{step}]");
            for [i, j] in (0..self.bodies.len()).combinations(2).map(|e| [e[0], e[1]]) {
                let depth1 = depths_lerp[i];
                let depth2 = depths_lerp[j];

                let delta1 = delta * (depth1 as f64 / iteration_cap as f64);
                let delta2 = delta * (depth2 as f64 / iteration_cap as f64);

                let perc_a1 = from_end(&velocities_lerp[i], 2).length() * delta1;
                let perc_a2 = from_end(&velocities_lerp[j], 2).length() * delta2;

                let distance2 = from_end(&positions_lerp[i], 2)
                    .distance_squared(*from_end(&positions_lerp[j], 2));

                for (perc, index) in [(perc_a1, i), (perc_a2, j)] {
                    let mut factor = distance2 / perc.powi(2);

                    info!("Factor[{factor}], Distance2[{distance2}], Perc[{}]", perc.powi(2));

                    while factor < 16.0 && step > 1 {
                        if depths_lerp[index] == 1 {
                            break;
                        }
                        depths_lerp[index] /= 2;
                        step = step.min(depths_lerp[index]);
                        factor *= 2.0;
                    }
                }

                let depth1 = depths_lerp[i];
                let depth2 = depths_lerp[j];

                let delta1 = delta * (depth1 as f64 / iteration_cap as f64);
                let delta2 = delta * (depth2 as f64 / iteration_cap as f64);

                info!("Delta1[{delta1}], Delta2[{delta2}]");

                if iteration % depth1 == 0 && iteration % depth2 == 0 {
                    let direction =
                        (*from_end(&positions_lerp[i], 1) - *from_end(&positions_lerp[j], 1)).normalize();

                    let (a1, a2) = calculate_accerelation(
                        distance2,
                        direction,
                        self.bodies.masses[i],
                        self.bodies.masses[j],
                        self.G,
                    );

                    for (a, index, delta) in [(-a1, i, delta1), (a2, j, delta2)] {
                        *from_end_mut(&mut velocities_lerp[index], 1) += a * delta;
                    }
                } else if iteration % depth1 != 0 && iteration % depth2 != 0 {
                } else {
                    let (main_index, lerp_index) = if depth1 < depth2 { (i, j) } else { (j, i) };
                    let main_delta = if depth1 < depth2 { delta1 } else { delta2 };

                    let lerp_depth = depths_lerp[lerp_index];

                    let lerp = (lerp_depth - (iteration % lerp_depth)) as f64 / lerp_depth as f64;

                    let lerp_pos = from_end(&positions_lerp[lerp_index], 3)
                        .lerp(*from_end(&positions_lerp[lerp_index], 2), lerp);

                    let distance2 = from_end(&positions_lerp[main_index], 2)
                        .distance_squared(lerp_pos);

                    let direction = (lerp_pos - *from_end(&positions_lerp[i], 1)).normalize();

                    let (a1, _) = calculate_accerelation(
                        distance2,
                        direction,
                        self.bodies.masses[main_index],
                        self.bodies.masses[lerp_index],
                        self.G,
                    );

                    *from_end_mut(&mut velocities_lerp[main_index], 1) -= a1 * main_delta
                }
            }
            for i in 0..self.bodies.len() {
                let depth = depths_lerp[i];
                if iteration % depth == 0 {
                    *from_end_mut(&mut positions_lerp[i], 1) +=
                        *from_end(&velocities_lerp[i], 1) * delta * (depth as f64 / iteration_cap as f64);

                    let new_pos = *from_end(&positions_lerp[i], 1);
                    let new_vel = *from_end(&velocities_lerp[i], 1);
                    positions_lerp[i].push(new_pos);
                    velocities_lerp[i].push(new_vel);
                }
            }

            iteration -= step
        }

        for i in 0..self.bodies.len() {
            self.bodies.positions[i] = *from_end(&positions_lerp[i], 1);
            self.bodies.velocities[i] = *from_end(&velocities_lerp[i], 1);
        }
    }
}

pub mod systems {
    use super::{SpaceSimulation, SpaceSimulationParams};
    use bevy::prelude::*;

    pub fn simulation_take_step(
        time: Res<Time>,
        simulation_params: Res<SpaceSimulationParams>,
        mut simulation: ResMut<SpaceSimulation>,
    ) {
        simulation.take_step_smooth(time.delta_seconds_f64() * simulation_params.speed);
    }
}
