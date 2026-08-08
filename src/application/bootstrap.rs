use sqlx::postgres::PgPoolOptions;

use crate::{
    api::http, config::settings::Config, errors::startup_error::StartupError,
    storage::postgres::ScenarioRepository,
};

pub async fn run(config: Config) -> Result<(), StartupError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_db_connections)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    let listener = tokio::net::TcpListener::bind(config.listen_addr).await?;
    println!("CBR mock listening on http://{}", config.listen_addr);

    axum::serve(listener, http::router(ScenarioRepository::new(pool)))
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("failed to install shutdown signal handler: {error}");
    }
}
