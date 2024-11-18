use app_cli_show_usb_devices::usb::list_usb_devices;

fn main() {
    match list_usb_devices() {
        Ok(_) => println!("USB devices listed successfully."),
        Err(e) => eprintln!("Error: {}", e),
    }
}
