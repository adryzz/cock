use std::time::Duration;

use clickhouse_postgres_client::{ClickhousePgPoolOptions, ClickhousePgConnectOptions, sqlx_clickhouse_ext::sqlx_core::{pool::Pool, postgres::Postgres}};

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
    let database_url = "postgres://default:xxx@127.0.0.1:9005";
    let pool = ClickhousePgPoolOptions::new()
    .max_connections(5)
    .connect_lazy_with(
        database_url
            .parse::<ClickhousePgConnectOptions>()?
            .into_inner(),
    );

    let mut interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        interval.tick().await;
        let system_info = sys_info::SystemInfo::read_all().await?;
    }
    Ok(())
}

async fn create_table_if_not_exists(pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let query = "CREATE TABLE IF NOT EXISTS system_info (
        timestamp DateTime DEFAULT now(),
        cpu_stats Nested (
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
        ),
        mem_info Nested (
            mem_total UInt64,
            mem_free UInt64,
            mem_available UInt64,
            buffers UInt64,
            cached UInt64,
            swap_total UInt64,
            swap_free UInt64
        ),
        network_info Array(
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
        ),
        disk_info Array(
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
    ORDER BY timestamp;";

    Ok(())
}