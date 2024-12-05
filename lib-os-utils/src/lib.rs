use std::process::Command;
use serde::Serialize;

pub mod location;
pub mod wifi;
pub mod serial;
pub mod speed;

pub fn git_version() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("--version")
        .output()?;
    if !output.status.success() {
        return Err(format!("Command failed with status: {}", output.status).into());
    } else {
        Ok(String::from_utf8(output.stdout).unwrap())
    }
}

#[derive(Debug, Serialize)]
pub struct VpnStatus {
    name: String,
    status: String,
}
pub fn get_vpn_status() -> Result<VpnStatus, Box<dyn std::error::Error>> {
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
