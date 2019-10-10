use actix::prelude::*;
use actix_redis::{Command, RedisActor};
use actix_web::{error, web, Error, HttpResponse};
use futures::{future, Future};
use redis_async::{resp::RespValue, resp_array};

use crate::common::{Message, Task, UpdateTask};

pub fn create_task(
    task: web::Json<Task>,
    redis: web::Data<Addr<RedisActor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let task = task.into_inner();
    let id = task.id.clone();
    redis
        .send(Command(resp_array!["EXISTS", &id]))
        .from_err()
        .and_then(move |res| match &res {
            Ok(RespValue::Integer(x)) => {
                if *x == 0 {
                    return future::err(error::ErrorNotFound(format!(
                        "Task with id '{}' doesn't exist",
                        &id
                    )));
                }

                future::ok(HttpResponse::Conflict().json(Message {
                    message: format!("Task with id '{}' already exists", &id),
                }))
            }
            _ => future::ok(HttpResponse::InternalServerError().finish()),
        })
        .or_else(move |_| {
            redis
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
                    Ok(RespValue::SimpleString(x)) if x == "OK" => {
                        future::ok(HttpResponse::Created().finish())
                    }
                    _ => future::ok(HttpResponse::InternalServerError().finish()),
                })
        })
}

pub fn read_task(
    info: web::Path<(String,)>,
    redis: web::Data<Addr<RedisActor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = info.0.clone();
    redis
        .send(Command(resp_array!["EXISTS", &id]))
        .from_err()
        .and_then(move |res| match &res {
            Ok(RespValue::Integer(x)) => {
                if *x == 0 {
                    return future::ok(HttpResponse::NotFound().json(Message {
                        message: format!("Task with id '{}' doesn't exist", &id),
                    }));
                }

                future::err(error::ErrorInternalServerError(""))
            }
            _ => future::ok(HttpResponse::InternalServerError().finish()),
        })
        .or_else(move |_| {
            redis
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

                        future::ok(HttpResponse::Ok().json(Task {
                            id: info.0.clone(),
                            title: vals[0].to_owned(),
                            author: vals[1].to_owned(),
                            description: vals[2].to_owned(),
                        }))
                    }
                    _ => future::ok(HttpResponse::InternalServerError().finish()),
                })
        })
}

