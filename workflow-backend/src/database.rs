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
    // The check state will be a string of digits, each one stands for the state of one inspection
    // but the first digit stands for the version of this string rule.
    // for example, define order of inspections: <Version><Screen><Keyboard><Touchpad>
    // if they all works well, giving string 1111. otherwise, like, the touchpad can't work, then
    // the string should be 1101.
    initial_check_state: Option<String>,
    remedy: Option<String>,
    final_check_state: Option<String>,
    additional: Option<String>,
}

pub struct InitialState {
    location: String,
    staff: String,
    initial_check: String,
    remedy: String,
    post: String,
}

pub struct FinalState {
    final_check: String,
    additional: Option<String>,
    post: String,
}

impl DatabaseManager {
    pub fn build<P>(path: (P, P)) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let basic_info_connection = open_database(path.0)?;
        let detail_info_connection = open_database(path.1)?;

        let manager = DatabaseManager {
            basic_info_connection: RefCell::new(basic_info_connection),
            detail_info_connection: RefCell::new(detail_info_connection),
            id: RefCell::new(-1),
        };
        Self::load_database(&manager)?;

        Ok(manager)
    }

    fn load_database(manager: &DatabaseManager) -> Result<(), Error> {
        execute_database_command(
            &manager.basic_info_connection.borrow_mut(),
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
            &manager.detail_info_connection.borrow_mut(),
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

        let id = manager
            .basic_info_connection
            .borrow_mut()
            .query_one("select max(id) from clinic_task", [], |r| {
                r.get::<usize, i32>(0)
            })
            .unwrap_or(0); // Error occurs only when the table is empty
        manager.id.replace(id);

        Ok(())
    }

    fn create_task(&self, task: &Task) -> Result<(), Error> {
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

    pub fn update_at_initial(&mut self, state: InitialState) -> Result<(), Error> {
        self.location = Some(state.location);
        self.staff = Some(state.staff);
        self.remedy = Some(state.remedy);
        self.initial_post = Some(state.post);
        self.initial_check_state = Some(state.initial_check);
        self.update_to_database()
    }

    pub fn update_initial_confirm(&mut self, time: String) -> Result<(), Error> {
        self.initial_confirm = Some(time);
        self.update_to_database()
    }

    pub fn update_at_final(&mut self, state: FinalState) -> Result<(), Error> {
        self.final_check_state = Some(state.final_check);
        self.additional = state.additional;
        self.final_post = Some(state.post);
        self.update_to_database()
    }

    pub fn update_final_confirm(&mut self, time: String) -> Result<(), Error> {
        self.final_confirm = Some(time);
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
    use crate::database::{DatabaseManager, FinalState, InitialState, Task};
    use std::rc::Rc;
    const BASIC_DATABASE: &str = "./clinic_test.db";
    const DETAIL_DATABASE: &str = "./clinic_test_detail.db";

    #[test]
    fn test_task() {
        let db = DatabaseManager::build((BASIC_DATABASE, DETAIL_DATABASE)).unwrap();
        let db = Rc::new(db);
        let mut task = Task::build(db.clone()).unwrap();
        assert_ne!(task.id, -1);
        assert!(task.location.is_none());
        task.update_at_initial(InitialState {
            location: "qiushi".to_string(),
            staff: "potato".to_string(),
            initial_check: "1111".to_string(),
            remedy: "Change the CPU fan".to_string(),
            post: "202508121505".to_string(),
        })
        .unwrap();
        task.update_initial_confirm("202508121626".to_string())
            .unwrap();
        task.update_at_final(FinalState {
            final_check: "1111".to_string(),
            additional: None,
            post: "202508121711".to_string(),
        }).unwrap();
        task.update_final_confirm("202506121713".to_string()).unwrap();
    }
}
