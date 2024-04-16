use rocket::State;
use rocket::tokio::sync::Mutex;
use rocket::serde::json::{Value, json, Json};
use rocket::serde::{Serialize, Deserialize};
use rocket::serde::uuid::Uuid;

type Id = usize;

type Tasks<'r> = &'r State<TaskList>;
type TaskList = Mutex<Vec<Task>>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Task {
  description: String,
  complete: bool,
  id: Uuid,
}

#[get("/list")]
async fn read(list: Tasks<'_>) -> Json<Tasks<'_>> {
  Json(list)
}

#[delete("/list/<id>")]
async fn delete(id: &str, list: Tasks<'_>) -> Option<Value> {
  let mut list = list.lock().await;
  let uuid = Uuid::try_parse(id);

  match uuid  {
    Ok(x) => {
      list.retain(|item| item.id == x);
      Some(json!({ "status": "ok" }))
    }
    Err(_) => None
  }
}

#[post("/list", format="json", data = "<task>")]
async fn create(task: Json<Task>, list: Tasks<'_>) -> Value {
  let mut list = list.lock().await;
  
  let new_task = Task {
    description: task.description.to_string(),
    complete: false,
    id: Uuid::new_v4(),
  };

  list.push(new_task);

  json!({ "status": "ok" })
}

#[put("/list/<id>", data = "<task>")]
async fn update(id: &str, task: Json<Task>, list: Tasks<'_>) -> Option<Value> {
  let list = list.lock().await.iter();

  let uuid = Uuid::try_parse(id);

  match uuid  {
      Ok(x) => {
        for item in list.into_iter() {
          if item.id == x {
            *item = task;
          }
        }
        Some(json!({ "status": "ok" }))
      }
      Err(_) => None
  }
}

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("JSON", |rocket| async {
        rocket.mount("/json", routes![create, update, read, delete])
            .register("/json", catchers![not_found])
            .manage(TaskList::new(Vec::new()))
    })
}