use crate::space::display::StarMaterial;
use bevy::{math::DVec3, prelude::*, tasks::Task};
use chrono::{DateTime, Duration, Utc};
use surf::StatusCode;

pub async fn get_body_dynamics_using_nasa_horizons(
    date: DateTime<Utc>,
    name: impl ToString,
) -> anyhow::Result<(DVec3, DVec3, f64)> {
    let data_regex = regex::Regex::new(r"\$\$SOE(.*)\$\$EOE").unwrap();
    let coord_regex = regex::Regex::new(r"[XYZ] ?=([ \-+0-9E.]+)").unwrap();
    let radii_regex = regex::Regex::new(r"Target radii *: *([0-9\.E+\-]+)").unwrap();

    #[allow(non_snake_case)]
    #[derive(serde::Serialize)]
    struct RequestQuery {
        COMMAND: String,
        CENTER: String,
        EPHEM_TYPE: String,
        START_TIME: String,
        STOP_TIME: String,
        OBJ_DATA: String,
        QUANTITIES: String,
    }

    const MAX_RETRIES: usize = 32;

    let body_dynamics = async {
        for _ in 0..MAX_RETRIES {
            info!("Trying to get NASA body [{}]", name.to_string());

            let mut resp = surf::Client::new()
                .get("https://ssd.jpl.nasa.gov/api/horizons.api")
                .query(&RequestQuery {
                    COMMAND: name.to_string(),
                    CENTER: "geo@0".into(),
                    EPHEM_TYPE: "VECTORS".into(),
                    START_TIME: date.format("%Y-%b-%d-%T").to_string(),
                    STOP_TIME: date
                        .checked_add_signed(Duration::hours(1))
                        .unwrap()
                        .format("%Y-%b-%d-%T")
                        .to_string(),
                    OBJ_DATA: "NO".into(),
                    QUANTITIES: "1".into(),
                })
                .expect("Failed to construct NASA request")
                .send()
                .await
                .expect("Failed to send NASA request");

            if resp.status() == StatusCode::ServiceUnavailable {
                async_std::task::sleep(Duration::milliseconds((rand::random::<i64>().abs() + 1) % 1000).to_std().unwrap()).await;
                continue;
            }

            return Some(
                resp.take_body()
                    .into_string()
                    .await
                    .expect("Failed to get string contents of NASA response"),
            );
        }

        None
    };

    let body_observer = async {
        for _ in 0..MAX_RETRIES {
            info!("Trying to get NASA body [{}]", name.to_string());

            let mut resp = surf::Client::new()
                .get("https://ssd.jpl.nasa.gov/api/horizons.api")
                .query(&RequestQuery {
                    COMMAND: name.to_string(),
                    CENTER: "geo@0".into(),
                    EPHEM_TYPE: "OBSERVER".into(),
                    START_TIME: date.format("%Y-%b-%d-%T").to_string(),
                    STOP_TIME: date
                        .checked_add_signed(Duration::hours(1))
                        .unwrap()
                        .format("%Y-%b-%d-%T")
                        .to_string(),
                    OBJ_DATA: "NO".into(),
                    QUANTITIES: "1".into(),
                })
                .expect("Failed to construct NASA request")
                .send()
                .await
                .expect("Failed to send NASA request");

            if resp.status() == StatusCode::ServiceUnavailable {
                async_std::task::sleep(Duration::milliseconds((rand::random::<i64>().abs() + 1) % 1000).to_std().unwrap()).await;
                continue;
            }

            return Some(
                resp.take_body()
                    .into_string()
                    .await
                    .expect("Failed to get string contents of NASA response"),
            );
        }

        None
    };

    let (Some(result_dynamics), Some(result_observer)) = futures::join!(body_dynamics, body_observer) else {
        error!("Failed to get NASA body: Serive Unavailable");
        return Err(anyhow::anyhow!("Failed to get NASA body: Serive Unavailable"));
    };

    let parsed = data_regex
        .captures(&result_dynamics)
        .expect("Failed to regex NASA dynamics")
        .get(0)
        .expect("Regex group [0] of NASA dynamics failed")
        .as_str(); // get FROM SOE to EOE

    let mut parsed = coord_regex
        .find_iter(parsed)
        .take(6)
        .map(|x| x.as_str().split('=').nth(1).unwrap());

    let x = parsed.next().unwrap().trim().parse().unwrap();
    let z = parsed.next().unwrap().trim().parse().unwrap();
    let y = parsed.next().unwrap().trim().parse().unwrap();

    let position = DVec3::new(
        x,
        y,
        z,
    ) * 1000.0;

    let x = parsed.next().unwrap().trim().parse().unwrap();
    let z = parsed.next().unwrap().trim().parse().unwrap();
    let y = parsed.next().unwrap().trim().parse().unwrap();

    let velocity = DVec3::new(
        x,
        y,
        z,
    ) * 1000.0;

    let radii = radii_regex
        .captures(&result_observer)
        .expect("Failed to regex NASA observer")
        .get(1)
        .expect("Regex group [1] of NASA observer failed")
        .as_str()
        .to_owned()
        .parse::<f64>()
        .expect("Failed to parse NASA observer radii")
        * 1000.0;

    Ok((position, velocity, radii))
}

