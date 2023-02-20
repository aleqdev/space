use std::{collections::HashMap, ops::Range};

use bevy::{math::DVec3, prelude::*};

use super::SpaceSimulationParams;

pub enum SpaceSimulationStepResult {
    Success,
    PercisionIssue,
}

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

        let original_positions = self.bodies.positions().clone();
        let original_velocities = self.bodies.velocities().clone();

        'mainloop: while try_count > 0 {
            for _ in 0..step_count {
                match self.take_step(delta) {
                    SpaceSimulationStepResult::Success => {}
                    SpaceSimulationStepResult::PercisionIssue => {
                        delta /= 4.0;
                        try_count -= 1;
                        step_count *= 4;
                        *self.bodies.positions_mut() = original_positions.clone();
                        *self.bodies.velocities_mut() = original_velocities.clone();
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
            let (p1, p2) = (self.bodies.positions()[i], self.bodies.positions()[j]);

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

            let a1 = direction / distance2 * self.bodies.masses()[j] * delta * self.G;
            let a2 = direction / distance2 * self.bodies.masses()[i] * delta * self.G;

            self.bodies.velocities_mut()[i] -= a1;
            self.bodies.velocities_mut()[j] += a2;

            let v_sum = self.bodies.velocities()[i].length() + self.bodies.velocities()[j].length();
            // info!("{distance} - {} : {}", v_sum * delta, delta);
            self.percision_table
                .insert(Self::make_percision_key(i, j), v_sum);
        }

        for i in range {
            let d = self.bodies.velocities()[i] * delta;
            self.bodies.positions_mut()[i] += d;
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
            let mut K = rug::Float::with_val(54, simulation.bodies.velocities()[i].length());
            K = K.pow(2);
            K *= simulation.bodies.masses()[i];
            K *= 0.5;
            total += K;
        }
        for [i, j] in range.combinations(2).map(|f| [f[0], f[1]]) {
            let mut P = rug::Float::with_val(54, -simulation.G);
            P *= simulation.bodies.masses()[i];
            P *= simulation.bodies.masses()[j];
            P /= simulation.bodies.positions()[i].distance(simulation.bodies.positions()[j]);
            total += P;
        }

        info!("Simulation total energy : {}", total)
    }
}
