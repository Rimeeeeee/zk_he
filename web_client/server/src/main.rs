use actix_cors::Cors;
use actix_web::{
    App, HttpServer,
    middleware::Logger,
    web::{self},
};

mod db;
mod models;
mod routes;
use db::Database;

use crate::routes::{auth, election, key};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let db = Database::init("./vote_db");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(web::Data::new(db.clone()))
            .service(auth::routes())
            .service(election::routes())
            .service(key::routes())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
