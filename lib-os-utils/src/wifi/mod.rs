use std::process::Command;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct WifiError {
    details: String,
}

impl WifiError {
    fn new(msg: &str) -> WifiError {
        WifiError { details: msg.to_string() }
    }
}

impl fmt::Display for WifiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for WifiError {}

/// Get the current Wi-Fi SSID based on the host operating system.
pub fn get_wifi_ssid() -> Result<String, Box<dyn Error>> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "windows")] {
            get_wifi_ssid_windows()
        } else if #[cfg(target_os = "macos")] {
            get_wifi_ssid_macos()
        } else if #[cfg(target_os = "linux")] {
            get_wifi_ssid_linux()
        } else {
            Err(Box::new(WifiError::new("Unsupported platform")))
        }
    }
}

/// Fetch Wi-Fi SSID for Windows using the 'netsh' command.
#[cfg(target_os = "windows")]
fn get_wifi_ssid_windows() -> Result<String, Box<dyn Error>> {
    let output = Command::new("netsh")
        .arg("wlan")
        .arg("show")
        .arg("interfaces")
        .output()
        .map_err(|e| WifiError::new(&format!("Failed to execute command: {}", e)))?;

    if output.status.success() {
        let ssid_info = String::from_utf8_lossy(&output.stdout);
        for line in ssid_info.lines() {
            if line.trim().starts_with("SSID") && !line.contains("BSSID") {
                let ssid = line.split(":").nth(1).unwrap_or("").trim();
                return Ok(ssid.to_string());
            }
        }
        Err(Box::new(WifiError::new("SSID not found")))
    } else {
        Err(Box::new(WifiError::new(&format!(
            "Command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))))
    }
}

/// Fetch Wi-Fi SSID for macOS using the 'networksetup' command.
#[cfg(target_os = "macos")]
fn get_wifi_ssid_macos() -> Result<String, Box<dyn Error>> {
    let output = Command::new("networksetup")
        .arg("-getairportnetwork")
        .arg("en0")  // Replace 'en0' with the proper interface if needed.
        .output()
        .map_err(|e| WifiError::new(&format!("Failed to execute command: {}", e)))?;

    if output.status.success() {
        let ssid_info = String::from_utf8_lossy(&output.stdout);
        let ssid = ssid_info.trim().replace("Current Wi-Fi Network: ", "");
        Ok(ssid)
    } else {
        Err(Box::new(WifiError::new(&format!(
            "Command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))))
    }
}

#[cfg(target_os = "linux")]
/// Fetch Wi-Fi SSID for Linux using the 'nmcli' command.
fn get_wifi_ssid_linux() -> Result<String, Box<dyn Error>> {
    let output = Command::new("nmcli")
        .arg("-t")
        .arg("-f")
        .arg("active,ssid")
        .arg("dev")
        .arg("wifi")
        .output()
        .map_err(|e| WifiError::new(&format!("Failed to execute command: {}", e)))?;

    if output.status.success() {
        let ssid_info = String::from_utf8_lossy(&output.stdout);
        for line in ssid_info.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields[0] == "yes" {
                return Ok(fields[1].to_string());
            }
        }
        Err(Box::new(WifiError::new("SSID not found")))
    } else {
        Err(Box::new(WifiError::new(&format!(
            "Command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))))
    }
}


#[derive(Debug)]
struct VpnStatus {
    name: String,
    status: String,
}


fn get_vpn_status() -> Result<VpnStatus, Box<dyn std::error::Error>> {
    // Run the command
    let output = Command::new("scutil")
        .arg("--nc")
        .arg("list")
        .output()?;

    // Check if the command was successful
    if !output.status.success() {
        return Err(format!("Command failed with status: {}", output.status).into());
    }

    // Parse the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(last_line) = stdout.lines().last() {
        // Extract status
        let status = if last_line.contains("(Disconnected)") {
            "Disconnected".to_string()
        } else if last_line.contains("(Connected)") {
            "Connected".to_string()
        } else {
            "Unknown".to_string()
        };

        // Extract VPN name
        let name = last_line
            .split('"')
            .nth(1)
            .unwrap_or("Unknown VPN Name")
            .to_string();

        // Return the result as a struct
        return Ok(VpnStatus { name, status });
    }

    Err("No VPN status found.".into())
}
