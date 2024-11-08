use lib_os_utils::serial::get_serial_number;

fn main() {
    match get_serial_number() {
        Some(serial) => println!("Serial Number: {}", serial),
        None => println!("Failed to retrieve serial number."),
    }
}
