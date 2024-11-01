use reqwest::blocking::get;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct IpApiResponse {
    pub city: String,
    pub region: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
}

pub fn get_location() -> Result<IpApiResponse, reqwest::Error> {
    let response: IpApiResponse = get("http://ip-api.com/json")?.json()?;
    Ok(response)
}
