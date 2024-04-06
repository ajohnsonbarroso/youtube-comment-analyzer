use crate::database::{Database, DatabaseError};
use rusqlite::{params, OptionalExtension};

pub struct UserRepository<'a> {
    db: &'a Database,
}

impl<'a> UserRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        UserRepository { db }
    }

    pub fn insert_user(&mut self, username: &str, password_hash: &str, api_key: &str) -> Result<(), DatabaseError> {
        self.db.connection.execute(
            "INSERT OR REPLACE INTO users (username, password, api_key) VALUES (?1, ?2, ?3)",
            params![username, password_hash, api_key],
        )?;
        Ok(())
    }

    pub fn get_user_password_hash(&self, username: &str) -> Result<Option<String>, DatabaseError> {
        let mut stmt = self.db.connection.prepare("SELECT password FROM users WHERE username = ?1")?;
        let stored_hash = stmt.query_row(params![username], |row| row.get::<_, String>(0)).optional()?;
        Ok(stored_hash)
    }

    pub fn get_api_key(&self, username: &str) -> Result<Option<String>, DatabaseError> {
        let mut stmt = self.db.connection.prepare("SELECT api_key FROM users WHERE username = ?1")?;
        let api_key = stmt.query_row(params![username], |row| row.get(0)).optional()?;
        Ok(api_key)
    }
}