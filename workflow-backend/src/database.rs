use crate::DATABASE_NONE_PLACER;
use rusqlite::{Connection, Params};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::io::{Error, ErrorKind};
use std::path::Path;

pub struct DatabaseManager {
    basic_info_connection: RefCell<Connection>,
    detail_info_connection: RefCell<Connection>,
    id: RefCell<i32>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Task<'a> {
    #[serde(skip_serializing, skip_deserializing)]
    __manager: Option<&'a DatabaseManager>, // it should never be None, it's just for Default
    id: i32,
    location: Option<String>,
    staff: Option<String>,
    customer: Option<String>,
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

#[derive(Deserialize, Debug)]
pub struct InitialState {
    location: String,
    staff: String,
    customer: String,
    initial_check: String,
    remedy: String,
    post: String,
}

#[derive(Deserialize)]
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
                customer text,
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
        let none = String::from(DATABASE_NONE_PLACER);
        execute_database_command(
            &self.basic_info_connection.borrow_mut(),
            "update clinic_task set location = ?1, staff = ?2, customer = ?3, initial_post = ?4, initial_confirm = ?5, final_post = ?6, final_confirm = ?7 where id = ?8",
            (
                &task.location.as_ref().unwrap_or(&none),
                &task.staff.as_ref().unwrap_or(&none),
                &task.customer.as_ref().unwrap_or(&none),
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

    pub fn get_task_by_id(&'_ self, id: i32) -> Result<Task<'_>, Error> {
        let mut task = Task::new(&self);
        task.id = id;
        self.basic_info_connection.borrow_mut().query_row(
            "select location, staff, customer, initial_post, initial_confirm, final_post, final_confirm from clinic_task where id=?1",
            [id],
            |r| {
                task.location = Self::option_string_replacer(r.get(0)?); // column of primary key doesn't  count
                task.staff = Self::option_string_replacer(r.get(1)?);
                task.customer = Self::option_string_replacer(r.get(2)?);
                task.initial_post = Self::option_string_replacer(r.get(3)?);
                task.initial_confirm = Self::option_string_replacer(r.get(4)?);
                task.final_post = Self::option_string_replacer(r.get(5)?);
                task.final_confirm = Self::option_string_replacer(r.get(6)?);

                Ok(())
            }
        ).map_err(|e| {
            Error::new(ErrorKind::Other, format!("Can't execute query on basic database with error {e}"))
        })?;
        self.detail_info_connection.borrow_mut().query_row(
            "select initial_check_state, remedy, final_check_state, additional from clinic_task where id = ?1",
            [id],
            |r| {
                task.initial_check_state = Self::option_string_replacer(r.get(0)?); // column of primary key doesn't  count
                task.remedy = Self::option_string_replacer(r.get(1)?);
                task.final_check_state = Self::option_string_replacer(r.get(2)?);
                task.additional = Self::option_string_replacer(r.get(3)?);

                Ok(())
            }
        ).map_err(|e| {
            Error::new(ErrorKind::Other, format!("Can't execute query on detailed database with error {e}"))
        })?;

        Ok(task)
    }

    fn option_string_replacer(s: String) -> Option<String> {
        if s == DATABASE_NONE_PLACER {
            return None;
        }
        Some(s)
    }

    fn get_id(&self) -> i32 {
        *self.id.borrow_mut() += 1;
        self.id.borrow().clone()
    }
}

impl<'a> Task<'a> {
    pub fn new(manager: &'a DatabaseManager) -> Self {
        Task {
            __manager: Some(manager),
            ..Default::default()
        }
    }

    pub fn build_new(manager: &'a DatabaseManager) -> Result<Self, Error> {
        let task = Task {
            __manager: Some(manager),
            id: manager.get_id(),
            ..Default::default()
        };
        manager.create_task(&task)?;

        Ok(task)
    }

    pub fn update_at_initial(&mut self, state: InitialState) -> Result<(), Error> {
        self.location = Some(state.location);
        self.staff = Some(state.staff);
        self.customer = Some(state.customer);
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

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_initial_post(&self) -> String {
        if let Some(s) = &self.initial_post {
            return s.clone();
        }
        String::from(DATABASE_NONE_PLACER)
    }

    fn update_to_database(&self) -> Result<(), Error> {
        self.__manager.as_ref().unwrap().update_from_task(&self)
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
    const BASIC_DATABASE: &str = "./clinic_test.db";
    const DETAIL_DATABASE: &str = "./clinic_test_detail.db";

    #[test]
    fn test_task() {
        let db = DatabaseManager::build((BASIC_DATABASE, DETAIL_DATABASE)).unwrap();
        let mut task = Task::build_new(&db).unwrap();
        assert_ne!(task.id, -1);
        assert!(task.location.is_none());
        task.update_at_initial(InitialState {
            location: "qiushi".to_string(),
            staff: "potato".to_string(),
            customer: "Y.S.".to_string(),
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
        })
        .unwrap();
        task.update_final_confirm("202506121713".to_string())
            .unwrap();

        let ta = db.get_task_by_id(1).unwrap();
        assert_eq!(ta.id, 1);
        assert_eq!(ta.location.unwrap(), "qiushi");
    }
}
