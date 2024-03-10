use crate::errors::ApiError;
use crate::AppState;
use crate::{models, services::Services};
use actix_web::{get, post, web, HttpResponse};

#[get("/{costumer_id}/extrato")]
pub async fn get_statements(
    state: web::Data<AppState>,
    customer_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let customer_id = customer_id.into_inner();
    if customer_id > 5 {
        return Err(ApiError::NotFound);
    }
    let services = Services::new(state);
    let fetch_all = services.get_statement(customer_id).await;
    match fetch_all {
        Ok(statement) => Ok(HttpResponse::Ok().json(statement)),
        Err(err) => Err(err),
    }
}
#[post("/{costumer_id}/transacoes")]
pub async fn create_transaction(
    state: web::Data<AppState>,
    customer_id: web::Path<i32>,
    input: web::Json<models::IncomeTransaction>,
) -> Result<HttpResponse, ApiError> {
    let customer_id = customer_id.into_inner();
    if customer_id > 5 {
        return Err(ApiError::NotFound);
    }
    let services = Services::new(state);
    let is_transaction_invalid = || {
        let transaction_type = input.r#type != "d" && input.r#type != "c";
        let transaciton = input.amount < 0;
        let description = input.description.is_empty() || input.description.len() > 10;
        transaction_type || transaciton || description
    };
    if is_transaction_invalid() {
        return Err(ApiError::UnprocessableEntity);
    }

    let create_transaction = services
        .create_transaction(customer_id, input.into_inner())
        .await;
    match create_transaction {
        Ok(transaction) => Ok(HttpResponse::Ok().json(transaction)),
        Err(err) => Err(err),
    }
}
