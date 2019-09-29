use actix_web::{web, HttpResponse};

use crate::handlers::tasks::*;

pub fn config_app(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(
                web::resource("/")
                    .route(web::get().to(|| HttpResponse::Ok())),
            )
            .service(
                web::resource("/create_task")
                    .route(web::post().to_async(create_task)),
            )
            .service(
                web::resource("/read_task/{id}")
                    .route(web::get().to_async(read_task)),
            )
            .service(
                web::resource("/update_task/{id}")
                    .route(web::put().to_async(update_task)),
            )
            .service(
                web::resource("/delete_task/{id}")
                    .route(web::delete().to_async(delete_task)),
            ),
    );
}
