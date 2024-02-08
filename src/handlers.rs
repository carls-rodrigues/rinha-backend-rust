use crate::{models, services::Services};
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use crate::errors::ApiError;

pub async fn get_statements(
    db_pool: web::Data<PgPool>,
    costumer_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let services = Services {
        connection: Box::new(db_pool.get_ref().to_owned()),
    };
    let fetch_all = services.get_statement(costumer_id.into_inner()).await;
    match fetch_all {
        Ok(statement) => Ok(HttpResponse::Ok().json(statement)),
        Err(err) => Err(err)
    }
}

pub async fn create_transaction(
    db_pool: web::Data<PgPool>,
    costumer_id: web::Path<i32>,
    input: web::Json<models::IncomeTransaction>,
) -> Result<HttpResponse, ApiError> {
    let services = Services {
        connection: Box::new(db_pool.get_ref().to_owned()),
    };
    let create_transaction = services
        .create_transaction(costumer_id.into_inner(), input.into_inner())
        .await;
    match create_transaction {
        Ok(transaction) => Ok(HttpResponse::Ok().json(transaction)),
        Err(err) => Err(err),
    }
}
