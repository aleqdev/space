use std::cell::UnsafeCell;

use bevy::{math::DVec3, prelude::*};
use chrono::{DateTime, Duration, Utc};

use super::SpaceSimulationParams;

#[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct SpaceBodyRotation {
    #[serde(
        serialize_with = "serialize_rotation_as_xyz",
        deserialize_with = "deserialize_rotation_from_xyz"
    )]
    pub initial: Quat,
    pub sideral_rotation_offset: f64,
    pub sideral_rotation_speed: f64,
}

fn serialize_rotation_as_xyz<S>(quat: &Quat, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;

    let rot = quat.to_euler(EulerRot::XYZ);

    let mut seq = s.serialize_seq(Some(3))?;
    seq.serialize_element(&rot.0)?;
    seq.serialize_element(&rot.1)?;
    seq.serialize_element(&rot.2)?;
    seq.end()
}

fn deserialize_rotation_from_xyz<'de, D>(de: D) -> Result<Quat, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let de: [f32; 3] = serde::Deserialize::deserialize(de)?;
    Ok(Quat::from_euler(EulerRot::XYZ, de[0], de[1], de[2]))
}

#[derive(Default, serde::Serialize, serde::Deserialize, Clone)]
pub struct SpaceBody {
    pub position: DVec3,
    pub velocity: DVec3,
    pub mass: f64,
    pub radius: f64,
    pub rotation: SpaceBodyRotation,
}

#[derive(Debug, Default)]
pub struct SpaceBodies {
    positions: Vec<DVec3>,
    velocities: Vec<DVec3>,
    masses: Vec<f64>,
    radiuses: Vec<f64>,
    rotations: Vec<SpaceBodyRotation>,
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
        self.rotations.push(body.rotation);
    }

    pub fn remove(&mut self, name: impl AsRef<str>) -> Option<usize> {
        let Some(index) = self.map.remove(name.as_ref()) else { return None };

        self.positions.swap_remove(index);
        self.velocities.swap_remove(index);
        self.masses.swap_remove(index);
        self.radiuses.swap_remove(index);
        self.rotations.swap_remove(index);

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

    pub fn rotations(&self) -> &Vec<SpaceBodyRotation> {
        &self.rotations
    }

    pub fn rotations_mut(&mut self) -> &mut Vec<SpaceBodyRotation> {
        &mut self.rotations
    }

    pub fn get_index(&self, name: impl AsRef<str>) -> usize {
        self.map[name.as_ref()]
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }
}

#[allow(non_snake_case)]
#[derive(Resource)]
pub struct SpaceSimulation {
    pub bodies: SpaceBodies,
    pub time: DateTime<Utc>,
    pub G: f64,
}

#[allow(unused_variables)]
impl SpaceSimulation {
    pub fn calculate_body_rotation(
        &self,
        initial: &Quat,
        sideral_offset: f64,
        sideral_speed: f64,
    ) -> Quat {
        initial.mul_quat(Quat::from_rotation_y(
            (sideral_offset + sideral_speed * self.time.timestamp() as f64)
                .rem_euclid(std::f64::consts::TAU) as f32,
        ))
    }

    pub fn take_step_smooth(&mut self, percision: usize, mut delta_seconds: f64) {
        fn from_matrix_to_vector(i: usize, j: usize, size: usize) -> usize {
            if i <= j {
                return i * size - (i - 1) * i / 2 + j - i;
            } else {
                return j * size - (j - 1) * j / 2 + i - j;
            }
        }

        self.time = self
            .time
            .checked_add_signed(Duration::milliseconds((delta_seconds * 1000.0) as i64))
            .unwrap();

        use itertools::Itertools;
        use rayon::prelude::*;

        let bodies_len = self.bodies.len();

        let mut iteration = percision;
        delta_seconds /= iteration as f64;

        let forces_size = (((1 + bodies_len) as f32 / 2.0) * bodies_len as f32) as usize;

        struct Forces(UnsafeCell<Vec<f64>>);
        unsafe impl Sync for Forces {}

        let mut forces = Forces(UnsafeCell::new(vec![0.0; forces_size]));

        while iteration > 0 {
            // Fill up forces table
            (0..self.bodies.len())
                .combinations(2)
                .par_bridge()
                .map(|e| [e[0], e[1]])
                .for_each_init(
                    || &forces,
                    |forces, [i, j]| {
                        let p1 = self.bodies.positions[i];
                        let p2 = self.bodies.positions[j];

                        let force = self.G * self.bodies.masses[i] * self.bodies.masses[j]
                            / p1.distance_squared(p2);

                        let index = from_matrix_to_vector(i, j, bodies_len);

                        // SAFETY: triangular matrix has unique index for each (i,j)-pair
                        unsafe {
                            (*forces.0.get())[index] = force;
                        }
                    },
                );

            let forces = &*forces.0.get_mut();

            self.bodies
                .velocities
                .par_iter_mut()
                .enumerate()
                .for_each(|(i, vel)| {
                    let a: DVec3 = (0..bodies_len)
                        .par_bridge()
                        .map(|j| {
                            if i == j {
                                return DVec3::ZERO;
                            }

                            let p1 = self.bodies.positions[i];
                            let p2 = self.bodies.positions[j];

                            let direction = (p1 - p2).normalize();

                            let index = from_matrix_to_vector(i, j, bodies_len);

                            let force = forces[index];

                            let a = force / self.bodies.masses[i];

                            -a * direction
                        })
                        .sum();

                    *vel += a * delta_seconds
                });

            self.bodies
                .positions
                .par_iter_mut()
                .zip(self.bodies.velocities.par_iter_mut())
                .for_each(|(pos, vel)| {
                    *pos += *vel * delta_seconds;
                });

            iteration -= 1;
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
        simulation.take_step_smooth(
            simulation_params.percision,
            time.delta_seconds_f64() * simulation_params.speed,
        );
    }
}
