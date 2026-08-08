use std::{error::Error, io};

use chrono::NaiveDate;
use sentry::{
    domain::scenario::{Rate, Scenario},
    errors::storage_error::StorageError,
    storage::postgres::ScenarioRepository,
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::task::JoinSet;

type TestResult = Result<(), Box<dyn Error>>;
type ExpectedScenario = (String, String);

fn rate(char_code: &str, value: &str) -> Rate {
    Rate {
        id: format!("R-{char_code}"),
        num_code: "840".to_owned(),
        char_code: char_code.to_owned(),
        nominal: 1,
        name: format!("Currency {char_code}"),
        value: value.to_owned(),
    }
}

fn scenario(date: &str, rates: Vec<Rate>) -> Scenario {
    Scenario {
        date: date.to_owned(),
        rates,
    }
}

fn generated_date(ordinal: u32) -> Result<String, io::Error> {
    NaiveDate::from_yo_opt(2000, ordinal)
        .map(|date| date.format("%d/%m/%Y").to_string())
        .ok_or_else(|| io::Error::other("test generated an invalid ordinal date"))
}

async fn store_parallel_scenarios(
    repository: &ScenarioRepository,
    count: u32,
) -> Result<Vec<ExpectedScenario>, Box<dyn Error>> {
    let mut writes = JoinSet::new();
    for ordinal in 1..=count {
        let date = generated_date(ordinal)?;
        let value = format!("{ordinal}.25");
        let repository = repository.clone();
        writes.spawn(async move {
            repository
                .replace(&scenario(&date, vec![rate("USD", &value)]))
                .await?;
            Ok::<_, StorageError>((date, value))
        });
    }

    let mut expected = Vec::with_capacity(count as usize);
    while let Some(result) = writes.join_next().await {
        expected.push(result??);
    }
    Ok(expected)
}

async fn verify_parallel_scenarios(
    repository: &ScenarioRepository,
    expected: Vec<ExpectedScenario>,
) -> Result<(), Box<dyn Error>> {
    let mut reads = JoinSet::new();
    for (date, expected_value) in expected {
        let repository = repository.clone();
        reads.spawn(async move {
            let rates = repository.find_rates(&date).await?;
            Ok::<_, StorageError>(rates.len() == 1 && rates[0].value == expected_value)
        });
    }

    while let Some(result) = reads.join_next().await {
        assert!(result??);
    }
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn stores_replaces_and_reads_scenario(pool: PgPool) -> TestResult {
    let repository = ScenarioRepository::new(pool);
    repository
        .replace(&scenario(
            "02/03/2002",
            vec![rate("USD", "30.1372"), rate("EUR", "26.3732")],
        ))
        .await?;

    repository
        .replace(&scenario("02/03/2002", vec![rate("GBP", "42.5000")]))
        .await?;

    let rates = repository.find_rates("02/03/2002").await?;
    assert_eq!(rates.len(), 1);
    assert_eq!(rates[0].char_code, "GBP");
    assert_eq!(rates[0].value, "42.5000");
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn explicitly_deletes_rates_before_scenario(pool: PgPool) -> TestResult {
    let repository = ScenarioRepository::new(pool.clone());
    repository
        .replace(&scenario("02/03/2002", vec![rate("USD", "30.1372")]))
        .await?;

    repository.delete("02/03/2002").await?;

    let rate_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rates")
        .fetch_one(&pool)
        .await?;
    let scenario_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scenarios")
        .fetch_one(&pool)
        .await?;
    assert_eq!(rate_count, 0);
    assert_eq!(scenario_count, 0);
    assert!(matches!(
        repository.find_rates("02/03/2002").await,
        Err(StorageError::NotFound)
    ));
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn rolls_back_failed_replacement(pool: PgPool) -> TestResult {
    let repository = ScenarioRepository::new(pool);
    let duplicate_rates = scenario(
        "02/03/2002",
        vec![rate("USD", "30.1372"), rate("USD", "31.0000")],
    );

    assert!(matches!(
        repository.replace(&duplicate_rates).await,
        Err(StorageError::Database(_))
    ));
    assert!(matches!(
        repository.find_rates("02/03/2002").await,
        Err(StorageError::NotFound)
    ));
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn isolates_two_hundred_parallel_scenarios(pool: PgPool) -> TestResult {
    const SCENARIO_COUNT: u32 = 200;

    let repository = ScenarioRepository::new(pool);
    let expected = store_parallel_scenarios(&repository, SCENARIO_COUNT).await?;
    verify_parallel_scenarios(&repository, expected).await?;
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn persists_scenario_across_pool_restart(pool: PgPool) -> TestResult {
    let connect_options = pool.connect_options().as_ref().clone();
    let repository = ScenarioRepository::new(pool.clone());
    repository
        .replace(&scenario("02/03/2002", vec![rate("USD", "30.1372")]))
        .await?;

    pool.close().await;
    let reopened_pool = PgPoolOptions::new().connect_with(connect_options).await?;
    let reopened_repository = ScenarioRepository::new(reopened_pool.clone());
    let rates = reopened_repository.find_rates("02/03/2002").await?;

    assert_eq!(rates.len(), 1);
    assert_eq!(rates[0].value, "30.1372");
    reopened_pool.close().await;
    Ok(())
}
