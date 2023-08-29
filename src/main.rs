use std::time::Duration;

use clickhouse_postgres_client::{ClickhousePgPoolOptions, ClickhousePgConnectOptions};

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