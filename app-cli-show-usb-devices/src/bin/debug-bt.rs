use app_cli_show_usb_devices::bt::{collect_bt_devices, format_bt_peripheral};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let devices = collect_bt_devices().await.unwrap();

    for device in devices {
        format_bt_peripheral(device).await?;
    }

    Ok(())
}
