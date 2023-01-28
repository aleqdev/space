use std::{collections::HashMap, ops::Range};

use bevy::{math::DVec3, prelude::*};
use ringbuffer::{AllocRingBuffer, RingBufferWrite};

#[derive(Resource)]
pub struct SpaceSimulationParams {
    pub speed: f64,
}
pub enum SpaceSimulationStepResult {
    Success,
    PercisionIssue,
}

#[allow(non_snake_case)]
#[derive(Resource)]
pub struct SpaceSimulation {
    pub positions: Vec<DVec3>,
    pub velocities: Vec<DVec3>,
    pub masses: Vec<f64>,
    pub time: f64,
    pub trails: Vec<AllocRingBuffer<DVec3>>,
    pub percision_table: HashMap<usize, f64, nohash_hasher::BuildNoHashHasher<usize>>,
    pub G: f64,
}

impl SpaceSimulation {
    pub fn make_percision_key(i: usize, j: usize) -> usize {
        (i << 32) | j
    }

    pub fn get_range(&self) -> Range<usize> {
        0..self.positions.len()
    }

    pub fn take_step_smooth(&mut self, mut delta: f64) {
        let mut try_count = 20;
        let mut step_count = 1;

        let original_positions = self.positions.clone();
        let original_velocities = self.velocities.clone();

        'mainloop: while try_count > 0 {
            for _ in 0..step_count {
                match self.take_step(delta) {
                    SpaceSimulationStepResult::Success => {}
                    SpaceSimulationStepResult::PercisionIssue => {
                        delta /= 2.0;
                        try_count -= 1;
                        step_count *= 2;
                        self.positions = original_positions.clone();
                        self.velocities = original_velocities.clone();
                        continue 'mainloop;
                    }
                }
            }
            break 'mainloop;
        }

        let range = self.get_range();

        for i in range {
            self.trails[i].push(self.positions[i]);
        }

        self.time += delta;
    }

    pub fn take_step(&mut self, delta: f64) -> SpaceSimulationStepResult {
        use itertools::Itertools;

        let range = self.get_range();

        for i in range.clone() {
            self.positions[i] += self.velocities[i] * delta;
        }

        for [i, j] in range.combinations(2).map(|f| [f[0], f[1]]) {
            let (p1, p2) = (self.positions[i], self.positions[j]);

            let distance = p1.distance(p2);

            let percision_entry = *self
                .percision_table
                .get(&Self::make_percision_key(i, j))
                .unwrap_or(&0.0)
                * delta;

            if distance < percision_entry {
                info!("PERCISION ISSUE DELTA {delta} when distance is {distance} and entry is {percision_entry}",);
                return SpaceSimulationStepResult::PercisionIssue;
            }

            let distance2 = distance.powi(2);
            let direction = (p1 - p2).normalize();

            let v1 = self.velocities[i];
            let v2 = self.velocities[j];
            let v_dot = v1.dot(v2);

            self.velocities[i] -= direction / distance2 * self.masses[j] * delta * self.G;
            self.velocities[j] += direction / distance2 * self.masses[i] * delta * self.G;

            if v_dot < 0.0 {
                //info!("PERCISION UPDATE");
                let v_sum = self.velocities[i].length() + self.velocities[j].length();
                self.percision_table
                    .insert(Self::make_percision_key(i, j), v_sum);
            }
        }

        SpaceSimulationStepResult::Success
    }
}

pub fn simulation_take_step(
    time: Res<Time>,
    simulation_params: Res<SpaceSimulationParams>,
    mut simulation: ResMut<SpaceSimulation>,
) {
    simulation.take_step_smooth(time.delta_seconds_f64() * simulation_params.speed);
}

pub fn print_simulation_energy(simulation: ResMut<SpaceSimulation>) {
    use itertools::*;

    let mut total = 0.0;
    let range = simulation.get_range();
    for i in range.clone() {
        total += simulation.velocities[i].length().powi(2) * simulation.masses[i] * 0.5;
    }
    for [i, j] in range.combinations(2).map(|f| [f[0], f[1]]) {
        total += -1.0
            * ((simulation.masses[i] * simulation.masses[j])
                / (simulation.positions[i] - simulation.positions[j]).length());
    }

    // info!("Simulation total energy : {total}")
}
