use std::path::Path;
use workflow_backend::database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_location = Path::new("./clinic.db");
    let database = database::TaskManager::build(database_location)?;


    Ok(())
}
