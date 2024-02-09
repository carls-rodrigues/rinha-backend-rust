use crate::errors::ApiError;
use crate::models::{self, StatementOutput};

pub struct Services {
    pub connection: Box<sqlx::PgPool>,
}

impl Services {
    pub async fn get_statement(
        &self,
        costumer_id: i32,
    ) -> Result<models::AccountStatement, ApiError> {
        if costumer_id > 5 {
            return Err(ApiError::NotFound);
        }
        let balance_query = sqlx::query(
            r#"
                SELECT s.valor, c.limite FROM public.clientes c
                LEFT JOIN public.saldos s ON c.id = s.cliente_id
                WHERE c.id = $1
            "#,
        )
        .bind(costumer_id)
        .fetch_one(self.connection.as_ref())
        .await?;
        let balance: StatementOutput = balance_query.try_into().unwrap();
        let transactions_query = sqlx::query(
            r#"
                SELECT t.id, t.cliente_id, t.valor, t.tipo, t.descricao, t.realizada_em FROM public.transacoes t
                WHERE t.cliente_id = $1
                ORDER BY t.realizada_em DESC
                LIMIT 10
            "#,
        )
        .bind(costumer_id)
        .fetch_all(self.connection.as_ref())
        .await?;
        let transactions = transactions_query
            .iter()
            .map(|row| {
                let transaction: models::Transaction = row.try_into().unwrap();
                transaction
            })
            .collect();
        let account_statement = models::AccountStatement {
            balance,
            transactions,
        };
        Ok(account_statement)
    }
    pub async fn create_transaction(
        &self,
        costumer_id: i32,
        input: models::IncomeTransaction,
    ) -> Result<models::OutputTransaction, ApiError> {
        let (amount, r#type, description) = self.validate_input(&input)?;
        if costumer_id > 5 {
            return Err(ApiError::NotFound);
        }
        let balance_query = sqlx::query(
            r#"
            SELECT s.valor, c.limite FROM public.clientes c
            LEFT JOIN public.saldos s ON c.id = s.cliente_id
            WHERE c.id = $1
        "#,
        )
        .bind(costumer_id)
        .fetch_one(self.connection.as_ref())
        .await?;
        let mut customer: models::Customer = balance_query.try_into().unwrap();

        let transaction = models::Transaction {
            id: 1,
            amount,
            r#type,
            costumer_id,
            created_at: chrono::Utc::now().naive_utc(),
            description,
        };

        if transaction.r#type == "d" {
            let _ = self.debit_transaction(&mut customer, &transaction);
        }
        if transaction.r#type == "c" {
            self.credit_transaction(&mut customer, &transaction);
        }
        let _ = self.do_transaction(&customer, &transaction).await;

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
        customer: &models::Customer,
        transaction: &models::Transaction,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.connection.begin().await?;
        sqlx::query(
            r#"
                INSERT INTO public.transacoes (cliente_id, valor, tipo, descricao)
                VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(transaction.costumer_id)
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
        .bind(transaction.costumer_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
    fn validate_input(
        &self,
        input: &models::IncomeTransaction,
    ) -> Result<(i32, String, String), ApiError> {
        if input.amount.is_none() || input.r#type.is_none() || input.description.is_none() {
            return Err(ApiError::UnprocessableEntity);
        }
        let amount = input.amount.unwrap();
        let r#type = input.r#type.clone().unwrap();
        let description = input.description.clone().unwrap();
        if description.trim().len() > 10 || description.is_empty() {
            return Err(ApiError::UnprocessableEntity);
        }
        if r#type != "c" && r#type != "d" {
            return Err(ApiError::UnprocessableEntity);
        }
        if amount < 0 {
            return Err(ApiError::UnprocessableEntity);
        }

        Ok((amount, r#type, description))
    }
}
