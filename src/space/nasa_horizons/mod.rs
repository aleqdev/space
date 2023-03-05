use crate::space::{display::StarMaterial, simulation::SpaceBodyRotation};
use bevy::{math::DVec3, prelude::*, tasks::Task};
use bevy_debug_text_overlay::screen_print;
use chrono::{DateTime, Duration, Utc};
use surf::StatusCode;

use super::simulation::SpaceBody;

lazy_static::lazy_static! {
    static ref LIMITER: async_lock::Semaphore = {
        async_lock::Semaphore::new(4)
    };
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Deserialize)]
pub struct MassParams {
    massValue: f64,
    massExponent: f64,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Deserialize)]
pub struct BodyParams {
    mass: Option<MassParams>,
    meanRadius: f64,
    sideralRotation: f64,
}

pub async fn get_body_dynamics_using_nasa_horizons(
    date: DateTime<Utc>,
    name: impl ToString,
) -> anyhow::Result<SpaceBody> {
    use anyhow::anyhow;
    let _guard = LIMITER.acquire().await;

    let data_regex = regex::Regex::new(r"\$\$SOE(.*)\$\$EOE").unwrap();
    let coord_regex = regex::Regex::new(r"[XYZ] ?=([ \-+0-9E.]+)").unwrap();
    let name_regex = regex::Regex::new(r"Target body name *: *([a-zA-Z0-9]+)").unwrap();

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
                .ok()?
                .send()
                .await
                .ok()?;

            if resp.status() == StatusCode::ServiceUnavailable {
                async_std::task::sleep(
                    Duration::milliseconds((rand::random::<i64>().abs() + 1) % 1000)
                        .to_std()
                        .unwrap(),
                )
                .await;
                continue;
            }

            return Some(resp.take_body().into_string().await.ok()?);
        }

        None
    };

    let Some(result_dynamics) = body_dynamics.await else {
        error!("Failed to get NASA body: Serive Unavailable");
        return Err(anyhow::anyhow!("Failed to get NASA body: Serive Unavailable"));
    };

    let parsed = data_regex
        .captures(&result_dynamics)
        .ok_or_else(|| anyhow!("Failed to regex NASA dynamics: [{result_dynamics:?}]"))?
        .get(0)
        .ok_or_else(|| anyhow!("Regex group [0] of NASA dynamics failed"))?
        .as_str(); // get FROM SOE to EOE

    let mut coords = coord_regex.find_iter(parsed).take(6).map(|x| {
        x.as_str()
            .split('=')
            .nth(1)
            .ok_or_else(|| anyhow!("coord_regex fault"))
    });

    let x = coords
        .next()
        .ok_or_else(|| anyhow!("coord_regex fault"))??
        .trim()
        .parse::<f64>()?;
    let z = -coords
        .next()
        .ok_or_else(|| anyhow!("coord_regex fault"))??
        .trim()
        .parse::<f64>()?;
    let y = -coords
        .next()
        .ok_or_else(|| anyhow!("coord_regex fault"))??
        .trim()
        .parse::<f64>()?;

    let position = DVec3::new(x, y, z) * 1000.0;

    let x = coords
        .next()
        .ok_or_else(|| anyhow!("coord_regex parse fault"))??
        .trim()
        .parse::<f64>()?;
    let z = -coords
        .next()
        .ok_or_else(|| anyhow!("coord_regex parse fault"))??
        .trim()
        .parse::<f64>()?;
    let y = -coords
        .next()
        .ok_or_else(|| anyhow!("coord_regex parse fault"))??
        .trim()
        .parse::<f64>()?;

    let velocity = DVec3::new(x, y, z) * 1000.0;

    let name = name_regex
        .captures(&result_dynamics)
        .ok_or_else(|| anyhow!("name_regex fault"))?
        .get(1)
        .ok_or_else(|| anyhow!("name_regex fault"))?
        .as_str();

    let body_params = async {
        for _ in 0..MAX_RETRIES {
            let mut resp = surf::Client::new()
                .with(surf::middleware::Redirect::new(2))
                .get(format!("https://api.le-systeme-solaire.net/rest/bodies/{name}?data=meanRadius,mass,massValue,massExponent,sideralRotation"))
                .send()
                .await
                .ok()?;

            if resp.status() == StatusCode::ServiceUnavailable {
                async_std::task::sleep(
                    Duration::milliseconds((rand::random::<i64>().abs() + 1) % 1000)
                        .to_std()
                        .unwrap(),
                )
                .await;
                continue;
            }

            return Some(resp.take_body().into_json().await.ok()?);
        }

        None
    };

    let data: BodyParams = body_params
        .await
        .ok_or_else(|| anyhow!("Failed to get body info"))?;

    screen_print!(sec: 3.0, col: Color::GREEN, "got response for {}", name.to_string());

    Ok(SpaceBody {
        position,
        velocity,
        radius: data.meanRadius * 1000.0,
        mass: data
            .mass
            .map(|mass| mass.massValue * 10f64.powf(mass.massExponent))
            .unwrap_or(1.0),
        rotation: SpaceBodyRotation {
            initial: Default::default(),
            sideral_rotation_offset: Default::default(),
            sideral_rotation_speed: data.sideralRotation,
        },
    })
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
                    sideral_rotation_offset: Default::default(),
                    sideral_rotation_speed: Default::default(),
                },
            );

            // MERCURY
            map.insert(
                "199".into(),
                SpaceBodyKnownDetails {
                    mass: 3.302e23,
                    material: TexturePath("textures/mercury_base_color.jpg".into()),
                    rotation: Quat::from_euler(
                        EulerRot::XYZ,
                        28.55f32.to_radians(),
                        329.548f32.to_radians(),
                        0.0,
                    ),
                    sideral_rotation_offset: Default::default(),
                    sideral_rotation_speed: 0.00000124001,
                },
            );

            // VENUS
            map.insert(
                "299".into(),
                SpaceBodyKnownDetails {
                    mass: 48.685e23,
                    material: TexturePath("textures/venus_base_color.jpg".into()),
                    rotation: Quat::from_euler(
                        EulerRot::XYZ,
                        157.16f32.to_radians(),
                        19.8f32.to_radians(),
                        0.0,
                    ),
                    sideral_rotation_offset: Default::default(),
                    sideral_rotation_speed: -0.00000029924,
                },
            );

            // EARTH
            map.insert(
                "399".into(),
                SpaceBodyKnownDetails {
                    mass: 5.97219e24,
                    material: TexturePath("textures/earth_base_color.jpg".into()),
                    rotation: Quat::from_euler(
                        EulerRot::XYZ,
                        -23.4392911f32.to_radians(),
                        (360.0 - 280.147f32).to_radians(),
                        0.0,
                    ),
                    sideral_rotation_offset: -15445678.5462,
                    sideral_rotation_speed: 0.00007292115,
                },
            );

            // MARS
            map.insert(
                "499".into(),
                SpaceBodyKnownDetails {
                    mass: 6.4171e23,
                    material: TexturePath("textures/mars_base_color.jpg".into()),
                    rotation: Default::default(),
                    sideral_rotation_offset: Default::default(),
                    sideral_rotation_speed: 0.0000708822,
                },
            );

            // JUPITER
            map.insert(
                "599".into(),
                SpaceBodyKnownDetails {
                    mass: 189818.722e22,
                    material: TexturePath("textures/jupiter_base_color.jpg".into()),
                    rotation: Default::default(),
                    sideral_rotation_offset: Default::default(),
                    sideral_rotation_speed: 0.00007292115,
                },
            );

            // SATURN
            map.insert(
                "699".into(),
                SpaceBodyKnownDetails {
                    mass: 5.6834e26,
                    material: TexturePath("textures/saturn_base_color.jpg".into()),
                    rotation: Default::default(),
                    sideral_rotation_offset: Default::default(),
                    sideral_rotation_speed: 0.0334979 / (24.0 * 60.0 * 60.0),
                },
            );

            // URANUS
            map.insert(
                "799".into(),
                SpaceBodyKnownDetails {
                    mass: 86.813e24,
                    material: TexturePath("textures/uranus_base_color.jpg".into()),
                    rotation: Default::default(),
                    sideral_rotation_offset: Default::default(),
                    sideral_rotation_speed: -0.000101237,
                },
            );

            // NEPTUNE
            map.insert(
                "899".into(),
                SpaceBodyKnownDetails {
                    mass: 102.409e24,
                    material: TexturePath("textures/neptune_base_color.jpg".into()),
                    rotation: Default::default(),
                    sideral_rotation_offset: Default::default(),
                    sideral_rotation_speed: 0.000108338,
                },
            );

            // MOON
            map.insert(
                "301".into(),
                SpaceBodyKnownDetails {
                    mass: 7.349e22,
                    material: TexturePath("textures/moon_base_color.jpg".into()),
                    rotation: Default::default(),
                    sideral_rotation_offset: Default::default(),
                    sideral_rotation_speed: 0.0000026617,
                },
            );

            SpaceBodiesKnownDetails { map }
        });

        app.add_system(systems::reqeust_nasa_bodies_on_event);
        app.add_system(
            systems::manage_nasa_bodies_on_response.after(systems::reqeust_nasa_bodies_on_event),
        );
        app.add_system(systems::insert_nasa_bodies.after(systems::manage_nasa_bodies_on_response));
    }
}
pub struct SpawnNasaBodyRequest {
    pub date: DateTime<Utc>,
    pub name: String,
}

