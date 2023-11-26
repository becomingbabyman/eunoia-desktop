import { tauri } from '@tauri-apps/api';
import { BaseDirectory, readDir, readTextFile } from '@tauri-apps/api/fs';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { format } from 'date-fns';
import { useCallback, useEffect, useState } from "react";
import { metadata as readMeta } from "tauri-plugin-fs-extra-api";

async function show_in_folder(path) {
  await tauri.invoke('show_in_folder', {path});
}

function File({ file: { name, path, children, createdAt }, depth, insertColumn, setPreview, preview }) {
  return (
    <div 
      className={`flex flex-row items-baseline hover:bg-blue-400 hover:text-white cursor-pointer ${preview.path?.includes(path) ? 'bg-blue-400 text-white' : ''}`}
      onClick={() => !!children ? insertColumn(depth, children) : setPreview({ path })}
    >
      <small className='p-1 font-mono'>{format(createdAt, "EEEEEE MM.dd")}</small>
      <span className='p-1'>{name}</span>
      <span className="flex flex-auto justify-end pr-2">
        <span className='p-1 cursor-alias' title='show in finder' onClick={e => {e.preventDefault(); show_in_folder(path); return false}}>⎆</span>
        {children ? <span className='p-1'>&rarr;</span> : ""}
      </span>
    </div>
  )
}

function Column({ files, depth, insertColumn, setPreview, preview }) {
  return (
    <div className='flex flex-1 flex-col h-full overflow-auto border-r-2'>
      {files.sort((a, b) => b.createdAt - a.createdAt).map(file => {
        if (file.name === ".DS_Store") return null
        return <File key={file.name} file={file} depth={depth + 1} insertColumn={insertColumn} setPreview={setPreview} preview={preview}/>
      })}
    </div>
  )
}

function Player({ mediaPath }) {
  const src = convertFileSrc(mediaPath)
  return <audio key={src} src={src} controls/>
}

const pathMap = {
  "/eunoia/*local.data/AppleVoiceMemos": [".m4a", "/Library/Group Containers/group.com.apple.VoiceMemos.shared/Recordings"],
  "/eunoia/*local.data/ApplePhotosLibrary": [".mov", "/Pictures/Photos Library.photoslibrary/originals"]
}

function eunoiaToOriginalPath(eunoiaPath) {
  if (!eunoiaPath) return null
  const corePath = Object.keys(pathMap).filter(key => eunoiaPath.includes(key))[0];
  const rootPath = eunoiaPath.replace(".txt", pathMap[corePath][0]).replace(corePath, pathMap[corePath][1])
  let finalPath = rootPath
  if (finalPath.includes("Photos Library.photoslibrary")) {
    const folder = finalPath.split("/").pop().slice(0, 1)
    finalPath = finalPath.replace("photoslibrary/originals/", `photoslibrary/originals/${folder}/`)
  }
  return finalPath
}

function Preview({ preview }) {
  const mediaPath = eunoiaToOriginalPath(preview.path)
  return (
    <div className='flex flex-col flex-1 h-full'>
      {mediaPath && <>
        <div className='flex flex-row m-2 items-center'>
          <Player mediaPath={mediaPath} />
          <span className='flex p-1 ml-2 cursor-alias' title='show media in finder' onClick={e => {e.preventDefault(); show_in_folder(mediaPath); return false}}>⎆</span>
        </div>
        <div className='flex flex-col flex-1 h-full overflow-auto p-2 m-2 mt-0 bg-slate-100 rounded'>
          {preview.text}
        </div>
      </>}
    </div>
  )
}

async function mergeMeta(file) {
  const metadata = await readMeta(file.path);
  return { ...file, metadata }
}

async function addMeta(files) {
  return await Promise.all(files.map(async file => {
     // NOTE: TODO: PERFORMANCE: consider only pulling in metadata on children lazily when their column is rendered
    if (file.children) file.children = await addMeta(file.children)
    if (!file.path) return file
    const meta = await readMeta(file.path)
    return { ...file, createdAt: meta.createdAt, size: meta.size }
  }));
}

function Log() {
  const [columns, setColumns] = useState([]);
  const [preview, setPreview] = useState({});

  const insertColumn = useCallback((depth, files) => {
    setColumns([...columns.slice(0, depth), files])
    setPreview({})
  }, [columns, setColumns, setPreview])

  useEffect(() => {
    if (columns.length !== 0) return () => {}
    async function fetchData() {
      const files = await readDir("eunoia/*local.data", {dir: BaseDirectory.Home, recursive: true})
      const filesWithMeta = await addMeta(files)
      setColumns([filesWithMeta])
    }
    fetchData()
  }, [columns, setColumns])

  useEffect(() => {
    if (!preview.path || !!preview.text) return () => {}
    async function fetchData() {
      const text = await readTextFile(preview.path)
      setPreview({ ...preview, text })
    }
    fetchData()
  }, [preview, setPreview])
  
  return (
    <div className='flex flex-1 flex-row h-full'>
      {columns.map((files, i) => files && <Column key={i} files={files} depth={0} insertColumn={insertColumn} setPreview={setPreview} preview={preview} />)}
      <Preview preview={preview} />
    </div>
  )
}

function App() {
  return (
    <div className="h-screen border-t-2">
      <Log />
    </div>
  )
}

export default App;
