use reqwest::blocking::get;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Location {
    pub city: String,
    #[serde(rename = "regionName")]
    pub region_name: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    pub isp: String,
}


pub fn get_location() -> Result<Location, reqwest::Error> {
    let response: Location = get("http://ip-api.com/json")?.json()?;
    Ok(response)
}