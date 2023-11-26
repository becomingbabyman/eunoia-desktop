# eunoia-desktop WORK IN PROGRESS

**This is project is very early in development. It comes with no guarantees.**

At the moment eunoia-desktop transcribes your Apple Voice Memos into text locally.

It's only tested on mac os 14.

In the future it might support more automations and platforms to help you organize the information on your devices.

The intent is to have an entirely local, free, composable, and open source suite of automations to help organize, categorize and search across all your data without ever needing to go online.

At the moment it's just a prototype.

It uses [whisper.cpp](https://github.com/ggerganov/whisper.cpp) which is built on OpenAis open source [whisper](https://openai.com/research/whisper) model for the transcription.

The [quick start](#quick-start) will help you get it running.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)


## Quick start

Make sure your terminal app has full disk access. E.g.

System Settings.app -> Privacy & Security -> Full Disk Access -> toggle on (iTerm or your preferred terminal)

Make some directories

```bash
mkdir -p ~/eunoia/\*local.data/AppleVoiceMemos
mkdir -p ~/eunoia/\*local.data/ApplePhotosLibrary
cd ~/eunoia
```

Clone this repo into `~/eunoia`

Clone [whisper.cpp](https://github.com/ggerganov/whisper.cpp#quick-start) into `~/eunoia` and follow the build instructions in their README

Install `ffmpeg` command line tool to convert media to wav for transcription

```bash
brew install ffmpeg
```

Make sure you've opened the Voice Memo's app on your mac at least once. This allows iCloud to sync with your other devices and download any memos to your local filesystem. You may need to restart your computer before the memos show up.

Build and run the Tauri app

```bash
cd ~/eunoia/eunoia-desktop
pnpm install
pnpm tauri dev
```

## Todo

transcribe fn
- [x] pull in ffmpeg to convert any AV to wav 16k
- [x] pull in whisper.cpp, maybe via whisper-rust along with the base.en model
- [x] create/expose a transcribe fn in rust that converts a given AV file to wav 16k and runs it through whisper base.en and saves the txt to the fs

voice memos
- [x] scan the local voice memos dir in MAC OS 14 -- ~/Library/Group\ Containers/group.com.apple.VoiceMemos.shared/Recordings
- [] if the dir is empty, prompt the user to open voice memos and check their iCloud sync
- [x] transcribe all the voice memos 
- [x] save the txt
- [x] make a bg process to watch the VoiceMemos folder for new/updated files and transcribe them if they're not already transcribed

list/log view
- [x] list everything in *local.data/(app name)/...
- [x] sort newest to oldest
- [x] display text preview of selected file to the right like in finder
- [x] display an audio player to the source media under the preview
- [x] link to the original txt and media files in finder

videos
- [x] scan photos (folder?)
- [x] transcribe all videos
- [x] save the txt
- [] make a bg process to watch the photos folder for new videos and transcribe them if they're not already transcribed
- [x] do not transcribe live photos

categorize/summarize fn
idk, is this best done with a full LLM like llama 2 or something more specialized.
at a high level i want to build a graph db. ideally something that plays nice with the filesystem and iCloud.
it will link all voice memos, photos, videos, notes, etc in a visual and searchable graph.
probably start with writing json to *local.data/eunoia or something along those lines

search
- [] index all the transcriptions and categories, (flexsearch?)
- [] return results in an autocomplete list view

graph
- [] render all files in a force directed graph

progress bar
- [] render a progress bar or at least a spinner when transcribing