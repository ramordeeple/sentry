use sentry::{Config, run};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(Config::from_env()?).await?;
    Ok(())
}
