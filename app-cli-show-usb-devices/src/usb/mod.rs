use btleplug::api::{Central, Peripheral};
use rusb::{Context, Device, UsbContext};
// use std::sync::{Arc, Mutex};
use std::time::Duration;

pub fn list_usb_devices() -> rusb::Result<()> {
    let context = Context::new()?; // Create a new libusb context
    let devices = context.devices()?; // Get the list of devices

    println!("Found {} USB devices:", devices.len());

    for device in devices.iter() {
        print_device_info(&device)?;
    }

    Ok(())
}

pub fn print_device_info(device: &Device<Context>) -> rusb::Result<()> {
    let descriptor = device.device_descriptor()?;

    println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
             device.bus_number(),
             device.address(),
             descriptor.vendor_id(),
             descriptor.product_id(),
    );

    Ok(())
}


pub fn fetch_usb_devices() -> rusb::Result<Vec<String>> {
    let context = Context::new()?;
    let devices = context.devices()?;
    let mut device_list = Vec::new();

    for device in devices.iter() {
        if let Ok(info) = format_device_info(&device) {
            device_list.push(info);
        }
    }

    Ok(device_list)
}

pub fn get_string_descriptor<T: UsbContext>(
    device: &Device<T>,
    index: u8,
) -> Option<String> {
    if index == 0 {
        return None;
    }

    let handle = device.open().ok()?;
    let language = handle
        .read_languages(Duration::from_secs(2))
        .ok()?
        .get(0)
        .cloned()?; // Get the first language

    // Extract the device descriptor or return None if it fails
    let descriptor = device.device_descriptor().ok()?;

    // Pass the descriptor reference to read_product_string
    handle
        .read_product_string(language, &descriptor, Duration::from_secs(2))
        .ok()
}

pub fn format_device_info(device: &Device<Context>) -> rusb::Result<String> {
    let descriptor = device.device_descriptor()?;

    // Attempt to retrieve the product string for a friendly name
    let friendly_name = get_string_descriptor(device, descriptor.product_string_index().unwrap())
        .unwrap_or_else(|| "Unknown Device".to_string());

    // Determine device class (e.g., Human Interface Device, Mass Storage)
    let device_type = match descriptor.class_code() {
        0x03 => "Human Interface Device (Mouse/Keyboard)".to_string(),
        0x08 => "Mass Storage (e.g., USB Drive)".to_string(),
        0x09 => "Hub".to_string(),
        0x0A => "CDC-Data (Communication Device)".to_string(),
        0xE0 => "Wireless Controller (e.g., Bluetooth Adapter)".to_string(),
        _ => "Other".to_string(),
    };

    Ok(format!(
        "Bus {:03} Device {:03} ID {:04x}:{:04x} - {} [{}]",
        device.bus_number(),
        device.address(),
        descriptor.vendor_id(),
        descriptor.product_id(),
        friendly_name,
        device_type
    ))
}
