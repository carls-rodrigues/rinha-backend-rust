use rayon::prelude::*;
use sqlx::Row;
use std::sync::Arc;

use crate::errors::ApiError;
use crate::models::{self, StatementOutput, Transaction};
use crate::queries::sql::{
    GET_STATEMENT_QUERY, INSERT_TRANSACTION_QUERY, UPDATE_CREDIT_TRANSACTION_QUERY,
    UPDATE_DEBIT_TRANSACTION_QUERY,
};

pub struct Services {
    pub connection: Arc<sqlx::PgPool>,
}

impl Services {
    pub fn new(connection: Arc<sqlx::PgPool>) -> Self {
        Self { connection }
    }
    pub async fn get_statement(
        &self,
        customer_id: i32,
    ) -> Result<models::AccountStatement, ApiError> {
        let query = sqlx::query(GET_STATEMENT_QUERY)
            .bind(customer_id)
            .fetch_all(self.connection.as_ref())
            .await?;
        let first = query.first().unwrap();
        let statement: StatementOutput = first.try_into().unwrap();
        let transactions: Vec<Transaction> = match first.try_get::<i32, _>("tvalor") {
            Ok(_) => query
                .par_iter()
                .map(|x| {
                    let transaction: models::Transaction = x.try_into().unwrap();
                    transaction
                })
                .collect::<Vec<models::Transaction>>(),
            Err(_) => Vec::new(),
        };
        let account_statement = models::AccountStatement {
            balance: statement,
            transactions,
        };
        Ok(account_statement)
    }

    pub async fn create_transaction(
        &self,
        customer_id: i32,
        input: models::IncomeTransaction,
    ) -> Result<models::OutputTransaction, ApiError> {
        let tx = self.connection.begin().await?;
        let transaction = models::Transaction {
            id: 1,
            amount: input.amount,
            r#type: input.r#type,
            customer_id,
            created_at: chrono::Utc::now().naive_utc(),
            description: input.description,
        };
        let (limit, balance) = self.commit_transaction(tx, transaction).await?;

        Ok(models::OutputTransaction { limit, balance })
    }
    async fn commit_transaction(
        &self,
        mut tx: sqlx::Transaction<'_, sqlx::Postgres>,
        transaction: models::Transaction,
    ) -> Result<(i32, i32), ApiError> {
        let current_transaction_query = match transaction.r#type.as_str() {
            "c" => UPDATE_CREDIT_TRANSACTION_QUERY,
            "d" => UPDATE_DEBIT_TRANSACTION_QUERY,
            _ => return Err(ApiError::UnprocessableEntity),
        };

        let update_balance = sqlx::query(current_transaction_query)
            .bind(transaction.amount)
            .bind(transaction.customer_id)
            .fetch_one(&mut *tx)
            .await;
        let customer = match update_balance.is_err() {
            false => {
                let customer: models::Customer = update_balance.unwrap().try_into().unwrap();
                Ok(customer)
            }
            _ => Err(ApiError::UnprocessableEntity),
        }?;
        let insert_transaction = sqlx::query(INSERT_TRANSACTION_QUERY)
            .bind(transaction.customer_id)
            .bind(transaction.amount)
            .bind(transaction.r#type.as_str())
            .bind(transaction.description.as_str())
            .execute(&mut *tx)
            .await;
        match insert_transaction {
            Ok(_) => {
                tx.commit().await?;
                Ok((customer.limit, customer.balance))
            }
            Err(_) => {
                tx.rollback().await?;
                Err(ApiError::UnprocessableEntity)
            }
        }
    }
}
