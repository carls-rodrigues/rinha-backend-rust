use sqlx::Row;

use crate::errors::ApiError;
use crate::models::{self, StatementOutput};

pub struct Services {
    pub connection: Box<sqlx::PgPool>,
}

impl Services {
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
        // let transactions: Vec<models::Transaction> = query
        //     .iter()
        //     .map(|row| {
        //         let transaction: models::Transaction = row.try_into().unwrap();
        //         transaction
        //     })
        //     .collect();
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
            WHERE c.id = $1
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

        if transaction.r#type == "d" {
            let _ = self.debit_transaction(&mut customer, &transaction);
        }
        if transaction.r#type == "c" {
            self.credit_transaction(&mut customer, &transaction);
        }
        self.do_transaction(tx, &customer, &transaction).await?;

        Ok(models::OutputTransaction {
            limit: customer.limit,
            balance: customer.balance,
        })
    }
    fn debit_transaction(
        &self,
        customer: &mut models::Customer,
        transaction: &models::Transaction,
    ) -> Result<(), ApiError> {
        let has_limit = customer.balance.abs() + transaction.amount.abs() <= customer.limit;
        if !has_limit {
            return Err(ApiError::UnprocessableEntity);
        }
        customer.balance -= transaction.amount;
        Ok(())
    }
    fn credit_transaction(
        &self,
        customer: &mut models::Customer,
        transaction: &models::Transaction,
    ) {
        customer.balance += transaction.amount;
    }
    async fn do_transaction(
        &self,
        mut tx: sqlx::Transaction<'_, sqlx::Postgres>,
        customer: &models::Customer,
        transaction: &models::Transaction,
    ) -> Result<(), sqlx::Error> {
        // let mut tx = self.connection.begin().await?;
        sqlx::query(
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
        .await?;
        sqlx::query(
            r#"
                UPDATE public.saldos
                SET valor = $1
                WHERE cliente_id = $2
            "#,
        )
        .bind(customer.balance)
        .bind(transaction.customer_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
