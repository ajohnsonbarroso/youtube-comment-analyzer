#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod database;
mod youtube;
mod sentiment;
mod authenticator;
mod user_repository;

use database::Database;
use models::CommentData;
use youtube::YoutubeFetcher;
use sentiment::SentimentAnalyzer;
use user_repository::UserRepository;
use authenticator::Authenticator;

use std::env;
use tokio;

#[tauri::command]
fn analyze_youtube_comments(
    url: &str,
    api_key: &str,
    analyzer: tauri::State<SentimentAnalyzer>,
) -> Result<Vec<CommentData>, String> {
    
    let run_time = tokio::runtime::Runtime::new().unwrap();
    run_time.block_on(async {
        let client = reqwest::Client::new();
        let fetcher = YoutubeFetcher::new(&client, api_key);
        let comments = fetcher.retrieve_comments_stream(url).await.map_err(|e| e.to_string())?;

        let analyzed_comments = analyzer.analyze_comments(&comments).map_err(|e| e.to_string())?;

        let mut db = Database::new(&Database::get_db_path()).map_err(|e| {
            let error_message = format!("Error creating database connection: {}", e);
            println!("{}", error_message);
            error_message
        })?;

        db.store_comments_in_database(&analyzed_comments).await?;

        println!("Comments analyzed and stored successfully");
        Ok(analyzed_comments)
    })
}

#[tauri::command]
fn get_stored_comments() -> Result<Vec<CommentData>, String> {
    let db = Database::new(&Database::get_db_path()).map_err(|e| {
        let error_message = format!("Error creating database connection: {}", e);
        println!("{}", error_message);
        error_message
    })?;
    let comments = db.get_comment_data().map_err(|e| {
        let error_message = format!("Error retrieving comment data: {}", e);
        println!("{}", error_message);
        error_message
    })?;

    println!("Stored comments retrieved successfully");
    Ok(comments)
}

#[tauri::command]
fn get_api_key(username: &str) -> Result<String, String> {
    let db = Database::new(&Database::get_db_path()).map_err(|e| {
        let error_message = format!("Error creating database connection: {}", e);
        println!("{}", error_message);
        error_message
    })?;

    let user_repo = UserRepository::new(&db);
    let api_key = user_repo.get_api_key(username).map_err(|e| {
        let error_message = format!("Error retrieving API key: {}", e);
        println!("{}", error_message);
        error_message
    })?;

    match api_key {
        Some(key) => {
            println!("API key retrieved successfully: {}", key);
            Ok(key)
        }
        None => {
            let message = format!("API key not found for user: {}", username);
            println!("{}", message);
            Err(message)
        }
    }
}

#[tauri::command]
fn submit_profile(username: &str, password: &str, api_key: &str) -> Result<(), String> {
    let conn = Database::new(&Database::get_db_path()).map_err(|e| e.to_string())?;
    let mut user_repo = UserRepository::new(&conn);

    let password_hash = Authenticator::hash_password(password).map_err(|_| "Failed to hash password".to_string())?;
    user_repo.insert_user(username, &password_hash, api_key)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn login(username: &str, password: &str) -> Result<bool, String> {
    let conn = Database::new(&Database::get_db_path()).map_err(|e| e.to_string())?;
    let user_repo = UserRepository::new(&conn);

    let stored_hash = user_repo.get_user_password_hash(username)
        .map_err(|e| e.to_string())?;

    match stored_hash {
        Some(hash) => {
            let is_valid = Authenticator::verify_password(password, &hash)
                .map_err(|_| "Failed to verify password".to_string())?;
            Ok(is_valid)
        }
        None => Ok(false),
    }
}

fn main() {
    env::set_var("PYTHONPATH", r"C:\Users\alexj\Documents\RustProjects\py03_test\venv\Lib\site-packages");
    
    if let Err(e) = Database::initialize_database(&Database::get_db_path()) {
        println!("{}", e);
        return;
    }

    if let Err(e) = SentimentAnalyzer::initialize_analyzer() {
        println!("{}", e);
        return;
    }

    let analyzer = SentimentAnalyzer::get_analyzer();

    tauri::Builder::default()
        .manage(analyzer)
        .invoke_handler(tauri::generate_handler![
            analyze_youtube_comments,
            get_stored_comments,
            get_api_key,
            submit_profile,
            login
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}