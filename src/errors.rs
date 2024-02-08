use actix_web::body::BoxBody;
use actix_web::{HttpResponse, ResponseError};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    UnprocessableEntity,
    InternalServerError,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NotFound => write!(f, "Resource not found"),
            ApiError::UnprocessableEntity => write!(f, "Unprocessable entity"),
            ApiError::InternalServerError => write!(f, "Internal server error"),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ApiError::NotFound => actix_web::http::StatusCode::NOT_FOUND,
            ApiError::UnprocessableEntity => actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::InternalServerError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).finish()
    }
}
impl From<sqlx::Error> for ApiError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => ApiError::NotFound,
            _ => ApiError::InternalServerError,
        }
    }
}
