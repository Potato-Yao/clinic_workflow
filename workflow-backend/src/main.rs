use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use std::path::Path;
use std::sync::{Arc, Mutex};
use workflow_backend::database;
use workflow_backend::network::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "actix_web=debug");
    }
    env_logger::init();
    #[cfg(debug_assertions)]
    let database_location = (
        Path::new("./clinic_test.db"),
        Path::new("./clinic_test_detail.db"),
    );
    #[cfg(not(debug_assertions))]
    let database_location = (
        Path::new("./clinic_basic.db"),
        Path::new("./clinic_detail.db"),
    );
    let database_manager = database::DatabaseManager::build(database_location)?;
    let db = web::Data::new(Arc::new(Mutex::new(database_manager)));

    let _ = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(db.clone())
            .service(create_task)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await;

    Ok(())
}
