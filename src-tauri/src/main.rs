// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tokio;
use tokio::process::Command;
use tokio::io;
use std::result::Result::Ok;
use std::process::Stdio;
use std::env;
use std::str;
use std::time::Instant;
use tauri::api::path::home_dir;
use walkdir::{WalkDir, DirEntry};
use std::fs::{self, Metadata};
use std::path::Path;
use std::collections::{HashSet, HashMap};
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayMenuItem};
mod watch;
use watch::watch;
mod show_in_folder;
use show_in_folder::show_in_folder;
use notify_debouncer_full::DebouncedEvent;
use notify::event::{Event, EventKind, CreateKind, ModifyKind, MetadataKind};

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

async fn transcribe(media_file_path: String, output_path: String) -> io::Result<()> {
    let now_instant = Instant::now();
    println!(">Start transcribe {}", media_file_path.clone());
    let home_dir = home_dir().unwrap().to_str().unwrap().to_string();
    
    let mut ffmpeg_child = Command::new("ffmpeg")
        .arg("-i")
        .arg(media_file_path.clone())
        .arg("-f")
        .arg("wav")
        .arg("-ar")
        .arg("16000")
        .arg("-loglevel")
        .arg("fatal")
        .arg("-")
        .stdout(Stdio::piped())       
        .spawn()
        .expect("Failed to start 'ffmpeg' command");                   
    let whisper_command = home_dir.clone() + "/eunoia/whisper.cpp/main";
    let mut whisper_cpp_child = Command::new(whisper_command)
        .arg("-m")
        .arg(home_dir.clone() + "/eunoia/whisper.cpp/models/ggml-base.en.bin")
        .arg("-p")
        .arg("1")
        .arg("-otxt")
        // .arg("-pp")
        .arg("-of")
        .arg(output_path.clone())
        .arg("-")
        .stdin(Stdio::piped())
        // .stdout(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start 'whisper.cpp' command");   

    if let Some(mut ffmpeg_stdout) = ffmpeg_child.stdout.take() {
        if let Some(mut whisper_stdin) = whisper_cpp_child.stdin.take() {
            tokio::io::copy(&mut ffmpeg_stdout, &mut whisper_stdin).await?;
        }
    }

    // Wait for 'ffmpeg_child' to finish
    ffmpeg_child.wait().await?;

    // Read the output of 'whisper_cpp_child'
    let _whisper_output = whisper_cpp_child
        .wait_with_output()
        .await
        .expect("Failed to read 'grep' output");

    // Convert the output to a String and print it
    // println!("Whisper output: {}", String::from_utf8_lossy(&_whisper_output.stdout));
    println!("<End   transcribe {} {:?}", media_file_path.clone(), now_instant.elapsed());

    Ok(())
}

async fn transcribe_folder(media_in_path: String, text_out_path: String, media_ext: String, max_depth: usize, min_size: u64) -> io::Result<()> {
    println!(">Start transcribe_folder {} {}", media_in_path, media_ext);
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
        let file_metadata = get_metadata(file_path.clone());
        if transcribed_files_map.contains(&output_file_name_and_ext) {
            let current_transcription_metadata = get_metadata(text_out_path.clone() + &output_file_name_and_ext);
            // println!("Comparing {:?} and {:?}", file_metadata.clone().unwrap().modified().unwrap(), current_transcription_metadata.clone().unwrap().modified().unwrap());
            if file_metadata.as_ref().unwrap().modified().unwrap() <= current_transcription_metadata.unwrap().modified().unwrap() {
                // println!(">   Skipping {}   <", output_file_name_and_ext);
                continue;
            }
        }
        if file_metadata.as_ref().unwrap().len() < min_size {
            continue;
        }
        let output_path = text_out_path.clone() + &file_name;
        let _ = transcribe(file_path.clone(), output_path.clone()).await;
    }
    println!("<End transcribe_folder {} {}", media_in_path, media_ext);
    Ok(())
}

// #[tauri::command]
async fn transcribe_apple_voice_memos() -> io::Result<()> {
    let home_dir = home_dir().unwrap().to_str().unwrap().to_string();
    let media_in_path = home_dir.clone() + "/Library/Group Containers/group.com.apple.VoiceMemos.shared/Recordings";
    let text_out_path = home_dir.clone() + "/eunoia/*local.data/AppleVoiceMemos/";
    let _ = transcribe_folder(media_in_path, text_out_path, "m4a".to_string(), 1, 0).await;
    Ok(())
}

// #[tauri::command]
async fn transcribe_apple_photos_library() -> io::Result<()> {
    let home_dir = home_dir().unwrap().to_str().unwrap().to_string();
    let media_in_path = home_dir.clone() + "/Pictures/Photos Library.photoslibrary/originals";
    let text_out_path = home_dir.clone() + "/eunoia/*local.data/ApplePhotosLibrary/";
    let _ = transcribe_folder(media_in_path, text_out_path, "mov".to_string(), 2, 9999999).await;
    Ok(())
}

fn get_ext (file_name: &str) -> String {
    let mut file_vec: Vec<&str> = file_name.split('.').collect();
    let file_type = file_vec.pop();
    if file_type.is_none() {
        return "".to_string();
    }
    return file_type.unwrap().to_string();
}

fn on_watch_event<F>(event: &DebouncedEvent, ext: &str, on_event: F)
where
    F: Fn() + Send + 'static, // Closure is Send and 'static because it's used with tokio::spawn
{
    match &event.event {
        Event { kind: EventKind::Create(CreateKind::File), paths, .. } => {
            if get_ext(paths.first().unwrap().to_str().unwrap()) == ext {
                println!("Create File {:?}", paths);
                on_event();
            }
        },
        Event { kind: EventKind::Modify(ModifyKind::Metadata(MetadataKind::Extended)), paths, .. } => {
            if get_ext(paths.first().unwrap().to_str().unwrap()) == ext {
                println!("Modify File {:?}", paths);
                on_event();
            }
        },
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
    tokio::spawn(watch(voice_memos_path, |event| {
        on_watch_event(event, "m4a", || {
            tokio::spawn(transcribe_apple_voice_memos());
    })}));
    let photos_path = home_dir.clone() + "/Pictures/Photos Library.photoslibrary/originals";
    tokio::spawn(watch(photos_path, |event| {
        on_watch_event(event, "mov", || {
            tokio::spawn(transcribe_apple_photos_library());
    })}));

    // Run all transcribers at startup to catch up on any missed files
    tokio::spawn(transcribe_apple_voice_memos());
    tokio::spawn(transcribe_apple_photos_library());
 
    // Run Tauri application
    tauri::Builder::default()
    .plugin(tauri_plugin_fs_extra::init())
    .invoke_handler(tauri::generate_handler![show_in_folder])
    .system_tray(system_tray)
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
