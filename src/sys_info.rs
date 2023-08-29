use anyhow::anyhow;
use clickhouse::Row;
use serde::Serialize;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};


#[derive(Debug, Clone, Row, Serialize)]
pub struct SystemInfo {
    pub cpu_stats: CpuStats,
    pub mem_info: MemInfo,
    pub network_info: Vec<NetworkInterface>,
    pub disk_info: Vec<DiskStats>
}

impl SystemInfo {
    pub async fn read_all() -> anyhow::Result<Self> {
        Ok(Self {

            cpu_stats: CpuStats::read_all().await?,
            mem_info: MemInfo::read_all().await?,
            network_info: NetworkInterface::read_all().await?,
            disk_info: DiskStats::read_all().await?
        })
    }
}

#[derive(Debug, Clone, Row, Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub receive_bytes: u64,
    pub transmit_bytes: u64,
    pub receive_packets: u64,
    pub transmit_packets: u64,
    pub receive_errors: u64,
    pub transmit_errors: u64,
    pub receive_dropped: u64,
    pub transmit_dropped: u64,
    pub fifo_errors: u64,
    pub frame_errors: u64,
    pub compressed_packets: u64,
    pub multicast_packets: u64,
}

impl NetworkInterface {
    pub async fn read_all() -> anyhow::Result<Vec<NetworkInterface>> {
        let file = File::open("/proc/net/dev").await?;
        let reader = BufReader::new(file);
        let mut interfaces: Vec<NetworkInterface> = Vec::new();
        let mut lines = reader.lines();

        lines.next_line().await?;
        lines.next_line().await?;

        while let Some(line) = lines.next_line().await? {
            // Split the line into parts
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 17 {
                let name = parts[0].trim_matches(':');

                // Create a new NetworkInterface pub struct and fill it with data
                let interface = Self {
                    name: name.to_string(),
                    receive_bytes: parts[1].parse::<u64>().unwrap_or(0),
                    transmit_bytes: parts[9].parse::<u64>().unwrap_or(0),
                    receive_packets: parts[2].parse::<u64>().unwrap_or(0),
                    transmit_packets: parts[10].parse::<u64>().unwrap_or(0),
                    receive_errors: parts[3].parse::<u64>().unwrap_or(0),
                    transmit_errors: parts[11].parse::<u64>().unwrap_or(0),
                    receive_dropped: parts[4].parse::<u64>().unwrap_or(0),
                    transmit_dropped: parts[12].parse::<u64>().unwrap_or(0),
                    fifo_errors: parts[5].parse::<u64>().unwrap_or(0),
                    frame_errors: parts[6].parse::<u64>().unwrap_or(0),
                    compressed_packets: parts[7].parse::<u64>().unwrap_or(0),
                    multicast_packets: parts[8].parse::<u64>().unwrap_or(0),
                };

                // Push the filled pub struct into the Vec
                interfaces.push(interface);
            }
        }

        Ok(interfaces)
    }
}

#[derive(Debug, Clone, Row, Serialize)]
pub struct DiskStats {
    pub major: u64,
    pub minor: u64,
    pub device_name: String,
    pub reads_completed: u64,
    pub reads_merged: u64,
    pub sectors_read: u64,
    pub read_time: u64,
    pub writes_completed: u64,
    pub writes_merged: u64,
    pub sectors_written: u64,
    pub write_time: u64,
    pub io_in_progress: u64,
    pub io_time: u64,
    pub io_weighted_time: u64,
}

