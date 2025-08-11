use std::path::Path;
use workflow_backend::database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_location = Path::new("./clinic.db");
    let task_manager = database::TaskManager::build(database_location)?;


    Ok(())
}
