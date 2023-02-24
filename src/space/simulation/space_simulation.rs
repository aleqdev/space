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
    pub percision_table: Mutex<HashMap<usize, f64, nohash_hasher::BuildNoHashHasher<usize>>>,
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
        fn calculate_accerelation(
            p1: DVec3,
            p2: DVec3,
            mass1: f64,
            mass2: f64,
            delta: f64,
            g: f64,
        ) -> (DVec3, DVec3) {
            let distance2 = p1.distance_squared(p2);
            let direction = (p1 - p2).normalize();

            let a1 = direction / distance2 * mass2 * delta * g;
            let a2 = direction / distance2 * mass1 * delta * g;

            (a1, a2)
        }

        fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
            mutex.lock().unwrap()
        }

        fn last<T>(vec: &Vec<T>) -> &T {
            vec.last().unwrap()
        }

        fn last_mut<T>(vec: &mut Vec<T>) -> &mut T {
            vec.last_mut().unwrap()
        }

        let get_percision_entry = |i, j| {
            *self
                .percision_table
                .lock()
                .unwrap()
                .get(&Self::make_percision_key(i, j))
                .unwrap_or(&f64::INFINITY)
        };

        use itertools::Itertools;
        use rayon::prelude::*;

        let depth_power_limit = 4;
        let delta_iteration_count = 2u32.pow(depth_power_limit - 1);
        let mut iteration_count = delta_iteration_count;
        let smallest_step = AtomicU32::new(iteration_count);

        let positions_interpol = self
            .bodies
            .positions()
            .iter()
            .map(|x| Mutex::new(vec![*x]))
            .collect::<Vec<_>>();
        let velocities_interpol = self
            .bodies
            .velocities()
            .iter()
            .map(|x| Mutex::new(vec![*x, *x]))
            .collect::<Vec<_>>();
        let interpol_depths = Vec::from_iter(
            std::iter::from_fn(|| {
                Some([
                    AtomicU32::new(iteration_count),
                    AtomicU32::new(iteration_count),
                ])
            })
            .take(positions_interpol.len()),
        );
        let mut interpol_depths_switch = 0;

        let range = self.get_range();

        let combinations = range.clone().combinations(2).collect_vec();

        while iteration_count > 0 {
            let flip_interpol_depths_switch = |interpol_depths_switch: &mut usize| {
                *interpol_depths_switch = 1 - *interpol_depths_switch;
            };

            let get_interpol_depth = |index: usize| {
                interpol_depths[index][interpol_depths_switch].load(Ordering::SeqCst)
            };

            let depths_equal = |d1: u32, d2: u32| d1 == d2;

            let depth_is_ticking = |d: u32| iteration_count % d == 0;

            let get_depth_delta = |d: u32| d as f64 / delta_iteration_count as f64;

            let opposite_interpol_depths_switch = || 1 - interpol_depths_switch;

            combinations
                .par_iter()
                .map(|f| [f[0], f[1]])
                .for_each(|[i, j]| {
                    let (d1, d2) = (get_interpol_depth(i), get_interpol_depth(j));

                    if depths_equal(d1, d2) && depth_is_ticking(d1) && depth_is_ticking(d2) {
                        // Run the usual simulation

                        let (p1, p2) = (
                                *last(&lock(&positions_interpol[i])),
                            *last(&lock(&positions_interpol[j])),
                        );

                        let percision_entry = get_percision_entry(i, j);

                        let (a1, a2) = calculate_accerelation(
                                p1,
                            p2,
                            self.bodies.masses()[i],
                            self.bodies.masses()[j],
                            delta,
                            self.G,
                        );

                        for (mut vel, d, index) in [(-a1, d1, i), (a2, d2, j)] {
                            vel *= get_depth_delta(d);
                            if vel.length() > percision_entry {
                                interpol_depths[index][opposite_interpol_depths_switch()]
                                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                                        let new_value = value / 2;
                                        smallest_step.fetch_min(new_value, Ordering::SeqCst);
                                        Some(new_value)
                                    })
                                    .unwrap();
                            } else {
                                *last_mut(&mut lock(&velocities_interpol[index])) += vel;
                            }
                        }

                        let v_sum = a1.length() + a2.length();

                        lock(&self.percision_table).insert(Self::make_percision_key(i, j), v_sum);
                    } else if !(depth_is_ticking(d1) || depth_is_ticking(d2)) {
                        // Do nothing
                    } else {
                        // Interpolate

                        info!("INTERPOLATE");

                        let ((main, _), (interpolable, interpolable_d)) = if d1 < d2 {
                            ((i, d1), (j, d2))
                        } else {
                            ((j, d2), (i, d1))
                        };

                        let (p1, (p2_last, p2_before_last)) =
                            (*last(&lock(&positions_interpol[main])), {
                                let v = lock(&positions_interpol[interpolable]);
                                (v[v.len() - 1], v[v.len() - 2])
                            });

                        let p2 = p2_before_last.lerp(
                            p2_last,
                            (interpolable_d - (iteration_count % interpolable_d)) as f64
                                / interpolable_d as f64,
                        );

                        let (a1, _) = calculate_accerelation(
                            p1,
                            p2,
                            self.bodies.masses()[main],
                            self.bodies.masses()[interpolable],
                            delta,
                            self.G,
                        );

                        *last_mut(&mut lock(&velocities_interpol[main])) += a1;
                    }
                });

            range.clone().into_par_iter().for_each(|i| {
                let mut v_vec = lock(&velocities_interpol[i]);
                let mut p_vec = lock(&positions_interpol[i]);
                let last_vel = *last(&v_vec);
                let last_pos = *last(&p_vec) + last_vel * delta;
                v_vec.push(last_vel);
                p_vec.push(last_pos);
            });

            iteration_count -= smallest_step.load(Ordering::SeqCst);
            flip_interpol_depths_switch(&mut interpol_depths_switch);
        }

        self.bodies.positions = positions_interpol
            .iter()
            .map(|vec| last(&lock(&vec)).clone())
            .collect();
        self.bodies.velocities = velocities_interpol
            .iter()
            .map(|vec| last(&lock(&vec)).clone())
            .collect();

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
