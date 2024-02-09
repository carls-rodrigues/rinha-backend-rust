use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Customer {
    pub limit: i32,
    pub balance: i32,
}
impl TryFrom<PgRow> for Customer {
    type Error = sqlx::Error;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        Ok(Customer {
            limit: row.try_get("limite")?,
            balance: row.try_get("valor")?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    #[serde(skip_serializing)]
    pub id: i32,
    #[serde(skip_serializing)]
    pub costumer_id: i32,
    #[serde(rename = "valor")]
    pub amount: i32,
    #[serde(rename = "tipo")]
    pub r#type: String,
    #[serde(rename = "descricao")]
    pub description: String,
    #[serde(rename = "realizada_em")]
    pub created_at: NaiveDateTime,
}

impl TryFrom<&PgRow> for Transaction {
    type Error = sqlx::Error;

    fn try_from(value: &PgRow) -> Result<Self, Self::Error> {
        Ok(Transaction {
            id: value.try_get("id")?,
            costumer_id: value.try_get("cliente_id")?,
            amount: value.try_get("valor")?,
            r#type: value.try_get("tipo")?,
            description: value.try_get("descricao")?,
            created_at: value.try_get("realizada_em")?,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct AccountBalance {
    pub id: i32,
    pub costumer_id: i32,
    #[serde(rename = "saldo")]
    pub balance: i32,
}

impl TryFrom<PgRow> for AccountBalance {
    type Error = sqlx::Error;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        Ok(AccountBalance {
            id: 1,
            costumer_id: 1,
            balance: row.try_get("valor")?,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct IncomeTransaction {
    #[serde(rename = "valor")]
    pub amount: Option<i32>,
    #[serde(rename = "tipo")]
    pub r#type: Option<String>,
    #[serde(rename = "descricao")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct OutputTransaction {
    #[serde(rename = "limite")]
    pub limit: i32,
    #[serde(rename = "saldo")]
    pub balance: i32,
}

#[derive(Serialize, Deserialize)]
pub struct StatementOutput {
    pub total: i32,
    #[serde(rename = "data_extrato")]
    pub created_at: NaiveDateTime,
    #[serde(rename = "limite")]
    pub limit: i32,
}

impl TryFrom<PgRow> for StatementOutput {
    type Error = sqlx::Error;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        Ok(StatementOutput {
            total: row.try_get("valor")?,
            created_at: Utc::now().naive_utc(),
            limit: row.try_get("limite")?,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct AccountStatement {
    #[serde(rename = "saldo")]
    pub balance: StatementOutput,
    #[serde(rename = "ultimas_transacoes")]
    pub transactions: Vec<Transaction>,
}
