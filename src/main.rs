use actix::prelude::*;
use actix_web::{
    error, web, App,
    Error, HttpResponse, HttpServer,
    middleware::Logger,
};
use actix_redis::{Command, RedisActor};
use redis_async::{resp_array, resp::RespValue};
use futures::{future, Future};
use serde::{Deserialize, Serialize};
use listenfd::ListenFd;
use std::env;

#[derive(Serialize, Deserialize)]
struct Task {
    id: String,
    title: String,
    author: String,
    description: String,
}

#[derive(Serialize)]
struct Message {
    message: String,
}

fn create_task(
    task: web::Json<Task>,
    redis: web::Data<Addr<RedisActor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let task = task.into_inner();
    let id = task.id.clone();
    redis
        .send(Command(resp_array![
            "EXISTS",
            &id
        ]))
        .from_err()
        .and_then(move |res| match &res {
            Ok(RespValue::Integer(x)) => {
                if *x == 0 {
                    return future::err(error::ErrorNotFound(
                        format!("Task with id '{}' doesn't exist", &id)));
                }

                future::ok(HttpResponse::Conflict().json(
                    Message{ message: format!("Task with id '{}' already exists", &id) }))
            }
            _ => future::ok(HttpResponse::InternalServerError().finish())
        })
        .or_else(move |_| redis
            .send(Command(resp_array![
                "HMSET",
                &task.id,
                "title",
                &task.title,
                "author",
                &task.author,
                "description",
                &task.description
            ]))
            .from_err()
            .and_then(move |res| match &res {
                Ok(RespValue::SimpleString(x)) if x == "OK" => future::ok(HttpResponse::Created().finish()),
                _ => future::ok(HttpResponse::InternalServerError().finish())
            })
        )
}

fn read_task(
    info: web::Path::<(String,)>,
    redis: web::Data<Addr<RedisActor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = info.0.clone();
    redis
        .send(Command(resp_array![
            "EXISTS",
            &id
        ]))
        .from_err()
        .and_then(move |res| match &res {
            Ok(RespValue::Integer(x)) => {
                if *x == 0 {
                    return future::ok(HttpResponse::NotFound().json(
                        Message{ message: format!("Task with id '{}' doesn't exist", &id) }));
                }

                future::err(error::ErrorInternalServerError(""))
            }
            _ => future::ok(HttpResponse::InternalServerError().finish())
        })
        .or_else(move |_| redis
            .send(Command(resp_array![
                "HMGET",
                &info.0,
                "title",
                "author",
                "description"
            ]))
            .from_err()
            .and_then(move |res| match &res {
                Ok(RespValue::Array(arr)) => {
                    let mut vals = vec![];
                    for resp in arr {
                        let val = match resp {
                            RespValue::SimpleString(x) => Some(x.to_string()),
                            RespValue::BulkString(x) => {
                                Some(std::str::from_utf8(x).unwrap().to_string())
                            }
                            RespValue::Nil => None,
                            _ => None,
                        };

                        if let Some(val) = val {
                            vals.push(val);
                        }
                    }

                    if vals.len() != 3 {
                        return future::ok(HttpResponse::InternalServerError().finish());
                    }

                    future::ok(HttpResponse::Ok().json(
                        Task{
                            id: info.0.clone(),
                            title: vals[0].to_owned(),
                            author: vals[1].to_owned(),
                            description: vals[2].to_owned(),
                        }
                    ))
                }
                _ => future::ok(HttpResponse::InternalServerError().finish())
            })
        )
}

#[derive(Serialize, Deserialize)]
struct UpdateTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

fn update_task(
    info: web::Path::<(String,)>,
    task: web::Json<UpdateTask>,
    redis: web::Data<Addr<RedisActor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = info.0.clone();
    redis
        .send(Command(resp_array![
            "EXISTS",
            &id
        ]))
        .from_err()
        .and_then(move |res| match &res {
            Ok(RespValue::Integer(x)) => {
                if *x == 0 {
                    return future::ok(HttpResponse::NotFound().json(
                        Message{ message: format!("Task with id '{}' doesn't exist", &id) }));
                }

                future::err(error::ErrorInternalServerError(""))
            }
            _ => future::ok(HttpResponse::InternalServerError().finish()),
        })
        .or_else(move |_| {
            let task = task.into_inner();
            let mut data = vec![];

            if let Some(title) = task.title {
                if !title.is_empty() {
                    data.push("title".to_owned());
                    data.push(title);
                }
            };

            if let Some(author) = task.author {
                if !author.is_empty() {
                    data.push("author".to_owned());
                    data.push(author);
                }
            }

            if let Some(description) = task.description {
                if !description.is_empty() {
                    data.push("description".to_owned());
                    data.push(description);
                }
            }

            redis
                .send(Command(resp_array![
                    "HMSET",
                    &info.0
                ].append(&mut data)))
                .from_err()
                .and_then(move |res| match &res {
                    Ok(RespValue::SimpleString(x)) if x == "OK" => future::ok(HttpResponse::Ok().finish()),
                    _ => future::ok(HttpResponse::InternalServerError().finish()),
                })
        })
}

fn delete_task(
    info: web::Path::<(String,)>,
    redis: web::Data<Addr<RedisActor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let info = info.into_inner();
    redis
        .send(Command(resp_array![
            "DEL",
            &info.0
        ]))
        .from_err()
        .and_then(move |res| match &res {
            Ok(RespValue::Integer(x)) => {
                if *x == 0 {
                    return future::ok(HttpResponse::NotFound().json(
                        Message{ message: format!("Task with id '{}' wasn't found", info.0) }));
                }

                future::ok(HttpResponse::Ok().json(
                    Message{ message: format!("Task with id '{}' was deleted", info.0) }))
            }
            _ => future::ok(HttpResponse::InternalServerError().finish()),
        })
}

fn main() {
    let redis_url = env::var("REDIS_URL")
        .expect("error reading REDIS_URL from env");

    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .data(RedisActor::start(&redis_url))
            .route("/create_task", web::post().to_async(create_task))
            .route("/read_task/{id}", web::get().to_async(read_task))
            .route("/update_task/{id}", web::put().to_async(update_task))
            .route("/delete_task/{id}", web::delete().to_async(delete_task))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.workers(4)
            .bind(format!("0.0.0.0:8080"))
            .unwrap()
    };

    server.run().unwrap();
}

