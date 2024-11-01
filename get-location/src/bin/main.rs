use get_location::get_location;

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
