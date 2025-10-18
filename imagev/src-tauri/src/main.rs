// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use exif;

struct AppState {
    initial_file: Mutex<Option<String>>,
    db_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ImageInfo {
    path: String,
    shot_at: i64, // Unix timestamp
}

#[tauri::command]
fn get_initial_file(state: State<AppState>) -> Option<String> {
    state.initial_file.lock().unwrap().take()
}

fn get_db_connection(app_handle: &AppHandle) -> Result<Connection> {
    let state = app_handle.state::<AppState>();
    Connection::open(&state.db_path)
}

fn sanitize_table_name(path: &str) -> String {
    path.replace(|c: char| !c.is_alphanumeric(), "_")
}

fn init_db(app_handle: &AppHandle) -> Result<()> {
    let _conn = get_db_connection(app_handle)?;
    // The table creation will be handled dynamically for each directory
    Ok(())
}

fn get_shot_at(path: &Path) -> Option<i64> {
    let file = std::fs::File::open(path).ok()?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).ok()?;
    if let Some(field) = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        if let exif::Value::Ascii(ref ascii) = field.value {
            if let Some(datetime_str) = ascii.get(0) {
                if let Ok(datetime_str) = std::str::from_utf8(datetime_str) {
                    if let Ok(datetime) = NaiveDateTime::parse_from_str(datetime_str.trim_end_matches('\0'), "%Y:%m:%d %H:%M:%S") {
                        return Some(datetime.and_utc().timestamp());
                    }
                }
            }
        }
    }

    let metadata = fs::metadata(path).ok()?;
    if let Ok(modified_time) = metadata.modified() {
        let datetime: DateTime<Utc> = modified_time.into();
        return Some(datetime.timestamp());
    }

    None
}

#[tauri::command]
fn get_sorted_image_list(initial_path: String, app_handle: AppHandle) -> Result<Vec<ImageInfo>, String> {
    let path = Path::new(&initial_path);
    let dir = match path.is_dir() {
        true => path,
        false => path.parent().ok_or("Could not get parent directory")?,
    };

    let mut conn = get_db_connection(&app_handle).map_err(|e| e.to_string())?;
    let table_name = sanitize_table_name(dir.to_str().unwrap());

    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {} (\n                filename TEXT PRIMARY KEY,\n                shot_at INTEGER NOT NULL\n            )",
            table_name
        ),
        [],
    )
    .map_err(|e| e.to_string())?;

    let fs_images = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                let extension = path.extension()?.to_str()?.to_lowercase();
                if ["jpg", "jpeg", "png", "heif", "heic"].contains(&extension.as_str()) {
                    return Some(path.file_name()?.to_str()?.to_string());
                }
            }
            None
        })
        .collect::<std::collections::HashSet<String>>();

    let db_images: std::collections::HashSet<String> = {
        let mut stmt = conn
            .prepare(&format!("SELECT filename FROM {}", table_name))
            .map_err(|e| e.to_string())?;
        let x = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<std::collections::HashSet<String>, _>>()
            .map_err(|e| e.to_string())?;
        x
    };

    let new_images = fs_images.difference(&db_images).cloned().collect::<Vec<String>>();
    let deleted_images = db_images.difference(&fs_images).cloned().collect::<Vec<String>>();

    if !deleted_images.is_empty() {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for image in &deleted_images {
            tx.execute(
                &format!("DELETE FROM {} WHERE filename = ?", table_name),
                params![image],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
    }

    for image_name in new_images {
        let image_path = dir.join(&image_name);
        if let Some(shot_at) = get_shot_at(&image_path) {
            conn.execute(
                &format!(
                    "INSERT OR REPLACE INTO {} (filename, shot_at) VALUES (?, ?)",
                    table_name
                ),
                params![image_name, shot_at],
            )
            .map_err(|e| e.to_string())?;
        } else {
            // If EXIF data is not available, use file modification time
            let metadata = fs::metadata(&image_path).map_err(|e| e.to_string())?;
            if let Ok(modified_time) = metadata.modified() {
                let datetime: DateTime<Utc> = modified_time.into();
                conn.execute(
                    &format!(
                        "INSERT OR REPLACE INTO {} (filename, shot_at) VALUES (?, ?)",
                        table_name
                    ),
                    params![image_name, datetime.timestamp()],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    let images = {
        let mut stmt = conn
            .prepare(&format!(
                "SELECT filename, shot_at FROM {} ORDER BY shot_at ASC",
                table_name
            ))
            .map_err(|e| e.to_string())?;

        let x = stmt
            .query_map([], |row| {
                Ok(ImageInfo {
                    path: dir
                        .join(row.get::<_, String>(0)?)
                        .to_str()
                        .unwrap()
                        .to_string(),
                    shot_at: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<ImageInfo>, _>>()
            .map_err(|e| e.to_string())?;
        x
    };

    Ok(images)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let mut db_path = app.path().app_data_dir().expect("Failed to get app data dir");
            if !db_path.exists() {
                fs::create_dir_all(&db_path).expect("Failed to create app data dir");
            }
            db_path.push("image_cache.db");

            let app_state = AppState {
                initial_file: Mutex::new(None),
                db_path,
            };

            if let Some(arg) = std::env::args().nth(1) {
                *app_state.initial_file.lock().unwrap() = Some(arg);
            }

            app.manage(app_state);

            init_db(&app.handle()).expect("Failed to initialize database");

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            get_initial_file,
            get_sorted_image_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}