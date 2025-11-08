use actix_cors::Cors;
use actix_web::{App, HttpServer, web};

mod db;
mod models;
mod routes;
use db::Database;

use crate::routes::{auth, ballot, election};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let db = Database::init("./vote_db");

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(web::Data::new(db.clone())) // now web::Data<Database>
            .service(auth::routes())
            .service(election::routes())
            .service(ballot::routes())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
