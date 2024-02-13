use std::sync::Arc;

use sqlx::Row;

use crate::errors::ApiError;
use crate::models::{self, StatementOutput};

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
        let query = sqlx::query(
            r#"
            SELECT s.valor, c.limite, t.id, t.cliente_id, t.valor as tvalor, t.tipo, t.descricao, t.realizada_em
            FROM public.clientes c
                JOIN public.saldos s ON c.id = s.cliente_id
                LEFT JOIN (
                    SELECT *
                    FROM transacoes
                    WHERE cliente_id = $1
                    ORDER BY realizada_em DESC
                    LIMIT 10
            ) t ON c.id = t.cliente_id
            WHERE c.id = $1;
            "#,
        )
        .bind(customer_id)
        .fetch_all(self.connection.as_ref())
        .await?;
        let statement: StatementOutput = query.first().unwrap().try_into().unwrap();
        let mut transactions = vec![];
        for row in query.iter() {
            let transactions_empty = row.try_get::<i32, _>("tvalor").is_err();
            if transactions_empty {
                break;
            }
            let transaction: models::Transaction = row.try_into().unwrap();
            transactions.push(transaction);
        }
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
        let mut tx = self.connection.begin().await?;
        let balance_query = sqlx::query(
            r#"
            SELECT s.valor, c.limite FROM public.clientes c
            JOIN public.saldos s ON c.id = s.cliente_id
            WHERE c.id = $1;
        "#,
        )
        .bind(customer_id)
        .fetch_one(&mut *tx)
        .await?;
        let mut customer: models::Customer = balance_query.try_into().unwrap();

        let transaction = models::Transaction {
            id: 1,
            amount: input.amount,
            r#type: input.r#type,
            customer_id,
            created_at: chrono::Utc::now().naive_utc(),
            description: input.description,
        };
        match transaction.r#type.as_str() {
            "c" => {
                self.credit_transaction(&mut tx, &mut customer, &transaction)
                    .await?
            }
            "d" => {
                self.debit_transaction(&mut tx, &mut customer, &transaction)
                    .await?
            }
            _ => return Err(ApiError::UnprocessableEntity),
        }
        self.do_transaction(tx, &transaction).await?;

        Ok(models::OutputTransaction {
            limit: customer.limit,
            balance: customer.balance,
        })
    }
    async fn debit_transaction(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        customer: &mut models::Customer,
        transaction: &models::Transaction,
    ) -> Result<(), ApiError> {
        let has_limit = customer.balance.abs() + transaction.amount.abs() <= customer.limit;
        if !has_limit {
            return Err(ApiError::UnprocessableEntity);
        }
        customer.balance -= transaction.amount;
        sqlx::query(
            r#"
                UPDATE public.saldos
                SET valor = valor - $1
                WHERE cliente_id = $2 AND valor - $1 >= - $3;
            "#,
        )
        .bind(transaction.amount)
        .bind(transaction.customer_id)
        .bind(customer.limit)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
    async fn credit_transaction(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        customer: &mut models::Customer,
        transaction: &models::Transaction,
    ) -> Result<(), ApiError> {
        customer.balance += transaction.amount;
        sqlx::query(
            r#"
                UPDATE public.saldos
                SET valor = valor + $1
                WHERE cliente_id = $2;
            "#,
        )
        .bind(transaction.amount)
        .bind(transaction.customer_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
    async fn do_transaction(
        &self,
        mut tx: sqlx::Transaction<'_, sqlx::Postgres>,
        transaction: &models::Transaction,
    ) -> Result<(), sqlx::Error> {
        let insert_transaction = sqlx::query(
            r#"
                INSERT INTO public.transacoes (cliente_id, valor, tipo, descricao)
                VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(transaction.customer_id)
        .bind(transaction.amount)
        .bind(transaction.r#type.as_str())
        .bind(transaction.description.as_str())
        .execute(&mut *tx)
        .await;
        if let Err(err) = insert_transaction {
            tx.rollback().await?;
            return Err(err);
        }
        tx.commit().await?;
        Ok(())
    }
}
