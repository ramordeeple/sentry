use std::error::Error;

use axum::{
    Router,
    body::Body,
    http::{Method, Request, Response, header},
};
use http_body_util::BodyExt;
use sentry::{api::http, domain::scenario::Scenario, storage::postgres::ScenarioRepository};
use sqlx::PgPool;
use tower::ServiceExt;

pub(crate) type TestError = Box<dyn Error + Send + Sync>;
pub(crate) type TestResult<T = ()> = Result<T, TestError>;

#[derive(Clone)]
pub(crate) struct TestApi {
    router: Router,
}

impl TestApi {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self {
            router: http::router(ScenarioRepository::new(pool)),
        }
    }

    pub(crate) async fn health(&self) -> TestResult<Response<Body>> {
        self.send(Method::GET, "/health", Body::empty(), None).await
    }

    pub(crate) async fn put_scenario(&self, scenario: &Scenario) -> TestResult<Response<Body>> {
        let body = Body::from(serde_json::to_vec(scenario)?);
        self.send(
            Method::PUT,
            "/__admin/scenarios",
            body,
            Some("application/json"),
        )
        .await
    }

    pub(crate) async fn put_raw_json(&self, body: &str) -> TestResult<Response<Body>> {
        self.send(
            Method::PUT,
            "/__admin/scenarios",
            Body::from(body.to_owned()),
            Some("application/json"),
        )
        .await
    }

    pub(crate) async fn get_rates(&self, date: &str) -> TestResult<Response<Body>> {
        let uri = format!("/scripts/XML_daily.asp?date_req={}", encode_date(date));
        self.send(Method::GET, &uri, Body::empty(), None).await
    }

    pub(crate) async fn get_rates_without_date(&self) -> TestResult<Response<Body>> {
        self.send(Method::GET, "/scripts/XML_daily.asp", Body::empty(), None)
            .await
    }

    pub(crate) async fn delete_scenario(&self, date: &str) -> TestResult<Response<Body>> {
        let uri = format!("/__admin/scenarios?date={}", encode_date(date));
        self.send(Method::DELETE, &uri, Body::empty(), None).await
    }

    async fn send(
        &self,
        method: Method,
        uri: &str,
        body: Body,
        content_type: Option<&str>,
    ) -> TestResult<Response<Body>> {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(content_type) = content_type {
            request = request.header(header::CONTENT_TYPE, content_type);
        }
        Ok(self.router.clone().oneshot(request.body(body)?).await?)
    }
}

pub(crate) async fn response_text(response: Response<Body>) -> TestResult<String> {
    let bytes = response.into_body().collect().await?.to_bytes();
    Ok(String::from_utf8(bytes.to_vec())?)
}

fn encode_date(date: &str) -> String {
    date.replace('/', "%2F")
}
