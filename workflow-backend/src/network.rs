use crate::database::{DatabaseManager, FinalState, InitialState, Task};
use actix_web::{HttpResponse, Responder, web};
use serde_json::json;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem::take;
use std::sync::{Arc, Mutex};

const STAFF: &str = "/staff";
const CUSTOMER: &str = "/customer";

#[actix_web::post("/staff/create_task")]
pub async fn post_initial_state(
    db_manager: web::Data<Arc<Mutex<DatabaseManager>>>,
    mes: web::Json<InitialState>,
) -> impl Responder {
    let manager = db_manager.lock().unwrap();
    let mut task = Task::build_new(&*manager).unwrap();
    task.update_at_initial(mes.into_inner()).unwrap();

    let uri = uri_generator(task.get_id(), 0, &*task.get_initial_post());
    let uri_customer = format!("{}/initial/x{}x{}", CUSTOMER, task.get_id(), uri);
    let uri_staff = format!("{}/x{}x{}", STAFF, task.get_id(), uri);

    HttpResponse::Ok().json(json!({
        "id": task.get_id(),
        "uri_customer": uri_customer,
        "uri_staff": uri_staff,
    }))
}

#[actix_web::post("/customer/initial/{path}/confirmed")]
pub async fn make_initial_confirm(
    path: web::Path<String>,
    db_manager: web::Data<Arc<Mutex<DatabaseManager>>>,
    mes: String,
) -> impl Responder {
    let path = path.into_inner();
    let id: i32 = path.split('x').nth(1).unwrap().parse().unwrap();
    let manager = db_manager.lock().unwrap();
    let mut task = manager.get_task_by_id(id).unwrap();
    task.update_initial_confirm(mes).unwrap();

    HttpResponse::Ok()
}

#[actix_web::post("/staff/{path}/finished")]
pub async fn post_final_state(
    path: web::Path<String>,
    db_manager: web::Data<Arc<Mutex<DatabaseManager>>>,
    mes: web::Json<FinalState>,
) -> impl Responder {
    let path = path.into_inner();
    let id: i32 = path.split('x').nth(1).unwrap().parse().unwrap();
    let manager = db_manager.lock().unwrap();
    let mut task = manager.get_task_by_id(id).unwrap();
    task.update_at_final(mes.into_inner()).unwrap();
    let uri = uri_generator(task.get_id(), 1, &*task.get_initial_post());
    let uri_customer = format!("{}/final/x{}x{}", CUSTOMER, task.get_id(), uri);

    HttpResponse::Ok().json(json!({
        "uri": uri_customer,
    }))
}

#[actix_web::post("/customer/final/{path}/confirmed")]
pub async fn make_final_confirm(
    path: web::Path<String>,
    db_manager: web::Data<Arc<Mutex<DatabaseManager>>>,
    mes: String,
) -> impl Responder {
    let path = path.into_inner();
    let id: i32 = path.split('x').nth(1).unwrap().parse().unwrap();
    let manager = db_manager.lock().unwrap();
    let mut task = manager.get_task_by_id(id).unwrap();
    task.update_final_confirm(mes).unwrap();

    HttpResponse::Ok()
}

fn uri_generator(a: i32, b: usize, c: &str) -> String {
    let mut hasher = DefaultHasher::new();
    a.hash(&mut hasher);
    b.hash(&mut hasher);
    c.hash(&mut hasher);

    format!("{:x}", hasher.finish())
}
