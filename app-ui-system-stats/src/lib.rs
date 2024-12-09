// use battery::Battery;
use get_if_addrs::get_if_addrs;
use serde::Serialize;
use sysinfo::{CpuExt, DiskExt, System, SystemExt};
use lib_os_utils::{get_vpn_status, git_version, VpnStatus};
use lib_os_utils::location::{get_location, Location};
use lib_os_utils::serial::get_serial_number;
use lib_os_utils::wifi::get_wifi_ssid;

#[ derive ( Serialize, Debug )]
pub struct Battery {
    // pub capacity
    pub state: String,
    pub state_of_charge: f32,
    pub cycle_count: String,
    pub capacity: f32,
    pub model: String,
    // pub energy_value: f32,
    pub voltage: f32,
}
#[ derive ( Serialize, Debug )]
pub struct Memory {
    pub total_memory: f64,
    pub used_memory: f64,
    pub free_memory: f64,
}
#[ derive ( Serialize, Debug )]
pub struct Disk {
    // pub name: String,
    // pub usage: f32,
    pub total_space_gb : f64,
    pub free_space_gb: f64,
    pub used_space_gb: f64,
    pub mount: String,
    pub name: String,
    // pub filesystem: String,
}
#[ derive ( Serialize, Debug )]
pub struct Cpu {
    pub name: String,
    pub usage: f32,
}
#[ derive ( Serialize, Debug )]
pub struct SysStats {
    #[serde( skip )]
    systeminfo: System,
    pub long_os_version: String,
    pub kernel_version: String,
    pub host_name: String,
    pub uptime: String,
    pub cpus: Vec<Cpu>,
    pub disks: Vec<Disk>,
    pub memory: Memory,
    pub wifi_ssid: String,
    pub location: Location,
    pub serial_number: String,
    pub vpn: Option<VpnStatus>,
    pub batteries: Vec<Battery>,
    pub git_version: String,
}
impl SysStats {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        Self {
            systeminfo: sys,
            long_os_version : "".to_string(),
            kernel_version : "".to_string(),
            host_name: "".to_string(),
            uptime : "".to_string(),
            cpus : Vec::new(),
            disks : Vec::new(),
            memory : Memory {total_memory: 0.0, used_memory: 0.0, free_memory: 0.0},
            wifi_ssid: "".to_string(),
            location: Location{
                city: "".to_string(),
                country: "".to_string(),
                lat: 0.0,
                lon: 0.0,
                region_name: "".to_string(),
                isp: "".to_string(),
            },
            serial_number: "".to_string(),
            vpn: None,
            batteries: Vec::new(),
            git_version: "".to_string(),
        }
    }

    pub fn refresh_all(&mut self) {
        self.systeminfo.refresh_all();
        
        self.long_os_version = self.systeminfo.long_os_version().unwrap_or(String::new());
        self.kernel_version = self.systeminfo.kernel_version().unwrap_or(String::new());
        self.host_name = self.systeminfo.host_name().unwrap_or(String::new());
        self.uptime = Option::from(self.format_uptime()).unwrap_or(String::new());
        self.memory = Memory {
            total_memory : Self::get_memory_in_gb(self.systeminfo.total_memory()),
            used_memory : Self::get_memory_in_gb(self.systeminfo.used_memory()),
            free_memory : Self::get_memory_in_gb(self.systeminfo.total_memory() - self.systeminfo.used_memory()),
        };
        self.cpus = self.systeminfo.cpus().iter().map(|cpu| Cpu {
            name: cpu.name().to_string(),
            usage: cpu.cpu_usage(),
        }).collect();
        self.disks = self.systeminfo.disks().iter().map(|disk| Disk{
            total_space_gb : (disk.total_space() as f64) / 1_000_000_000.0,
            free_space_gb : (disk.available_space() as f64) / 1_000_000_000.0,
            used_space_gb : ((disk.total_space() as f64) - (disk.available_space() as f64)) / 1_000_000_000.0,
            mount : disk.mount_point().display().to_string(),
            name: disk.name().to_string_lossy().into_owned(),
        }).collect();
        self.wifi_ssid = get_wifi_ssid().unwrap_or("No Wi-Fi connection found".to_string());
        self.location = get_location().unwrap_or(Location{
            city: "None".to_string(),
            country: "None".to_string(),
            lat: 0.0,
            lon: 0.0,
            region_name: "".to_string(),
            isp: "".to_string(),
        });
        self.serial_number = get_serial_number().unwrap_or("".to_string());
        self.vpn = get_vpn_status().ok();
        self.batteries = battery::Manager::new().unwrap().batteries().unwrap().map(|b| {
            let bb = b.unwrap();
            Battery {
                voltage : bb.voltage().value,
                state: bb.state().to_string(),
                state_of_charge: bb.state_of_charge().value,
                capacity: bb.energy().value,
                cycle_count: bb.cycle_count().unwrap().to_string(),
                model: bb.model().unwrap().to_string(),
                // energy_value: bb.energy().value,
            }
        }).collect();
        self.git_version = git_version().unwrap();
    }

    pub fn total_cpu_usage(&self) -> f32 {
        self.systeminfo
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>()
            / self.systeminfo.cpus().len() as f32
    }

    pub fn format_uptime(&self) -> String {
        let uptime_seconds = self.systeminfo.uptime();
        let hours = uptime_seconds / 3600;
        let minutes = (uptime_seconds % 3600) / 60;
        let seconds = uptime_seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    pub fn get_memory_in_gb(bytes: u64) -> f64 {
        (bytes as f64) / 1_073_741_824.0 // Convert bytes to GB (1 GB = 1024^3 bytes)
    }

    pub fn get_ip_address() -> Option<String> {
        if let Ok(interfaces) = get_if_addrs() {
            for iface in interfaces {
                if iface.is_loopback() {
                    continue;
                }
                if let std::net::IpAddr::V4(ipv4) = iface.ip() {
                    return Some(ipv4.to_string());
                }
            }
        }
        None
    }
}
