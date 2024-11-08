use std::process::Command;

#[cfg(target_os = "linux")]
pub fn get_serial_number() -> Option<String> {
    let output = Command::new("dmidecode")
        .arg("-s")
        .arg("system-serial-number")
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
pub fn get_serial_number() -> Option<String> {
    let output = Command::new("wmic")
        .args(&["bios", "get", "serialnumber"])
        .output()
        .ok()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        // WMIC output may contain headers, so you might need to clean it up
        let serial_number = result.lines().nth(1)?.trim().to_string();
        Some(serial_number)
    } else {
        None
    }
}


#[cfg(target_os = "macos")]
pub fn get_serial_number() -> Option<String> {
    let output = Command::new("bash").arg("-c")
        .arg("ioreg -l | grep IOPlatformSerialNumber")
        .output()
        .ok()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        let serial_number = result.split("=").nth(1)?.trim().trim_matches('"').to_string();
        Some(serial_number)
    } else {
        None
    }
}