pub struct SpawnNasaBodyResponse {
    pub date: DateTime<Utc>,
    pub name: String,
    pub body: SpaceBody,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct NasaBodyAddition {
    pub date: DateTime<Utc>,
    pub name: String,
    pub body: SpaceBody,
    pub material: SpaceBodyKnownDetailsMaterial,
}

pub enum SpawnNasaBodyResponseResult {
    Errored,
    Some(SpawnNasaBodyResponse),
}

#[derive(Resource, Default)]
pub struct NasaTasksManager {
    pub tasks: Vec<Task<SpawnNasaBodyResponseResult>>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum SpaceBodyKnownDetailsMaterial {
    TexturePath(std::borrow::Cow<'static, str>),
    Star(StarMaterial),
}

pub struct SpaceBodyKnownDetails {
    pub mass: f64,
    pub material: SpaceBodyKnownDetailsMaterial,
    pub rotation: Quat,
    pub sideral_rotation_offset: f64,
    pub sideral_rotation_speed: f64,
}

#[derive(Resource)]
pub struct SpaceBodiesKnownDetails {
    pub map: bevy::utils::HashMap<String, SpaceBodyKnownDetails>,
}

pub mod systems {
    use bevy::{prelude::*, tasks::AsyncComputeTaskPool};

    use crate::space::{
        nasa_horizons::NasaBodyAddition,
        simulation::{SpaceBody, SpaceBodyRotation, SpaceSimulation},
    };

    use super::{
        NasaTasksManager, SpaceBodiesKnownDetails, SpawnNasaBodyRequest, SpawnNasaBodyResponse,
        SpawnNasaBodyResponseResult,
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
                let Ok(body) =
                    super::get_body_dynamics_using_nasa_horizons(date, &name)
                        .await
                        else { return SpawnNasaBodyResponseResult::Errored };

                SpawnNasaBodyResponseResult::Some(SpawnNasaBodyResponse { date, name, body })
            }));
        }
    }

    pub fn manage_nasa_bodies_on_response(
        mut manager: ResMut<NasaTasksManager>,
        known_details: ResMut<SpaceBodiesKnownDetails>,
        mut ev: EventWriter<NasaBodyAddition>,
    ) {
        use futures_lite::future;

        manager.tasks.retain_mut(|task| {
            let Some(response) = future::block_on(future::poll_once(task)) else { return true };

            let SpawnNasaBodyResponseResult::Some(response) = response else { return false };

            let mass;
            let rotation;
            let sideral_rotation_speed;
            let sideral_rotation_offset;
            let material;

            if let Some(details) = known_details.map.get(&response.name) {
                mass = details.mass;
                rotation = details.rotation;
                sideral_rotation_speed = details.sideral_rotation_speed;
                sideral_rotation_offset = details.sideral_rotation_offset;
                material = details.material.clone();
            } else {
                mass = response.body.mass;
                rotation = Default::default();
                sideral_rotation_speed = Default::default();
                sideral_rotation_offset = Default::default();
                material = crate::space::nasa_horizons::SpaceBodyKnownDetailsMaterial::TexturePath(
                    "textures/asteroid.jpg".into(),
                );
            }

            let st = NasaBodyAddition {
                date: response.date,
                name: response.name,
                body: SpaceBody {
                    mass,
                    rotation: SpaceBodyRotation {
                        initial: rotation,
                        sideral_rotation_offset,
                        sideral_rotation_speed,
                    },
                    ..response.body
                },
                material,
            };

            ev.send(st);

            return false;
        });
    }

    pub fn insert_nasa_bodies(
        mut ev: EventReader<NasaBodyAddition>,
        mut simulation: ResMut<SpaceSimulation>,
    ) {
        for e in ev.iter().cloned() {
            simulation.bodies.insert(e.name, e.body);
        }
    }
}