pub fn update_task(
    info: web::Path<(String,)>,
    task: web::Json<UpdateTask>,
    redis: web::Data<Addr<RedisActor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = info.0.clone();
    redis
        .send(Command(resp_array!["EXISTS", &id]))
        .from_err()
        .and_then(move |res| match &res {
            Ok(RespValue::Integer(x)) => {
                if *x == 0 {
                    return future::ok(HttpResponse::NotFound().json(Message {
                        message: format!("Task with id '{}' doesn't exist", &id),
                    }));
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
                .send(Command(resp_array!["HMSET", &info.0].append(&mut data)))
                .from_err()
                .and_then(move |res| match &res {
                    Ok(RespValue::SimpleString(x)) if x == "OK" => {
                        future::ok(HttpResponse::Ok().finish())
                    }
                    _ => future::ok(HttpResponse::InternalServerError().finish()),
                })
        })
}

pub fn delete_task(
    info: web::Path<(String,)>,
    redis: web::Data<Addr<RedisActor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let info = info.into_inner();
    redis
        .send(Command(resp_array!["DEL", &info.0]))
        .from_err()
        .and_then(move |res| match &res {
            Ok(RespValue::Integer(x)) => {
                if *x == 0 {
                    return future::ok(HttpResponse::NotFound().json(Message {
                        message: format!("Task with id '{}' wasn't found", info.0),
                    }));
                }

                future::ok(HttpResponse::Ok().json(Message {
                    message: format!("Task with id '{}' was deleted", info.0),
                }))
            }
            _ => future::ok(HttpResponse::InternalServerError().finish()),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::appconfig::config_app;
    use actix_service::Service;
    use actix_web::{
        http::{header, StatusCode},
        test, App,
    };

    #[test]
    fn test_create_task_created() {
        let redis_addr = test::run_on(|| RedisActor::start("127.0.0.1:6379"));
        let mut app = test::init_service(App::new().configure(config_app).data(redis_addr));

        let payload = r#"{"id":"create_created","title":"Test Task",
            "author":"somebody","description":"Simple task"}"#
            .as_bytes();

        let req = test::TestRequest::post()
            .uri("/create_task")
            .header(header::CONTENT_TYPE, "application/json")
            .set_payload(payload)
            .to_request();

        let resp = test::block_fn(|| app.call(req)).unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let _ = test::block_fn(|| {
            RedisActor::start("127.0.0.1:6379").send(Command(resp_array!["DEL", "create_created"]))
        });
    }

    #[test]
    fn test_create_task_conflict() {
        let redis_addr = test::run_on(|| RedisActor::start("127.0.0.1:6379"));
        let mut app = test::init_service(App::new().configure(config_app).data(redis_addr));

        let payload = r#"{"id":"create_conflict","title":"Test Task",
            "author":"somebody","description":"Simple task"}"#
            .as_bytes();

        let req = test::TestRequest::post()
            .uri("/create_task")
            .header(header::CONTENT_TYPE, "application/json")
            .set_payload(payload.clone())
            .to_request();

        let resp = test::block_fn(|| app.call(req)).unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let redis_addr = test::run_on(|| RedisActor::start("127.0.0.1:6379"));
        let mut app = test::init_service(App::new().configure(config_app).data(redis_addr));

        let req = test::TestRequest::post()
            .uri("/create_task")
            .header(header::CONTENT_TYPE, "application/json")
            .set_payload(payload)
            .to_request();

        let resp = test::block_fn(|| app.call(req)).unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);

        let _ = test::block_fn(|| {
            RedisActor::start("127.0.0.1:6379").send(Command(resp_array!["DEL", "create_conflict"]))
        });
    }

    #[test]
    fn test_read_task_ok() {
        let redis_addr = test::run_on(|| RedisActor::start("127.0.0.1:6379"));
        let mut app = test::init_service(App::new().configure(config_app).data(redis_addr));

        let payload = r#"{"id":"read_ok","title":"Test Task",
            "author":"somebody","description":"Simple task"}"#
            .as_bytes();

        let req = test::TestRequest::post()
            .uri("/create_task")
            .header(header::CONTENT_TYPE, "application/json")
            .set_payload(payload)
            .to_request();

        let resp = test::block_fn(|| app.call(req)).unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let redis_addr = test::run_on(|| RedisActor::start("127.0.0.1:6379"));
        let mut app = test::init_service(App::new().configure(config_app).data(redis_addr));

        let req = test::TestRequest::get()
            .uri("/read_task/read_ok")
            .to_request();

        let resp = test::block_fn(|| app.call(req)).unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let _ = test::block_fn(|| {
            RedisActor::start("127.0.0.1:6379").send(Command(resp_array!["DEL", "read_ok"]))
        });
    }

    #[test]
    fn test_read_task_not_found() {
        let redis_addr = test::run_on(|| RedisActor::start("127.0.0.1:6379"));
        let mut app = test::init_service(App::new().configure(config_app).data(redis_addr));

        let req = test::TestRequest::get()
            .uri("/read_task/read_not_found")
            .to_request();

        let resp = test::block_fn(|| app.call(req)).unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_update_task_ok() {
        let redis_addr = test::run_on(|| RedisActor::start("127.0.0.1:6379"));
        let mut app = test::init_service(App::new().configure(config_app).data(redis_addr));

        let payload = r#"{"id":"update_ok","title":"Test Task",
            "author":"somebody","description":"Simple task"}"#
            .as_bytes();

        let req = test::TestRequest::post()
            .uri("/create_task")
            .header(header::CONTENT_TYPE, "application/json")
            .set_payload(payload)
            .to_request();

        let resp = test::block_fn(|| app.call(req)).unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let redis_addr = test::run_on(|| RedisActor::start("127.0.0.1:6379"));
        let mut app = test::init_service(App::new().configure(config_app).data(redis_addr));

        let payload = r#"{"title":"New Test Task"}"#.as_bytes();

        let req = test::TestRequest::put()
            .uri("/update_task/update_ok")
            .header(header::CONTENT_TYPE, "application/json")
            .set_payload(payload)
            .to_request();

        let resp = test::block_fn(|| app.call(req)).unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let _ = test::block_fn(|| {
            RedisActor::start("127.0.0.1:6379").send(Command(resp_array!["DEL", "update_ok"]))
        });
    }

    #[test]
    fn test_update_task_not_found() {
        let redis_addr = test::run_on(|| RedisActor::start("127.0.0.1:6379"));
        let mut app = test::init_service(App::new().configure(config_app).data(redis_addr));

        let payload = r#"{"title":"New Test Task"}"#.as_bytes();

        let req = test::TestRequest::put()
            .uri("/update_task/update_not_found")
            .header(header::CONTENT_TYPE, "application/json")
            .set_payload(payload)
            .to_request();

        let resp = test::block_fn(|| app.call(req)).unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_delete_task_ok() {
        let redis_addr = test::run_on(|| RedisActor::start("127.0.0.1:6379"));
        let mut app = test::init_service(App::new().configure(config_app).data(redis_addr));

        let payload = r#"{"id":"delete_ok","title":"Test Task",
            "author":"somebody","description":"Simple task"}"#
            .as_bytes();

        let req = test::TestRequest::post()
            .uri("/create_task")
            .header(header::CONTENT_TYPE, "application/json")
            .set_payload(payload)
            .to_request();

        let resp = test::block_fn(|| app.call(req)).unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let redis_addr = test::run_on(|| RedisActor::start("127.0.0.1:6379"));
        let mut app = test::init_service(App::new().configure(config_app).data(redis_addr));

        let req = test::TestRequest::delete()
            .uri("/delete_task/delete_ok")
            .to_request();

        let resp = test::block_fn(|| app.call(req)).unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let _ = test::block_fn(|| {
            RedisActor::start("127.0.0.1:6379").send(Command(resp_array!["DEL", "delete_ok"]))
        });
    }

    #[test]
    fn test_delete_task_not_found() {
        let redis_addr = test::run_on(|| RedisActor::start("127.0.0.1:6379"));
        let mut app = test::init_service(App::new().configure(config_app).data(redis_addr));

        let req = test::TestRequest::delete()
            .uri("/delete_task/delete_not_found")
            .to_request();

        let resp = test::block_fn(|| app.call(req)).unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