pub struct NasaHorizonsPlugin;

impl Plugin for NasaHorizonsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnNasaBodyRequest>();
        app.add_event::<NasaBodyAddition>();

        app.init_resource::<NasaTasksManager>();
        app.insert_resource({
            use SpaceBodyKnownDetailsMaterial::*;

            let mut map = bevy::utils::HashMap::new();

            // SUN
            map.insert(
                "10".into(),
                SpaceBodyKnownDetails {
                    mass: 1988500e24,
                    material: Star(StarMaterial {
                        primary_color: Color::rgb(8.0 * 4.0, 8.0 * 4.0, 0.0),
                        secondary_color: Color::rgb(8.0 * 4.0, 5.2 * 4.0, 0.0),
                        ..Default::default()
                    }),
                    rotation: Default::default(),
                    rotation_rate: Default::default(),
                },
            );

            // MERCURY
            map.insert(
                "199".into(),
                SpaceBodyKnownDetails {
                    mass: 3.302e23,
                    material: TexturePath("textures/mercury_base_color.jpg"),
                    rotation: Default::default(),
                    rotation_rate: 0.00000124001,
                },
            );

            // VENUS
            map.insert(
                "299".into(),
                SpaceBodyKnownDetails {
                    mass: 48.685e23,
                    material: TexturePath("textures/venus_base_color.jpg"),
                    rotation: Default::default(),
                    rotation_rate: -0.00000029924,
                },
            );

            // EARTH
            map.insert(
                "399".into(),
                SpaceBodyKnownDetails {
                    mass: 5.97219e24,
                    material: TexturePath("textures/earth_base_color.jpg"),
                    rotation: Default::default(),
                    rotation_rate: 0.00007292115,
                },
            );

            // MARS
            map.insert(
                "499".into(),
                SpaceBodyKnownDetails {
                    mass: 6.4171e23,
                    material: TexturePath("textures/mars_base_color.jpg"),
                    rotation: Default::default(),
                    rotation_rate: 0.0000708822,
                },
            );

            // JUPITER
            map.insert(
                "599".into(),
                SpaceBodyKnownDetails {
                    mass: 189818.722e22,
                    material: TexturePath("textures/jupiter_base_color.jpg"),
                    rotation: Default::default(),
                    rotation_rate: 0.00007292115,
                },
            );

            // SATURN
            map.insert(
                "699".into(),
                SpaceBodyKnownDetails {
                    mass: 5.6834e26,
                    material: TexturePath("textures/saturn_base_color.jpg"),
                    rotation: Default::default(),
                    rotation_rate: 0.0334979 / (24.0 * 60.0 * 60.0),
                },
            );

            // URANUS
            map.insert(
                "799".into(),
                SpaceBodyKnownDetails {
                    mass: 86.813e24,
                    material: TexturePath("textures/uranus_base_color.jpg"),
                    rotation: Default::default(),
                    rotation_rate: -0.000101237,
                },
            );

            // NEPTUNE
            map.insert(
                "899".into(),
                SpaceBodyKnownDetails {
                    mass: 102.409e24,
                    material: TexturePath("textures/neptune_base_color.jpg"),
                    rotation: Default::default(),
                    rotation_rate: 0.000108338,
                },
            );

            // MOON
            map.insert(
                "301".into(),
                SpaceBodyKnownDetails {
                    mass: 7.349e22,
                    material: TexturePath("textures/moon_base_color.jpg"),
                    rotation: Default::default(),
                    rotation_rate: 0.0000026617,
                },
            );

            SpaceBodiesKnownDetails { map }
        });

        app.add_system(systems::reqeust_nasa_bodies_on_event);
        app.add_system(
            systems::manage_nasa_bodies_on_response.after(systems::reqeust_nasa_bodies_on_event),
        );
    }
}
pub struct SpawnNasaBodyRequest {
    pub date: DateTime<Utc>,
    pub name: String,
}

