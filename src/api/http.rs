use axum::{
    Json, Router,
    body::Bytes,
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, put},
};
use serde::Deserialize;

use crate::{
    domain::scenario::Scenario, errors::api_error::ApiError, serialization::cbr_xml,
    storage::postgres::ScenarioRepository,
};

#[derive(Deserialize)]
struct DateQuery {
    date_req: String,
}

#[derive(Deserialize)]
struct DeleteQuery {
    date: String,
}

pub fn router(repository: ScenarioRepository) -> Router {
    Router::new()
        .route("/scripts/XML_daily.asp", get(get_rates))
        .route(
            "/__admin/scenarios",
            put(put_scenario).delete(delete_scenario),
        )
        .route("/health", get(health))
        .with_state(repository)
}

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn put_scenario(
    State(repository): State<ScenarioRepository>,
    Json(scenario): Json<Scenario>,
) -> Result<StatusCode, ApiError> {
    scenario.validate()?;
    repository.replace(&scenario).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_scenario(
    State(repository): State<ScenarioRepository>,
    Query(query): Query<DeleteQuery>,
) -> Result<StatusCode, ApiError> {
    repository.delete(&query.date).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_rates(
    State(repository): State<ScenarioRepository>,
    Query(query): Query<DateQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let rates = repository.find_rates(&query.date_req).await?;
    let document = cbr_xml::render(&query.date_req, &rates)?;

    Ok((
        [(header::CONTENT_TYPE, cbr_xml::CONTENT_TYPE)],
        Bytes::from(document),
    ))
}
