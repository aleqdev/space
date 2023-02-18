use chrono::{DateTime, Utc};

use super::SpaceSimulation;

impl SpaceSimulation {
    async fn insert_using_nasa_horizons(&mut self, date: DateTime<Utc>, name: impl AsRef<str>) {
        let result = reqwest::Client::new()
            .get("https://ssd.jpl.nasa.gov/api/horizons.api")
            .query(parameters)
            .send()
            .await
            .map_err(|_| HorizonsQueryError)?
            .json::<HorizonsResponse>()
            .await
            .map_err(|_| HorizonsQueryError)?
            .result
            .split('\n')
            .map(str::to_owned)
            .collect::<Vec<String>>();
    }
}