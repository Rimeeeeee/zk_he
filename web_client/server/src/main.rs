use actix_web::{web, App, HttpServer};
use std::sync::Arc;

mod routes;
mod db;
mod models;
use db::Database;

use crate::routes::{auth, ballot, election};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let db = Arc::new(Database::init("./vote_db"));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(auth::routes())
            .service(election::routes())
            .service(ballot::routes())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
