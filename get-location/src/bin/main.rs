use reqwest::blocking::get;
use serde::Deserialize;

#[derive(Deserialize)]
struct IpApiResponse {
    city: String,
    region: String,
    country: String,
    lat: f64,
    lon: f64,
}

fn get_location() -> Result<IpApiResponse, reqwest::Error> {
    let response: IpApiResponse = get("http://ip-api.com/json")?.json()?;
    Ok(response)
}

fn main() {
    match get_location() {
        Ok(location) => {
            println!("City: {}", location.city);
            println!("Region: {}", location.region);
            println!("Country: {}", location.country);
            println!("Latitude: {}", location.lat);
            println!("Longitude: {}", location.lon);
        }
        Err(e) => eprintln!("Error fetching location: {}", e),
    }
}
