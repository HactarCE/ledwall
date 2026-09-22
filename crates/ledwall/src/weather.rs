use std::collections::HashMap;

use chrono::NaiveDateTime;
use eyre::{Context, OptionExt, ensure};
use serde::Deserialize;

const API_URL: &str = "https://api.open-meteo.com/v1/forecast";
const LOCATION: &str = include_str!("../../../location.txt");

#[derive(Debug, Clone, PartialEq)]
pub struct WeatherForecast {
    /// Current temperature, in degrees Fahrenheit.
    pub current_temperature: f64,
    /// Millimeters of rainfall per half-hour.
    pub rainfall: HashMap<NaiveDateTime, f64>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    current: CurrentWeather,
    minutely_15: Minutely15,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    /// Temperature at 2 meters above the ground.
    temperature_2m: f64,
}

#[derive(Debug, Deserialize)]
struct Minutely15 {
    time: Vec<String>,
    precipitation: Vec<f64>,
}

/// Fetch the weather forecast for today.
pub fn fetch_weather() -> eyre::Result<WeatherForecast> {
    let mut coordinates = LOCATION
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.parse::<f32>());
    let lat = coordinates.next().ok_or_eyre("missing lattitude")??;
    let lon = coordinates.next().ok_or_eyre("missing longitude")??;
    ensure!(coordinates.next().is_none(), "unexpected extra coordinate");
    ensure!((-90.0..=90.0).contains(&lat), "lattitude outside of ±90");
    ensure!((-180.0..=180.0).contains(&lon), "longitude outside of ±180");

    let response: ApiResponse = ureq::get(API_URL)
        .query("latitude", lat.to_string())
        .query("longitude", lon.to_string())
        .query("current", "temperature_2m")
        .query("temperature_unit", "fahrenheit")
        .query("minutely_15", "precipitation")
        .query("forecast_days", "1")
        .query("timezone", "auto")
        .header("Accept", "application/json")
        .call()?
        .into_body()
        .read_json()
        .wrap_err("API error")?;

    let rainfall = (0..response.minutely_15.time.len() / 2)
        .map(|i| {
            let half_hour_start_time =
                NaiveDateTime::parse_from_str(&response.minutely_15.time[i * 2], "%Y-%m-%dT%H:%M")?;
            let half_hour_precipitation = response.minutely_15.precipitation[i * 2]
                + response.minutely_15.precipitation[i * 2 + 1];
            Ok((half_hour_start_time, half_hour_precipitation))
        })
        .collect::<Result<_, chrono::ParseError>>()?;

    Ok(WeatherForecast {
        current_temperature: response.current.temperature_2m,
        rainfall,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_weather() {
        assert!(fetch_weather().is_ok());
    }
}
