use rusqlite::{Connection, Params};
use std::cell::RefCell;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::rc::Rc;

pub struct DatabaseManager {
    basic_info_connection: RefCell<Connection>,
    detail_info_connection: RefCell<Connection>,
    id: RefCell<i32>,
}

pub struct Task {
    __manager: Rc<DatabaseManager>,
    id: i32,
    location: Option<String>,
    staff: Option<String>,
    initial_post: Option<String>,
    initial_confirm: Option<String>,
    final_post: Option<String>,
    final_confirm: Option<String>,
    initial_check_state: Option<String>,
    remedy: Option<String>,
    final_check_state: Option<String>,
    additional: Option<String>,
}

pub struct InitialState {
    location: String,
    staff: String,
    remedy: String,
    post: String,
}

impl DatabaseManager {
    pub fn build<P>(path: (P, P)) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let basic_info_connection = open_database(path.0)?;
        let detail_info_connection = open_database(path.1)?;

        Ok(DatabaseManager {
            basic_info_connection: RefCell::new(basic_info_connection),
            detail_info_connection: RefCell::new(detail_info_connection),
            id: RefCell::new(-1),
        })
    }

    pub fn load_database(&self) -> Result<(), Error> {
        execute_database_command(
            &self.basic_info_connection.borrow_mut(),
            "create table if not exists clinic_task(
                id integer primary key,
                location text,
                staff text,
                initial_post text,
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

        execute_database_command(
            &self.detail_info_connection.borrow_mut(),
            "create table if not exists clinic_task(
                id integer primary key,
                initial_check_state text,
                remedy text,
                final_check_state text,
                additional text
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
            .basic_info_connection
            .borrow_mut()
            .query_one("select max(id) from clinic_task", [], |r| {
                r.get::<usize, i32>(0)
            })
            .unwrap_or(0); // Error occurs only when the table is empty
        self.id.replace(id);

        Ok(())
    }

    pub fn create_task(&self, task: &Task) -> Result<(), Error> {
        execute_database_command(
            &self.basic_info_connection.borrow_mut(),
            "insert into clinic_task (id) values (?1)",
            [&task.id],
        )
        .map(|_| {})?;

        execute_database_command(
            &self.detail_info_connection.borrow_mut(),
            "insert into clinic_task (id) values (?1)",
            [&task.id],
        )
        .map(|_| {})
    }

    fn update_from_task(&self, task: &Task) -> Result<(), Error> {
        let none = String::from("None");
        execute_database_command(
            &self.basic_info_connection.borrow_mut(),
            "update clinic_task set location = ?1, staff = ?2, initial_post = ?3, initial_confirm = ?4, final_post = ?5, final_confirm = ?6 where id = ?7",
            (
                &task.location.as_ref().unwrap_or(&none),
                &task.staff.as_ref().unwrap_or(&none),
                &task.initial_post.as_ref().unwrap_or(&none),
                &task.initial_confirm.as_ref().unwrap_or(&none),
                &task.final_post.as_ref().unwrap_or(&none),
                &task.final_confirm.as_ref().unwrap_or(&none),
                &task.id,
            ),
        )?;
        execute_database_command(
            &self.detail_info_connection.borrow_mut(),
            "update clinic_task set initial_check_state = ?1, remedy = ?2, final_check_state = ?3, additional = ?4 where id = ?5",
            (&task.initial_check_state.as_ref().unwrap_or(&none), &task.remedy.as_ref().unwrap_or(&none), &task.final_check_state.as_ref().unwrap_or(&none), &task.additional.as_ref().unwrap_or(&none), &task.id),
        ).map(|_| {})
    }

    fn get_id(&self) -> i32 {
        *self.id.borrow_mut() += 1;
        self.id.borrow().clone()
    }
}

impl Task {
    pub fn build(manager: Rc<DatabaseManager>) -> Result<Self, Error> {
        let task = Task {
            __manager: Rc::clone(&manager),
            id: manager.get_id(),
            location: None,
            staff: None,
            initial_post: None,
            initial_confirm: None,
            final_post: None,
            final_confirm: None,
            initial_check_state: None,
            remedy: None,
            final_check_state: None,
            additional: None,
        };
        manager.create_task(&task)?;

        Ok(task)
    }

    pub fn update_initial_state(&mut self, state: InitialState) -> Result<(), Error> {
        self.location = Some(state.location);
        self.staff = Some(state.staff);
        self.remedy = Some(state.remedy);
        self.initial_post = Some(state.post);
        self.update_to_database()
    }

    fn update_to_database(&self) -> Result<(), Error> {
        self.__manager.update_from_task(&self)
    }
}

fn open_database<P>(path: P) -> Result<Connection, Error>
where
    P: AsRef<Path>,
{
    Connection::open(path).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Can't open database with error {e}"),
        )
    })
}

fn execute_database_command<T, P>(
    connection: &Connection,
    command: T,
    params: P,
) -> Result<usize, Error>
where
    T: AsRef<str>,
    P: Params,
{
    connection
        .execute(command.as_ref(), params)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Database error with {e}")))
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use crate::database::{DatabaseManager, InitialState, Task};
    const BASIC_DATABASE: &str = "./clinic_test.db";
    const DETAIL_DATABASE: &str = "./clinic_test_detail.db";

    #[test]
    fn database_load() {
        let db = DatabaseManager::build((BASIC_DATABASE, DETAIL_DATABASE)).unwrap();
        db.load_database().unwrap();
        assert_ne!(*db.id.borrow(), -1);
    }

    #[test]
    fn new_task() {
        let db = DatabaseManager::build((BASIC_DATABASE, DETAIL_DATABASE)).unwrap();
        db.load_database().unwrap();
        let db = Rc::new(db);
        let task = Task::build(db.clone()).unwrap();
        assert_ne!(task.id, -1);
        assert!(task.location.is_none());
        db.create_task(&task).unwrap();
    }

    #[test]
    fn test_task() {
        let db = DatabaseManager::build((BASIC_DATABASE, DETAIL_DATABASE)).unwrap();
        db.load_database().unwrap();
        let db = Rc::new(db);
        let mut task = Task::build(db.clone()).unwrap();
        task.update_initial_state(InitialState {
            location: "qiushi".to_string(),
            staff: "potato".to_string(),
            remedy: "Change the CPU fan".to_string(),
            post: "202508121505".to_string(),
        }).unwrap();
    }
}
