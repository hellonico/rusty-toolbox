use std::error::Error;
use std::sync::Arc;
use btleplug::api::{Central, Manager, Peripheral};
use tokio::sync::Mutex;

pub async fn collect_bt_devices() -> btleplug::Result<Vec<impl Peripheral>> {
    // Initialize Bluetooth manager
    let manager = btleplug::platform::Manager::new().await?;

    // Get the list of Bluetooth adapters
    let adapters = manager.adapters().await;
    //
    // if adapters.is_empty() {
    //     eprintln!("No Bluetooth adapters found!");
    //     return Ok(Vec::new());
    // }

    // Select the first available adapter
    let adapter = adapters.unwrap().into_iter().next().unwrap();

    // Start scanning for devices
    adapter.start_scan(btleplug::api::ScanFilter::default()).await.unwrap();

    // Wait for a few seconds to detect nearby devices
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Get the list of discovered devices
    adapter.peripherals().await
}


pub async fn format_bt_peripheral(device: impl btleplug::api::Peripheral) -> Result<(), Box<dyn Error>> {
    let name = device.properties().await?.unwrap().local_name.unwrap_or_else(|| "Unknown".to_string());
    let address = device.address();
    println!("Device: {}, Address: {}", name, address);
    Ok(())
}

pub async fn fetch_bluetooth_devices(
    bluetooth_devices: Arc<Mutex<Vec<String>>>, // The shared vector to store device names
    central: Arc<Mutex<btleplug::platform::Adapter>>, // The Bluetooth adapter (central)
) {
    let bluetooth_devices = bluetooth_devices.clone(); // Clone the Arc to move into async block

    // Spawn a new async task
    tokio::spawn(async move {
        // Lock the Adapter object
        let mut central = central.lock().await; // This is an async lock on the Adapter

        // Fetch available peripherals (Bluetooth devices)
        match central.peripherals().await {
            Ok(peripherals) => {
                println!("Found {} Bluetooth peripheral:", peripherals.len());
                // Lock the bluetooth_devices vector to modify it safely
                let mut devices = bluetooth_devices.lock().await;

                // Iterate over all available peripherals and add their names to the list
                for peripheral in peripherals {
                    match peripheral.properties().await {
                        Ok(properties) => {
                            let name = properties.unwrap().local_name.unwrap_or_else(|| "Unknown Device".to_string());
                            println!("Found Bluetooth Device {}", name);
                            devices.push(name);
                        }
                        Err(err) => {
                            eprintln!("Failed to fetch properties for peripheral: {}", err);
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to fetch peripherals: {}", err);
            }
        }
    });
}
