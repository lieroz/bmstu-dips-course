mod wallet;
mod config;

use actix::prelude::*;
use actix_web::{web, middleware, App, Error as AWError, HttpResponse, HttpServer};
use listenfd::ListenFd;

use actix_redis::{Command, Error as ARError, RedisActor};
use redis_async::{resp_array, resp::{RespValue, FromResp}};

use wallet::Wallet;
use config::Config;

use std::sync::Arc;
use futures::future::Future;

use serde::Deserialize;

fn create(
    wallet: web::Json<Wallet>,
    redis: web::Data<Addr<RedisActor>>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let wallet = wallet.into_inner();
    redis.send(Command(
        resp_array!["HMSET", wallet.get_owner(),
            "balance", wallet.get_balance().to_string(),
            "currency", wallet.get_currency()])
        )
        .map_err(AWError::from)
        .and_then(|res: Result<RespValue, ARError>| match &res {
            Ok(RespValue::SimpleString(x)) if x == "OK" => {
                Ok(HttpResponse::Ok().body("success"))
            }
            _ => {
                Ok(HttpResponse::InternalServerError().finish())
            }
        })
}

fn read(
    owner: web::Path<String>,
    redis: web::Data<Addr<RedisActor>>,
) -> impl Future<Item = HttpResponse, Error = AWError> {
    let owner = owner.into_inner();
    redis.send(Command(
            resp_array!["HMGET", &owner, "balance", "currency"])
        )
        .map_err(AWError::from)
        .and_then(|res: Result<RespValue, ARError>| match &res {
            Ok(RespValue::Array(resps)) => {
                let wallet = Wallet::new(owner,
                    String::from_resp(resps[0].clone()).unwrap().parse::<u64>().unwrap(),
                    String::from_resp(resps[1].clone()).unwrap());
                Ok(HttpResponse::Ok().json(wallet))
            }
            _ => {
                Ok(HttpResponse::InternalServerError().finish())
            }
        })

}
/*
fn update() -> impl Future<Item = HttpResponse, Error = AWError> {
    Ok(HttpResponse::Ok().json(Wallet::new("wallet", 300, Currency::Rouble)))
}

fn delete() -> impl Future<Item = HttpResponse, Error = AWError> {
    Ok(HttpResponse::Ok().json(Wallet::new("wallet", 300, Currency::Rouble)))
}
*/
fn main() {
    let config = Arc::new(Config::new("/home/lieroz/bmstu-dips-course/config.yaml").unwrap());
    let thr_config = config.clone();

    std::env::set_var("RUST_LOG", "actix_web=debug,actix_redis=debug");
    env_logger::init();

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        let redis_addr = format!("{}:{}", thr_config.get_redis_host(),
                thr_config.get_redis_port());

        App::new()
            .data(RedisActor::start(redis_addr))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api/wallet")
                    .route("/create", web::post().to_async(create))
                    .route("/read/{owner}", web::get().to_async(read))
                    //.route("/update/{wallet}", web::put().to(update))
                    //.route("/delete/{wallet}", web::delete().to(delete))
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
