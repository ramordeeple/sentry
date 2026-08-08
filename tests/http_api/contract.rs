use axum::http::{StatusCode, header};
use sqlx::PgPool;

use crate::{
    client::{TestApi, TestResult, response_text},
    fixtures::{SCENARIO_DATE, SCENARIO_RATE, scenario},
};

const XML_CONTENT_TYPE: &str = "application/xml; charset=utf-8";

#[sqlx::test(migrations = "./migrations")]
async fn supports_complete_http_scenario_lifecycle(pool: PgPool) -> TestResult {
    let api = TestApi::new(pool);

    let created = api
        .put_scenario(&scenario(SCENARIO_DATE, SCENARIO_RATE))
        .await?;
    assert_eq!(created.status(), StatusCode::NO_CONTENT);

    let fetched = api.get_rates(SCENARIO_DATE).await?;
    assert_eq!(fetched.status(), StatusCode::OK);
    assert_eq!(fetched.headers()[header::CONTENT_TYPE], XML_CONTENT_TYPE);

    let xml = response_text(fetched).await?;
    assert!(xml.contains("<CharCode>USD</CharCode>"));
    assert!(xml.contains("<Value>30,1372</Value>"));
    assert!(xml.contains("Доллар США"));
    assert!(!xml.contains("<CharCode>EUR</CharCode>"));

    let deleted = api.delete_scenario(SCENARIO_DATE).await?;
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        api.get_rates(SCENARIO_DATE).await?.status(),
        StatusCode::NOT_FOUND
    );
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn rejects_invalid_admin_requests_without_writing_data(pool: PgPool) -> TestResult {
    let api = TestApi::new(pool.clone());

    let invalid_scenario = api
        .put_scenario(&scenario("31/02/2024", SCENARIO_RATE))
        .await?;
    assert_eq!(invalid_scenario.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        api.put_raw_json("{").await?.status(),
        StatusCode::BAD_REQUEST
    );

    let scenario_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scenarios")
        .fetch_one(&pool)
        .await?;
    assert_eq!(scenario_count, 0);
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn returns_expected_statuses_for_empty_state(pool: PgPool) -> TestResult {
    let api = TestApi::new(pool);

    assert_eq!(api.health().await?.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        api.get_rates_without_date().await?.status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        api.delete_scenario("01/01/2000").await?.status(),
        StatusCode::NOT_FOUND
    );
    Ok(())
}
