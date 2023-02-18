use std::{collections::HashMap, ops::Range};

use bevy::{math::DVec3, prelude::*};

use super::SpaceSimulationParams;

pub enum SpaceSimulationStepResult {
    Success,
    PercisionIssue,
}

#[derive(Default, soa_derive::StructOfArray)]
pub struct SpaceBody {
    pub position: DVec3,
    pub velocity: DVec3,
    pub mass: f64,
    pub radius: f64,
    pub name: String
}

#[allow(non_snake_case)]
#[derive(Resource)]
pub struct SpaceSimulation {
    pub bodies: SpaceBodyVec,
    pub time: f64,
    pub percision_table: HashMap<usize, f64, nohash_hasher::BuildNoHashHasher<usize>>,
    pub G: f64,
}

impl SpaceSimulation {
    pub fn make_percision_key(i: usize, j: usize) -> usize {
        (i << 32) | j
    }

    pub fn get_range(&self) -> Range<usize> {
        0..self.bodies.len()
    }

    pub fn take_step_smooth(&mut self, mut delta: f64) {
        let mut try_count = 20;
        let mut step_count = 1;

        let original_positions = self.bodies.position.clone();
        let original_velocities = self.bodies.velocity.clone();

        'mainloop: while try_count > 0 {
            for _ in 0..step_count {
                match self.take_step(delta) {
                    SpaceSimulationStepResult::Success => {}
                    SpaceSimulationStepResult::PercisionIssue => {
                        delta /= 4.0;
                        try_count -= 1;
                        step_count *= 4;
                        self.bodies.position = original_positions.clone();
                        self.bodies.velocity = original_velocities.clone();
                        continue 'mainloop;
                    }
                }
            }
            break 'mainloop;
        }

        self.time += delta;
    }

    pub fn take_step(&mut self, delta: f64) -> SpaceSimulationStepResult {
        use itertools::Itertools;

        let range = self.get_range();

        for [i, j] in range.clone().combinations(2).map(|f| [f[0], f[1]]) {
            let (p1, p2) = (self.bodies.position[i], self.bodies.position[j]);

            let distance = p1.distance(p2);

            let percision_entry = *self
                .percision_table
                .get(&Self::make_percision_key(i, j))
                .unwrap_or(&0.0)
                * delta;

            if distance < percision_entry {
                // info!("PERCISION ISSUE DELTA {delta} when distance is {distance} and entry is {percision_entry}",);
                return SpaceSimulationStepResult::PercisionIssue;
            }

            let distance2 = distance.powi(2);
            let direction = (p1 - p2).normalize();

            self.bodies.velocity[i] -= direction / distance2 * self.bodies.mass[j] * delta * self.G;
            self.bodies.velocity[j] += direction / distance2 * self.bodies.mass[i] * delta * self.G;

            let v_sum = self.bodies.velocity[i].length() + self.bodies.velocity[j].length();
            // info!("{distance} - {} : {}", v_sum * delta, delta);
            self.percision_table
                .insert(Self::make_percision_key(i, j), v_sum);
        }

        for i in range {
            self.bodies.position[i] += self.bodies.velocity[i] * delta;
        }

        SpaceSimulationStepResult::Success
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

    #[allow(non_snake_case, dead_code)]
    pub fn print_simulation_energy(simulation: ResMut<SpaceSimulation>) {
        use itertools::*;
        use rug::ops::Pow;

        let mut total = rug::Float::with_val(54, 0);
        let range = simulation.get_range();
        for i in range.clone() {
            let mut K = rug::Float::with_val(54, simulation.bodies.velocity[i].length());
            K = K.pow(2);
            K *= simulation.bodies.mass[i];
            K *= 0.5;
            total += K;
        }
        for [i, j] in range.combinations(2).map(|f| [f[0], f[1]]) {
            let mut P = rug::Float::with_val(54, -simulation.G);
            P *= simulation.bodies.mass[i];
            P *= simulation.bodies.mass[j];
            P /= simulation.bodies.position[i].distance(simulation.bodies.position[j]);
            total += P;
        }

        info!("Simulation total energy : {}", total)
    }
}
