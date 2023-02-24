use std::{collections::HashMap, ops::Range, sync::{Arc, Mutex, atomic::{AtomicUsize, AtomicU32, Ordering}}};

use bevy::{math::DVec3, prelude::*};

use super::SpaceSimulationParams;

#[derive(PartialEq, Eq)]
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

    pub fn take_step_smooth(&mut self, delta: f64) {
        use itertools::Itertools;
        use rayon::prelude::*;

        let depth_power_limit = 4;
        let mut iteration_count = 2u32.pow(depth_power_limit - 1);
        let smallest_step = AtomicU32::new(iteration_count);

        let positions_interpol = self.bodies.positions().iter().map(|x| Mutex::new(vec![*x])).collect::<Vec<_>>();
        let velocities_interpol = self.bodies.velocities().iter().map(|x| Mutex::new(vec![*x, *x])).collect::<Vec<_>>();
        let interpol_depths = Vec::from_iter(std::iter::from_fn(|| Some([AtomicU32::new(iteration_count), AtomicU32::new(iteration_count)])).take(positions_interpol.len()));
        let mut interpol_depths_switch = 0;

        let range = self.get_range();

        let combinations = range
            .clone()
            .combinations(2)
            .collect_vec();

        while iteration_count > 0 {
            combinations
            .par_iter()
            .map(|f| [f[0], f[1]])
            .for_each(|[i, j]| {
                let (d1, d2) = (interpol_depths[i][interpol_depths_switch].load(Ordering::SeqCst), interpol_depths[j][interpol_depths_switch].load(Ordering::SeqCst));

                if d1 == d2 && iteration_count % d1 == 0 && iteration_count % d2 == 0 {
                    // Run the usual simulation

                    let (p1, p2) = (*positions_interpol[i].lock().unwrap().last().unwrap(), *positions_interpol[j].lock().unwrap().last().unwrap());

                    let distance = p1.distance(p2);

                    let percision_entry = *
                    self.percision_table
                    .get(&Self::make_percision_key(i, j))
                    .unwrap_or(&f64::INFINITY)
                                          * delta;

                    let distance2 = distance.powi(2);
                    let direction = (p1 - p2).normalize();

                    let a1 = direction / distance2 * self.bodies.masses()[j] * delta * self.G;
                    let a2 = direction / distance2 * self.bodies.masses()[i] * delta * self.G;

                    for (vel, index) in [(-a1, i), (a2, j)] {
                        if vel.length() > percision_entry {
                            interpol_depths[index][1 - interpol_depths_switch].fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                                let new_value = value / 2;
                                smallest_step.fetch_min(new_value, Ordering::SeqCst);
                                Some(new_value)
                            }).unwrap();
                        } else {
                            *velocities_interpol[index].lock().unwrap().last_mut().unwrap() += vel;
                        }
                    }

                    //let v_sum =
                    //    bodies.velocities()[i].lock().unwrap().length() + bodies.velocities()[j].lock().unwrap().length();
                    // info!("{distance} - {} : {}", v_sum * delta, delta);
                    //self.percision_table
                    //    .insert(Self::make_percision_key(i, j), v_sum);
                } else if iteration_count % d1 != 0 && iteration_count % d1 != 0 {
                    // Do nothing
                } else {
                    // Interpolate

                    info!("INTERPOLATE");

                    let ((main, _), (interpolable, interpolable_d)) = if d1 < d2 {((i, d1), (j, d2))} else {((j, d2), (i, d1))};

                    let (p1, (p2_last, p2_before_last)) =
                    (*positions_interpol[main].lock().unwrap().last().unwrap(),
                    {
                        let v = positions_interpol[interpolable].lock().unwrap();
                        (v[v.len() - 1], v[v.len() - 2])
                    });

                    let p2 = p2_before_last.lerp(p2_last, (interpolable_d - (iteration_count % interpolable_d)) as f64 / interpolable_d as f64);

                    let distance = p1.distance(p2);

                    let distance2 = distance.powi(2);
                    let direction = (p1 - p2).normalize();

                    let a1 = direction / distance2 * self.bodies.masses()[interpolable] * delta * self.G;

                    *velocities_interpol[main].lock().unwrap().last_mut().unwrap() += a1;
                }
            });

            range.clone()
                .into_par_iter()
                .for_each(|i| {
                let mut v_vec = velocities_interpol[i].lock().unwrap();
                let mut p_vec = positions_interpol[i].lock().unwrap();
                let last_vel = *v_vec.last().unwrap();
                let last_pos = *p_vec.last().unwrap() + last_vel * delta;
                v_vec.push(last_vel);
                p_vec.push(last_pos);
            });

            iteration_count -= smallest_step.load(Ordering::SeqCst);
            interpol_depths_switch = 1 - interpol_depths_switch;
        }

        self.bodies.positions = positions_interpol.iter().map(|vec| vec.lock().unwrap().last().unwrap().clone()).collect();
        self.bodies.velocities = velocities_interpol.iter().map(|vec| vec.lock().unwrap().last().unwrap().clone()).collect();

        self.time += delta;
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
