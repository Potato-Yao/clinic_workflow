use rusqlite::{Connection, Params};
use std::cell::RefCell;
use std::io::{Error, ErrorKind};
use std::path::Path;

pub struct TaskManager {
    connection: RefCell<Connection>,
    id: RefCell<i32>,
}

#[derive(Default)]
pub struct Task {
    id: i32,
    location: Option<String>,
    staff: Option<String>,
    initial_post: Option<String>,
    initial_confirm: Option<String>,
    final_post: Option<String>,
    final_confirm: Option<String>,
}

impl TaskManager {
    pub fn build<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let connection = open_database(path).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to open database with error {e}"),
            )
        })?;

        Ok(TaskManager {
            connection: RefCell::new(connection),
            id: RefCell::new(-1),
        })
    }

    pub fn load_database(&self) -> Result<(), Error> {
        execute_database_command(
            &self.connection.borrow_mut(),
            "create table if not exists clinic_task(
                id integer primary key,
                location text,
                staff text,
                initial_post text,
                remedy text,
                initial_confirm text,
                final_post text,
                final_confirm text
            )",
            [],
        )
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to load the database with error {e}"),
            )
        })?;

        let id = self
            .connection
            .borrow_mut()
            .query_one("select max(id) as latest_id from clinic_task", [], |r| {
                r.get::<usize, i32>(0)
            })
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("Unable to get the latest id of the database with error {e}"),
                )
            })?;
        self.id.replace(id);

        Ok(())
    }

    pub fn create_task(&self, task: &Task) -> Result<(), Error> {
        if task.initial_post.is_none() {
            return Err(Error::new(ErrorKind::InvalidInput, ""));
        }
        execute_database_command(
            &self.connection.borrow_mut(),
            "insert into clinic_task (id, location, staff)
                                 values (?1, ?2, ?3)",
            (&task.id, &task.location, &task.staff),
        )
        .map(|_| ())
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to create a task with error {e}"),
            )
        })
    }

    fn get_id(&self) -> i32 {
        *self.id.borrow_mut() += 1;
        self.id.borrow().clone()
    }
}

impl Task {
    fn new(manager: &TaskManager) -> Self {
        Task {
            id: manager.get_id(),
            ..Default::default()
        }
    }
}

fn open_database<P>(path: P) -> Result<Connection, rusqlite::Error>
where
    P: AsRef<Path>,
{
    Connection::open(path)
}

fn execute_database_command<T, P>(
    connection: &Connection,
    command: T,
    params: P,
) -> Result<usize, rusqlite::Error>
where
    T: AsRef<str>,
    P: Params,
{
    connection.execute(command.as_ref(), params)
}

#[cfg(test)]
mod tests {
    use crate::database::{Task, TaskManager};
    const DATABASE_LOCATION: &str = "./clinic_test.db";

    #[test]
    fn database_load() {
        let db = TaskManager::build(DATABASE_LOCATION).unwrap();
        db.load_database().unwrap();
        assert_ne!(*db.id.borrow(), -1);
    }

    #[test]
    fn new_task() {
        let db = TaskManager::build(DATABASE_LOCATION).unwrap();
        let task = Task::new(&db);
        assert_ne!(task.id, -1);
    }
}
