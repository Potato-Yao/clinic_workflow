use std::hash::{DefaultHasher, Hash, Hasher};
use crate::database::{DatabaseManager, InitialState, Task};
use actix_web::{HttpResponse, Responder, web};
use serde_json::json;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

const STAFF: &str = "/staff";
const CUSTOMER: &str = "/customer";

#[actix_web::post("/staff/create_task")]
pub async fn create_task(
    db_manager: web::Data<Arc<Mutex<DatabaseManager>>>,
    mes: web::Json<InitialState>,
) -> impl Responder {
    let manager = db_manager.lock().unwrap();
    let mut task = Task::build_new(&*manager).unwrap();
    task.update_at_initial(mes.into_inner()).unwrap();
    
    let uri = uri_generator(task.get_id(), 0, &*task.get_initial_post());
    let uri = format!("{}/{}", CUSTOMER, uri);

    HttpResponse::Ok().json(json!({
        "id": task.get_id(),
        "uri": uri,
    }))
}

#[actix_web::post("/staff")]
pub async fn complete_task(task: web::Json<Task<'_>>) -> impl Responder {
    let task = task.into_inner();

    format!("Task with id: {} created!", task.get_id())
}

fn uri_generator(a: i32, b: usize, c: &str) -> String {
    let mut hasher = DefaultHasher::new();
    a.hash(&mut hasher);
    b.hash(&mut hasher);
    c.hash(&mut hasher);

    format!("{:x}", hasher.finish())
}