pub struct SpawnNasaBodyResponse {
    pub date: DateTime<Utc>,
    pub name: String,
    pub position: DVec3,
    pub velocity: DVec3,
    pub radius: f64,
}

pub struct NasaBodyAddition {
    pub date: DateTime<Utc>,
    pub name: String,
    pub position: DVec3,
    pub velocity: DVec3,
    pub radius: f64,
    pub mass: f64,
    pub material: SpaceBodyKnownDetailsMaterial,
    pub rotation: Quat,
    pub rotation_rate: f64,
}

#[derive(Resource, Default)]
pub struct NasaTasksManager {
    pub tasks: Vec<Task<SpawnNasaBodyResponse>>,
}

#[derive(Clone)]
pub enum SpaceBodyKnownDetailsMaterial {
    TexturePath(&'static str),
    Star(StarMaterial),
}

pub struct SpaceBodyKnownDetails {
    pub mass: f64,
    pub material: SpaceBodyKnownDetailsMaterial,
    pub rotation: Quat,
    pub rotation_rate: f64,
}

#[derive(Resource)]
pub struct SpaceBodiesKnownDetails {
    pub map: bevy::utils::HashMap<String, SpaceBodyKnownDetails>,
}

pub mod systems {
    use bevy::{prelude::*, tasks::AsyncComputeTaskPool};

    use crate::space::{
        nasa_horizons::NasaBodyAddition,
        simulation::{SpaceBody, SpaceSimulation},
    };

    use super::{
        NasaTasksManager, SpaceBodiesKnownDetails, SpawnNasaBodyRequest, SpawnNasaBodyResponse,
    };

    pub fn reqeust_nasa_bodies_on_event(
        mut ev: EventReader<SpawnNasaBodyRequest>,
        mut manager: ResMut<NasaTasksManager>,
    ) {
        let thread_pool = AsyncComputeTaskPool::get();

        for e in ev.iter() {
            let date = e.date;
            let name = e.name.clone();
            manager.tasks.push(thread_pool.spawn(async move {
                let (position, velocity, radius) =
                    super::get_body_dynamics_using_nasa_horizons(date, &name)
                        .await
                        .unwrap();

                SpawnNasaBodyResponse {
                    date,
                    name,
                    position,
                    velocity,
                    radius,
                }
            }));
        }
    }

    pub fn manage_nasa_bodies_on_response(
        mut manager: ResMut<NasaTasksManager>,
        mut simulation: ResMut<SpaceSimulation>,
        known_details: ResMut<SpaceBodiesKnownDetails>,
        mut ev: EventWriter<NasaBodyAddition>,
    ) {
        const AVERAGE_DENSITY: f64 = 3346.4;

        use futures_lite::future;

        manager.tasks.retain_mut(|task| {
            let Some(response) = future::block_on(future::poll_once(task)) else { return true };

            let mass;
            let rotation;
            let rotation_rate;
            let material;

            if let Some(details) = known_details.map.get(&response.name) {
                mass = details.mass;
                rotation = details.rotation;
                rotation_rate = details.rotation_rate;
                material = details.material.clone();
            } else {
                mass =
                    AVERAGE_DENSITY * std::f64::consts::PI * (4.0 / 3.0) * response.radius.powi(3);
                rotation = Default::default();
                rotation_rate = Default::default();
                material = crate::space::nasa_horizons::SpaceBodyKnownDetailsMaterial::Star(
                    crate::space::display::StarMaterial {
                        primary_color: Color::WHITE,
                        secondary_color: Color::GRAY,
                        ..Default::default()
                    },
                );
            }

            simulation.bodies.insert(
                response.name.clone(),
                SpaceBody {
                    position: response.position,
                    velocity: response.velocity,
                    mass,
                    radius: response.radius,
                },
            );

            ev.send(NasaBodyAddition {
                date: response.date,
                name: response.name,
                position: response.position,
                velocity: response.velocity,
                radius: response.radius,
                mass,
                material,
                rotation,
                rotation_rate,
            });

            return false;
        });
    }
}
