// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::env;
use std::str;
use tauri::api::path::home_dir;
use walkdir::{WalkDir, DirEntry};
use std::fs::{self, Metadata};
use std::path::Path;
use std::collections::{HashSet, HashMap};
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayMenuItem};
mod watch;
use watch::watch;
use notify_debouncer_full::DebouncedEvent;
use notify::event::{Event, EventKind, CreateKind, ModifyKind, MetadataKind};
use tokio;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn get_metadata(file_path: String) -> Option<Metadata> {
    let path = Path::new(file_path.as_str());
    // Attempt to retrieve the metadata of the file
    if let Ok(metadata) = fs::metadata(path) {
        return Some(metadata);
    }
    None
}

fn list_files_in_directory(max_depth: usize, directory_path: String) -> Vec<DirEntry> {
    let mut file_paths = Vec::new();

    for entry in WalkDir::new(directory_path).max_depth(max_depth).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            file_paths.push(entry);
        }
    }

    file_paths
}

async fn transcribe(media_file_path: String, output_path: String) {
    println!("Transcribing {} to {}", media_file_path, output_path);
    let home_dir = home_dir().unwrap().to_str().unwrap().to_string();
    
    let ffmpeg_child = Command::new("ffmpeg")
        .arg("-i")
        .arg(media_file_path)
        .arg("-f")
        .arg("wav")
        .arg("-ar")
        .arg("16000")
        .arg("-")
        .stdout(Stdio::piped())       
        .spawn()                   
        .unwrap();
    let whisper_command = home_dir.clone() + "/eunoia/whisper.cpp/main";
    let _whisper_cpp_child = Command::new(whisper_command)
        .arg("-m")
        .arg(home_dir.clone() + "/eunoia/whisper.cpp/models/ggml-base.en.bin")
        .arg("-p")
        .arg("1")
        .arg("-otxt")
        .arg("-pp")
        .arg("-of")
        .arg(output_path)
        .arg("-")
        .stdin(Stdio::from(ffmpeg_child.stdout.unwrap()))
        // .stdout(Stdio::null())
        // .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    // let output = whisper_cpp_child.wait_with_output().unwrap();
    // let result = str::from_utf8(&output.stdout).unwrap();
    // // println!("{}", result);
    // result.to_string()
}

async fn transcribe_folder(media_in_path: String, text_out_path: String, media_ext: String, max_depth: usize) {
    let media_in_files: HashMap<String, String> = list_files_in_directory(max_depth, media_in_path.clone()).iter().map(|dir_entry| (dir_entry.file_name().to_str().unwrap().to_string(), dir_entry.path().to_str().unwrap().to_string())).collect();
    let transcribed_files_map: HashSet<String> = list_files_in_directory(max_depth, text_out_path.clone()).iter().map(|dir_entry| dir_entry.file_name().to_str().unwrap().to_string()).collect();
    for (file, file_path) in media_in_files {
        let mut file_vec: Vec<&str> = file.split('.').collect();
        let file_type = file_vec.pop();
        if Some(media_ext.as_str()) != file_type {
            continue;
        }
        let file_name = file_vec.join("");  
        let output_file_name_and_ext = file_name.clone() + ".txt";
        if transcribed_files_map.contains(&output_file_name_and_ext) {
            let file_metadata = get_metadata(file_path.clone());
            let current_transcription_metadata = get_metadata(text_out_path.clone() + &output_file_name_and_ext);
            // println!("Comparing {:?} and {:?}", file_metadata.clone().unwrap().modified().unwrap(), current_transcription_metadata.clone().unwrap().modified().unwrap());
            if file_metadata.unwrap().modified().unwrap() <= current_transcription_metadata.unwrap().modified().unwrap() {
                println!("Skipping {}", output_file_name_and_ext);
                continue;
            }
        }
        let output_path = text_out_path.clone() + &file_name;
        transcribe(file_path.clone(), output_path.clone()).await;
    }
}

#[tauri::command]
async fn transcribe_apple_voice_memos() {
    let home_dir = home_dir().unwrap().to_str().unwrap().to_string();
    let media_in_path = home_dir.clone() + "/Library/Group Containers/group.com.apple.VoiceMemos.shared/Recordings";
    let text_out_path = home_dir.clone() + "/eunoia/*local.data/AppleVoiceMemos/";
    transcribe_folder(media_in_path, text_out_path, "m4a".to_string(), 1).await;
}

#[tauri::command]
async fn transcribe_apple_photos_library() {
    let home_dir = home_dir().unwrap().to_str().unwrap().to_string();
    let media_in_path = home_dir.clone() + "/Pictures/Photos Library.photoslibrary/originals";
    let text_out_path = home_dir.clone() + "/eunoia/*local.data/ApplePhotosLibrary/";
    transcribe_folder(media_in_path, text_out_path, "mov".to_string(), 2).await;
}

fn get_ext (file_name: &str) -> String {
    let mut file_vec: Vec<&str> = file_name.split('.').collect();
    let file_type = file_vec.pop();
    if file_type.is_none() {
        return "".to_string();
    }
    return file_type.unwrap().to_string();
}

fn on_apple_voice_memos_watch_event (event: &DebouncedEvent) {
    match &event.event {
        Event { kind: EventKind::Create(CreateKind::File), paths, .. } => {
            if get_ext(paths.first().unwrap().to_str().unwrap()) == "m4a" {
                println!("Create File {:?}", paths);
                tokio::spawn(transcribe_apple_voice_memos());
            }
        },
        Event { kind: EventKind::Modify(ModifyKind::Metadata(MetadataKind::Extended)), paths, .. } => {
            if get_ext(paths.first().unwrap().to_str().unwrap()) == "m4a" {
                println!("Modify File {:?}", paths);
                tokio::spawn(transcribe_apple_voice_memos());
            }
        },
        // Event { kind: EventKind::Remove(_), paths, .. } => {
        //     println!("Remove File {:?}", paths);
        // },
        _ => {
            println!("Other");
        }
    }
}

#[tokio::main]
async fn main() {
    // Create system tray
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let tray_menu = SystemTrayMenu::new()
    .add_item(quit)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(hide);
    let system_tray = SystemTray::new().with_menu(tray_menu);

    // Watch for file changes
    let home_dir = home_dir().unwrap().to_str().unwrap().to_string();
    let voice_memos_path = home_dir.clone() + "/Library/Group Containers/group.com.apple.VoiceMemos.shared/Recordings";
    tokio::spawn(watch(voice_memos_path, on_apple_voice_memos_watch_event));

    // Run all transcribers at startup to catch up on any missed files
    tokio::spawn(transcribe_apple_voice_memos());
    tokio::spawn(transcribe_apple_photos_library());
 
    // Run Tauri application
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet, transcribe_apple_voice_memos])
    .system_tray(system_tray)
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
