use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::web::JsonConfig;
use actix_web::{web, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use std::str::FromStr;

mod errors;
mod handlers;
mod models;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_pool = PgPoolOptions::new()
        .connect("postgres://admin:123@db:5432/rinha")
        .await
        .expect("Failed to connect to Postgres.");
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
    let mut route = web::scope("clientes").app_data(web::Data::new(db_pool));
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