impl DiskStats {
    pub async fn read_all() -> anyhow::Result<Vec<DiskStats>> {
        let file = File::open("/proc/diskstats").await?;
        let reader = BufReader::new(file);
        let mut disks: Vec<DiskStats> = Vec::new();
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 14 {
                // Create a new DiskStats pub struct and fill it with data
                let disk_stat = Self {
                    major: parts[0].parse::<u64>().unwrap_or(0),
                    minor: parts[1].parse::<u64>().unwrap_or(0),
                    device_name: parts[2].to_string(),
                    reads_completed: parts[3].parse::<u64>().unwrap_or(0),
                    reads_merged: parts[4].parse::<u64>().unwrap_or(0),
                    sectors_read: parts[5].parse::<u64>().unwrap_or(0),
                    read_time: parts[6].parse::<u64>().unwrap_or(0),
                    writes_completed: parts[7].parse::<u64>().unwrap_or(0),
                    writes_merged: parts[8].parse::<u64>().unwrap_or(0),
                    sectors_written: parts[9].parse::<u64>().unwrap_or(0),
                    write_time: parts[10].parse::<u64>().unwrap_or(0),
                    io_in_progress: parts[11].parse::<u64>().unwrap_or(0),
                    io_time: parts[12].parse::<u64>().unwrap_or(0),
                    io_weighted_time: parts[13].parse::<u64>().unwrap_or(0),
                };

                // Push the filled pub struct into the Vec
                disks.push(disk_stat);
            }
        }

        Ok(disks)
    }
}

#[derive(Debug, Clone, Copy, Row, Serialize)]
pub struct CpuStats {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
    pub guest: u64,
    pub guest_nice: u64,
}

impl CpuStats {
    pub async fn read_all() -> anyhow::Result<CpuStats> {
        let file = File::open("/proc/stat").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 && parts[0] == "cpu" {
                // Skip the "cpu" prefix and parse the CPU statistics
                return Ok(Self {
                    user: parts[1].parse::<u64>().unwrap_or(0),
                    nice: parts[2].parse::<u64>().unwrap_or(0),
                    system: parts[3].parse::<u64>().unwrap_or(0),
                    idle: parts[4].parse::<u64>().unwrap_or(0),
                    iowait: parts[5].parse::<u64>().unwrap_or(0),
                    irq: parts[6].parse::<u64>().unwrap_or(0),
                    softirq: parts[7].parse::<u64>().unwrap_or(0),
                    steal: parts[8].parse::<u64>().unwrap_or(0),
                    guest: parts[9].parse::<u64>().unwrap_or(0),
                    guest_nice: parts[10].parse::<u64>().unwrap_or(0),
                });
            }
        }
        Err(anyhow!("Bad /proc/stat file"))
    }
}

#[derive(Debug, Clone, Copy, Row, Serialize)]
pub struct MemInfo {
    pub mem_total: u64,
    pub mem_free: u64,
    pub mem_available: u64,
    pub buffers: u64,
    pub cached: u64,
    pub swap_total: u64,
    pub swap_free: u64,
}

impl MemInfo {
    pub async fn read_all() -> anyhow::Result<MemInfo> {
        let file = File::open("/proc/diskstats").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut mem_info = Self {
            mem_total: 0,
            mem_free: 0,
            mem_available: 0,
            buffers: 0,
            cached: 0,
            swap_total: 0,
            swap_free: 0,
        };

        while let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                match parts[0] {
                    "MemTotal:" => {
                        mem_info.mem_total = parts[1].parse::<u64>().unwrap_or(0);
                    }
                    "MemFree:" => {
                        mem_info.mem_free = parts[1].parse::<u64>().unwrap_or(0);
                    }
                    "MemAvailable:" => {
                        mem_info.mem_available = parts[1].parse::<u64>().unwrap_or(0);
                    }
                    "Buffers:" => {
                        mem_info.buffers = parts[1].parse::<u64>().unwrap_or(0);
                    }
                    "Cached:" => {
                        mem_info.cached = parts[1].parse::<u64>().unwrap_or(0);
                    }
                    "SwapTotal:" => {
                        mem_info.swap_total = parts[1].parse::<u64>().unwrap_or(0);
                    }
                    "SwapFree:" => {
                        mem_info.swap_free = parts[1].parse::<u64>().unwrap_or(0);
                    }
                    _ => {}
                }
            }
        }

        Ok(mem_info)
    }
}
