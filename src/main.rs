use actix_web::web::JsonConfig;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::net::TcpListener;
use std::str::FromStr;

mod errors;
mod handlers;
mod models;
mod queries;
mod services;

struct AppState {
    pub db: sqlx::PgPool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let port = env::var("PORT").unwrap();
    let db_pool = establish_connection("postgres://rinha:rinha@localhost:5432/rinha").await;
    let state = web::Data::new(AppState { db: db_pool });
    let port = u16::from_str(&port);
    let address = format!("0.0.0.0:{}", port.unwrap());
    let listener = TcpListener::bind(address).expect("Failed to bind random port");
    for _ in 0..50 {
        let pool = state.db.clone();
        let _ = sqlx::query("SELECT * FROM public.transacoes;")
            .execute(&pool)
            .await;
    }
    HttpServer::new(move || {
        App::new().configure(|cfg| {
            cfg.service(
                web::scope("/clientes")
                    .service(handlers::get_statements)
                    .service(handlers::create_transaction),
            )
            .app_data(error_parse())
            .app_data(state.clone());
        })
    })
    .shutdown_timeout(60)
    .listen(listener)
    .unwrap()
    .run()
    .await
    .expect("Failed to run server.");

    Ok(())
}
fn error_parse() -> JsonConfig {
    JsonConfig::default().error_handler(|err, _| {
        actix_web::error::InternalError::from_response(
            err,
            actix_web::HttpResponse::UnprocessableEntity().finish(),
        )
        .into()
    })
}

async fn establish_connection(database_url: &str) -> Pool<Postgres> {
    PgPoolOptions::new()
        .min_connections(30)
        .connect(database_url)
        .await
        .expect("Failed to connect to Postgres.")
}
