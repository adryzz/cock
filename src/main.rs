use std::{time::Duration, env};

use clickhouse::Client;
use tracing::debug;

mod sys_info;

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
    let client = Client::default()
    .with_url(env::var("CLICKHOUSE_DATABASE")?);

    create_table_if_not_exists(&client).await?;

    let mut interval = tokio::time::interval(Duration::from_millis(env::var("COCK_TIMER_MS")?.parse::<u64>()?));

    loop {
        interval.tick().await;
        debug!("Logging system info...");
        let system_info = sys_info::SystemInfo::read_all().await?;
        let mut insert = client.insert("cpu_stats")?;
        insert.write(&system_info.cpu_stats).await?;
        insert.end().await?;

        let mut insert = client.insert("mem_stats")?;
        insert.write(&system_info.mem_info).await?;
        insert.end().await?;

        let mut insert = client.insert("net_stats")?;
        insert.write(&system_info.network_info).await?;
        insert.end().await?;

        let mut insert = client.insert("disk_stats")?;
        insert.write(&system_info.disk_info).await?;
        insert.end().await?;
    }
}

async fn create_table_if_not_exists(client: &Client) -> anyhow::Result<()> {
    let cpu_query = r#"CREATE TABLE IF NOT EXISTS cpu_stats (
        timestamp DateTime DEFAULT now(),
        user UInt64,
        nice UInt64,
        system UInt64,
        idle UInt64,
        iowait UInt64,
        irq UInt64,
        softirq UInt64,
        steal UInt64,
        guest UInt64,
        guest_nice UInt64
        
    ) ENGINE = MergeTree()
    ORDER BY timestamp;"#;

    let mem_query = r#"CREATE TABLE IF NOT EXISTS mem_stats (
        timestamp DateTime DEFAULT now(),
        mem_total UInt64,
        mem_free UInt64,
        mem_available UInt64,
        buffers UInt64,
        cached UInt64,
        swap_total UInt64,
        swap_free UInt64
        
    ) ENGINE = MergeTree()
    ORDER BY timestamp;"#;

    let net_query = r#"CREATE TABLE IF NOT EXISTS net_stats (
        timestamp DateTime DEFAULT now(),
        Array(
            (name String,
            receive_bytes UInt64,
            transmit_bytes UInt64,
            receive_packets UInt64,
            transmit_packets UInt64,
            receive_errors UInt64,
            transmit_errors UInt64,
            receive_dropped UInt64,
            transmit_dropped UInt64,
            fifo_errors UInt64,
            frame_errors UInt64,
            compressed_packets UInt64,
            multicast_packets UInt64)
        )
        
    ) ENGINE = MergeTree()
    ORDER BY timestamp;"#;

    let disk_query = r#"CREATE TABLE IF NOT EXISTS disk_stats (
        Array(
            (major UInt64,
            minor UInt64,
            device_name String,
            reads_completed UInt64,
            reads_merged UInt64,
            sectors_read UInt64,
            read_time UInt64,
            writes_completed UInt64,
            writes_merged UInt64,
            sectors_written UInt64,
            write_time UInt64,
            io_in_progress UInt64,
            io_time UInt64,
            io_weighted_time UInt64)
        )
        
    ) ENGINE = MergeTree()
    ORDER BY timestamp;"#;
    client.query(cpu_query).execute().await?;
    client.query(mem_query).execute().await?;
    client.query(net_query).execute().await?;
    client.query(disk_query).execute().await?;
    Ok(())
}