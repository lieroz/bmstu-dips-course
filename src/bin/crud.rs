use actix_web::{App, HttpServer, middleware::Logger};
use actix_redis::{RedisActor};
use listenfd::ListenFd;
use std::env;

use bmstu_dips_course::appconfig::config_app;

fn main() {
    let port = env::var("PORT")
        .expect("error reading PORT from env");
    let redis_url = env::var("REDIS_URL")
        .expect("error reading REDIS_URL from env");

    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .configure(config_app)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .data(RedisActor::start(&redis_url))
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
