// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::env;
use std::str;


// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn transcribe() {
    let home_dir = tauri::api::path::home_dir().unwrap().to_str().unwrap().to_string();
    
    let ffmpeg_child = Command::new("ffmpeg")
        .arg("-i")
        .arg(home_dir.clone() + "/eunoia/samples/New Recording 19.m4a")
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
        .arg("-otxt")
        .arg("-of")
        .arg(home_dir.clone() + "/eunoia/.output/Voice Memos/New Recording 19")
        .arg("-")
        .stdin(Stdio::from(ffmpeg_child.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let output = whisper_cpp_child.wait_with_output().unwrap();
    let result = str::from_utf8(&output.stdout).unwrap();
    println!("{}", result);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, transcribe])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}