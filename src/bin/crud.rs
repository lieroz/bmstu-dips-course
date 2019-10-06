use actix_web::{App, HttpServer, middleware::Logger};
use actix_redis::{RedisActor};
use listenfd::ListenFd;
use std::env;

use rsoi_lab1::appconfig::config_app;

fn main() {
    let port = env::var("PORT")
        .expect("error reading PORT from env");
    let redis_host = env::var("REDIS_HOST")
        .expect("error reading REDIS_HOST from env");
    let redis_port = env::var("REDIS_PORT")
        .expect("error reading REDIS_PORT from env");

    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .configure(config_app)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .data(RedisActor::start(format!("{}:{}", redis_host, redis_port)))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.workers(4)
            .bind(format!("0.0.0.0:{}", port))
            .unwrap()
    };

    server.run().unwrap();
}
