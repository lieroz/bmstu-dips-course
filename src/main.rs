mod wallet;
mod config;

use actix_redis::RedisSession;
use actix_session::Session;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use listenfd::ListenFd;

use wallet::{Wallet, Currency};
use config::Config;

use std::sync::Arc;

fn create(session: Session) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(Wallet::new("wallet", 300, Currency::Rouble)))
}

fn read(session: Session) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(Wallet::new("wallet", 300, Currency::Rouble)))
}

fn update(session: Session) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(Wallet::new("wallet", 300, Currency::Rouble)))
}

fn delete(session: Session) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(Wallet::new("wallet", 300, Currency::Rouble)))
}

fn main() {
    let config = Arc::new(Config::new("/home/lieroz/bmstu-dips-course/config.yaml").unwrap());
    let thrConfig = config.clone();

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(RedisSession::new(format!("{}:{}", thrConfig.get_redis_host(),
                thrConfig.get_redis_port()), &[0; 32]))
            .service(
                web::scope("/api/wallet")
                    .route("/create", web::post().to(create))
                    .route("/read", web::get().to(read))
                    .route("/update", web::put().to(update))
                    .route("/delete", web::delete().to(delete))
            )
            .route("/", web::get().to(|| "Hello, World!"))
    })
    .workers(config.get_app_workers() as usize);

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind(format!("{}:{}", "127.0.0.1", config.get_app_port())).unwrap()
    };

    server.run().unwrap();
}
