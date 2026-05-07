pub mod collections;
pub mod examples;
pub mod requests;
pub mod schema;

use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug)]
pub struct Database {
    conn: Mutex<Connection>,
    path: Mutex<PathBuf>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path.as_ref())?;
        let db = Database {
            conn: Mutex::new(conn),
            path: Mutex::new(path.as_ref().to_path_buf()),
        };
        db.init()?;
        Ok(db)
    }

    pub fn connection(&self) -> &Mutex<Connection> {
        &self.conn
    }

    pub fn path(&self) -> std::sync::MutexGuard<'_, PathBuf> {
        self.path.lock().unwrap()
    }

    fn init(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        schema::run(&conn)?;
        Ok(())
    }
}
