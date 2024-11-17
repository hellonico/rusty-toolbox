use rusb::{Context, Device, UsbContext};

pub fn list_usb_devices() -> rusb::Result<()> {
    let context = Context::new()?; // Create a new libusb context
    let devices = context.devices()?; // Get the list of devices

    println!("Found {} USB devices:", devices.len());

    for device in devices.iter() {
        print_device_info(&device)?;
    }

    Ok(())
}

fn print_device_info(device: &Device<Context>) -> rusb::Result<()> {
    let descriptor = device.device_descriptor()?;

    println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
             device.bus_number(),
             device.address(),
             descriptor.vendor_id(),
             descriptor.product_id(),
    );

    Ok(())
}
