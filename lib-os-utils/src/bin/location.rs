use lib_os_utils::location::get_location;

fn main() {
    match get_location() {
        Ok(location) => {
            println!("City: {}", location.city);
            println!("Region: {}", location.region_name);
            println!("Country: {}", location.country);
            println!("Latitude: {}", location.lat);
            println!("Longitude: {}", location.lon);
            println!("ISP: {}", location.isp);
        }
        Err(e) => eprintln!("Error fetching location: {}", e),
    }
}
