use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    domain::scenario::{Rate, Scenario},
    errors::storage_error::StorageError,
};

#[derive(Clone)]
pub struct ScenarioRepository {
    pool: PgPool,
}

impl ScenarioRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn replace(&self, scenario: &Scenario) -> Result<(), StorageError> {
        let mut transaction = self.pool.begin().await?;
        self.ensure_scenario(&mut transaction, &scenario.date)
            .await?;
        self.delete_rates(&mut transaction, &scenario.date).await?;

        for rate in &scenario.rates {
            sqlx::query(
                "INSERT INTO rates
                 (scenario_date, id, num_code, char_code, nominal, name, value)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(&scenario.date)
            .bind(&rate.id)
            .bind(&rate.num_code)
            .bind(&rate.char_code)
            .bind(rate.nominal)
            .bind(&rate.name)
            .bind(&rate.value)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        Ok(())
    }

    pub async fn find_rates(&self, date: &str) -> Result<Vec<Rate>, StorageError> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM scenarios WHERE date = $1)")
                .bind(date)
                .fetch_one(&self.pool)
                .await?;

        if !exists {
            return Err(StorageError::NotFound);
        }

        Ok(sqlx::query_as::<_, Rate>(
            "SELECT id, num_code, char_code, nominal, name, value
             FROM rates WHERE scenario_date = $1 ORDER BY char_code",
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn delete(&self, date: &str) -> Result<(), StorageError> {
        let mut transaction = self.pool.begin().await?;
        self.delete_rates(&mut transaction, date).await?;
        let result = sqlx::query("DELETE FROM scenarios WHERE date = $1")
            .bind(date)
            .execute(&mut *transaction)
            .await?;
        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }
        transaction.commit().await?;
        Ok(())
    }

    async fn ensure_scenario(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        date: &str,
    ) -> Result<(), StorageError> {
        sqlx::query("INSERT INTO scenarios(date) VALUES ($1) ON CONFLICT(date) DO NOTHING")
            .bind(date)
            .execute(&mut **transaction)
            .await?;
        Ok(())
    }

    async fn delete_rates(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        date: &str,
    ) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM rates WHERE scenario_date = $1")
            .bind(date)
            .execute(&mut **transaction)
            .await?;
        Ok(())
    }
}
