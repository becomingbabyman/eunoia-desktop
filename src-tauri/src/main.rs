// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::env;
use std::str;
use walkdir::WalkDir;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashSet;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn get_updated_at_time(file_path: String) -> Option<SystemTime> {
    let path = Path::new(file_path.as_str());

    // Attempt to retrieve the metadata of the file
    if let Ok(metadata) = fs::metadata(path) {
        if let Ok(modified_time) = metadata.modified() {
            return Some(modified_time);
        }
    }

    None
}

fn time_to_string(time: Option<SystemTime>) -> String {
    if time.is_none() {
        return "".to_string();
    }
    let duration = time.unwrap().duration_since(UNIX_EPOCH).unwrap();
    let duration_string = duration.as_secs().to_string() + "." + &duration.subsec_nanos().to_string();
    let ret = duration_string.replace(".", "-");
    if ret.is_empty() {
        return ret + "0";
    } else {
        return ret;
    }
}

fn list_files_in_directory(directory_path: String) -> Vec<String> {
    let mut file_paths = Vec::new();

    for entry in WalkDir::new(directory_path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            // let s = get_updated_at_time(entry.path().to_str().unwrap());
            // println!("{:?}", time_to_string(s.unwrap()));
            file_paths.push(entry.file_name().to_str().unwrap().to_string());
        }
    }

    file_paths
}

async fn transcribe(media_file_path: String, output_path: String) -> String {
    let home_dir = tauri::api::path::home_dir().unwrap().to_str().unwrap().to_string();
    
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
    let whisper_cpp_child = Command::new(whisper_command)
        .arg("-m")
        .arg(home_dir.clone() + "/eunoia/whisper.cpp/models/ggml-base.en.bin")
        .arg("-p")
        .arg("2")
        .arg("-otxt")
        .arg("-of")
        .arg(output_path)
        .arg("-")
        .stdin(Stdio::from(ffmpeg_child.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let output = whisper_cpp_child.wait_with_output().unwrap();
    let result = str::from_utf8(&output.stdout).unwrap();
    // println!("{}", result);
    result.to_string()
}

#[tauri::command]
async fn transcribe_apple_voice_memos() {
    let home_dir = tauri::api::path::home_dir().unwrap().to_str().unwrap().to_string();
    let voice_memos_path = home_dir.clone() + "/Library/Group Containers/group.com.apple.VoiceMemos.shared/Recordings";
    let output_folder_path = home_dir.clone() + "/eunoia/*local.datoms/AppleVoiceMemos/";
    let voice_memo_files = list_files_in_directory(voice_memos_path.clone());
    let transcribed_files_set: HashSet<String> = list_files_in_directory(output_folder_path.clone()).iter().cloned().collect();
    for file in voice_memo_files {
        let mut file_vec: Vec<&str> = file.split('.').collect();
        let file_type = file_vec.pop();
        if Some("m4a") != file_type {
            continue;
        }
        let file_name = file_vec.join("");  
        let file_path = voice_memos_path.clone() + "/" + &file;
        let output_file_name = file_name + "|updated-" + &time_to_string(get_updated_at_time(file_path.clone()));
        let output_file_name_and_ext = output_file_name.clone() + ".txt";
        if transcribed_files_set.contains(&output_file_name_and_ext) {
            println!("Skipping {}", output_file_name_and_ext);
            continue;
        }
        let output_path = output_folder_path.clone() + &output_file_name;
        transcribe(file_path.clone(), output_path.clone()).await;
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, transcribe_apple_voice_memos])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}