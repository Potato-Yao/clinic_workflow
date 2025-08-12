use std::path::Path;
use workflow_backend::database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_location = (Path::new("./clinic_basic.db"), Path::new("./clinic_detail.db"));
    let database_manager = database::DatabaseManager::build(database_location)?;


    Ok(())
}
