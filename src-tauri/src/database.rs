use rusqlite::{params, Connection};
use std::error::Error;
use std::fmt;
use crate::models::CommentData;

#[derive(Debug)]
pub enum DatabaseError {
    Rusqlite(rusqlite::Error),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::Rusqlite(err) => write!(f, "Rusqlite error: {}", err),
        }
    }
}

impl Error for DatabaseError {}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        DatabaseError::Rusqlite(err)
    }
}

pub struct Database {
    pub connection: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self, DatabaseError> {
        let connection = Connection::open(db_path)?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS comments (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                sentiment TEXT NOT NULL,
                score REAL NOT NULL
            )",
            params![],
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS users (
                username TEXT PRIMARY KEY,
                password TEXT NOT NULL,
                api_key TEXT
            )",
            params![],
        )?;
        Ok(Database { connection })
    }

    pub fn initialize_database(db_path: &str) -> Result<(), String> {
        match Database::new(db_path) {
            Ok(_) => {
                println!("Database initialized successfully");
                Ok(())
            }
            Err(e) => {
                println!("Error initializing database: {}", e);
                Err(format!("Error initializing database: {}", e))
            }
        }
    }

    pub async fn store_comments_in_database(&mut self, comments: &[CommentData]) -> Result<(), String> {
        let tx = self.connection.transaction().map_err(|e| format!("Error starting transaction: {}", e))?;
        for comment in comments {
            tx.execute(
                "INSERT OR REPLACE INTO comments (id, text, sentiment, score) VALUES (?1, ?2, ?3, ?4)",
                params![comment.id, comment.text, comment.sentiment, comment.score],
            )
            .map_err(|e| format!("Error inserting comment data: {}", e))?;
        }
        tx.commit().map_err(|e| format!("Error committing transaction: {}", e))?;
        Ok(())
    }

    pub fn get_comment_data(&self) -> Result<Vec<CommentData>, DatabaseError> {
        let mut stmt = self.connection.prepare("SELECT id, text, sentiment, score FROM comments")?;
        let comments = stmt
            .query_map(params![], |row| {
                Ok(CommentData {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    sentiment: row.get(2)?,
                    score: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(comments)
    }

    pub fn get_db_path() -> String {
        let src_tauri_dir = std::env::current_dir().unwrap();
        let project_root = src_tauri_dir.parent().unwrap();
        let db_path = project_root.join("db").join("youtube_comments.db");
        let db_dir = db_path.parent().unwrap();
        std::fs::create_dir_all(db_dir).unwrap();
        db_path.to_str().unwrap().to_string()
    }
}