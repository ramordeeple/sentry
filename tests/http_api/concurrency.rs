use axum::http::StatusCode;
use sqlx::PgPool;
use tokio::task::JoinSet;

use crate::{
    client::{TestApi, TestResult, response_text},
    fixtures::{generated_date, scenario},
};

const FLOW_COUNT: u32 = 200;

async fn execute_isolated_flow(api: TestApi, ordinal: u32) -> TestResult<bool> {
    let date = generated_date(ordinal)?;
    let value = format!("{ordinal}.25");

    if api.put_scenario(&scenario(&date, &value)).await?.status() != StatusCode::NO_CONTENT {
        return Ok(false);
    }

    let response = api.get_rates(&date).await?;
    if response.status() != StatusCode::OK {
        return Ok(false);
    }

    let xml = response_text(response).await?;
    Ok(xml.contains(&format!("<Value>{ordinal},25</Value>")))
}

#[sqlx::test(migrations = "./migrations")]
async fn isolates_two_hundred_parallel_http_flows(pool: PgPool) -> TestResult {
    let api = TestApi::new(pool);
    let mut flows = JoinSet::new();

    for ordinal in 1..=FLOW_COUNT {
        flows.spawn(execute_isolated_flow(api.clone(), ordinal));
    }

    while let Some(result) = flows.join_next().await {
        assert!(result??);
    }
    Ok(())
}
