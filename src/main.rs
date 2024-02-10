use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::web::JsonConfig;
use actix_web::{web, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Pool, Postgres};
use std::net::TcpListener;
use std::str::FromStr;

mod errors;
mod handlers;
mod models;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // TODO: move database url to env variable and check current environment
    let db_pool = establish_connection("postgres://rinha:rinha@db:5432/rinha").await;
    let port = u16::from_str("8080");
    let address = format!("0.0.0.0:{}", port.unwrap());
    let listener = TcpListener::bind(address).expect("Failed to bind random port");
    let server = HttpServer::new(move || mk_app(db_pool.clone()))
        .shutdown_timeout(60)
        .listen(listener)
        .unwrap()
        .run();
    server.await.expect("Failed to start server");
    Ok(())
}
pub fn mk_app(
    db_pool: PgPool,
) -> actix_web::App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<impl MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let mut server_app = actix_web::App::new();
    server_app = server_app.app_data(JsonConfig::default().error_handler(|err, _| {
        actix_web::error::InternalError::from_response(
            err,
            actix_web::HttpResponse::UnprocessableEntity().finish(),
        )
        .into()
    }));
    let mut route = web::scope("clientes")
        .guard(actix_web::guard::fn_guard(|ctx| {
            let binding = ctx.head().uri.to_string();
            let url = binding.split('/').collect::<Vec<&str>>();
            let parse_int = url[2].parse::<i32>();
            if let Ok(customer_id) = parse_int {
                return customer_id <= 5;
            }
            false
        }))
        .app_data(web::Data::new(db_pool));
    route = route.service(
        web::resource("/{costumer_id}/extrato").route(web::get().to(handlers::get_statements)),
    );
    route = route.service(
        web::resource("/{costumer_id}/transacoes")
            .route(web::post().to(handlers::create_transaction)),
    );
    server_app = server_app.service(route);
    server_app
}

async fn establish_connection(database_url: &str) -> Pool<Postgres> {
    PgPoolOptions::new()
        .connect(database_url)
        .await
        .expect("Failed to connect to Postgres.")
}
