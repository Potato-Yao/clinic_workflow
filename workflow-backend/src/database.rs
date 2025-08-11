use rusqlite::{Connection, Params};
use std::io::{Error, ErrorKind};
use std::path::Path;

pub struct TaskManager {
    connection: Connection,
    id: i32,
}

pub struct Task {
    id: i32,
    location: String,
    staff: String,
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

        Ok(TaskManager { connection, id: -1 })
    }

    pub fn load_database(&mut self) -> Result<(), Error> {
        execute_database_command(
            &self.connection,
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

        self.id = self
            .connection
            .query_one("select max(id) as latest_id from clinic_task", [], |r| {
                r.get::<usize, i32>(0)
            })
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("Unable to get the latest id of the database with error {e}"),
                )
            })?;

        Ok(())
    }

    pub fn create_task(&self, task: &Task) -> Result<(), Error> {
        if task.initial_post.is_none() {
            return Err(Error::new(ErrorKind::InvalidInput, ""));
        }
        execute_database_command(
            &self.connection,
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
}

impl Task {
    // pub fn new() -> Self {
    //
    // }
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
    use crate::database::TaskManager;
    const DATABASE_LOCATION: &str = "./clinic_test.db";

    #[test]
    fn database_load() -> Result<(), ()> {
        let mut db = TaskManager::build(DATABASE_LOCATION).unwrap();
        db.load_database().unwrap();
        assert_ne!(db.id, -1);

        Ok(())
    }
}
