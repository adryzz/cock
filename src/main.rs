use axum::{routing::get, Router, Json, http::StatusCode};
use serde::Serialize;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting cock...");

    match run().await {
        Ok(_) => tracing::info!("Program exited successfully."),
        Err(e) => tracing::error!("Error: {}", e),
    }
}

async fn run() -> anyhow::Result<()> {

    let app = Router::new()
    .route("/", get(index));
    let listener = std::net::TcpListener::bind("127.0.0.1:3727")?;
    tracing::info!("Listening on {}...", listener.local_addr()?);

    axum::Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn index() -> Result<Json<SystemInfo>, StatusCode> {
    Ok(Json(SystemInfo::read_all().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemInfo {
    network_info: Vec<NetworkInterface>,
    disk_info: Vec<DiskStats>
}

impl SystemInfo {
    async fn read_all() -> anyhow::Result<Self> {
        Ok(Self {
            network_info: NetworkInterface::read_all().await?,
            disk_info: DiskStats::read_all().await?
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NetworkInterface {
    name: String,
    receive_bytes: u64,
    transmit_bytes: u64,
    receive_packets: u64,
    transmit_packets: u64,
    receive_errors: u64,
    transmit_errors: u64,
    receive_dropped: u64,
    transmit_dropped: u64,
    fifo_errors: u64,
    frame_errors: u64,
    compressed_packets: u64,
    multicast_packets: u64,
}

impl NetworkInterface {
    async fn read_all() -> anyhow::Result<Vec<NetworkInterface>> {
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

                // Create a new NetworkInterface struct and fill it with data
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

                // Push the filled struct into the Vec
                interfaces.push(interface);
            }
        }

        Ok(interfaces)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiskStats {
    major: u64,
    minor: u64,
    device_name: String,
    reads_completed: u64,
    reads_merged: u64,
    sectors_read: u64,
    read_time: u64,
    writes_completed: u64,
    writes_merged: u64,
    sectors_written: u64,
    write_time: u64,
    io_in_progress: u64,
    io_time: u64,
    io_weighted_time: u64,
}

impl DiskStats {
    async fn read_all() -> anyhow::Result<Vec<DiskStats>> {
        let file = File::open("/proc/diskstats").await?;
        let reader = BufReader::new(file);
        let mut disks: Vec<DiskStats> = Vec::new();
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 14 {
                // Create a new DiskStats struct and fill it with data
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

                // Push the filled struct into the Vec
                disks.push(disk_stat);
            }
        }

        Ok(disks)
    }
}
