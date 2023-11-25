import { tauri } from '@tauri-apps/api';
import { BaseDirectory, readDir, readTextFile } from '@tauri-apps/api/fs';
import { format } from 'date-fns';
import { useCallback, useEffect, useState } from "react";
import { metadata as readMeta } from "tauri-plugin-fs-extra-api";
import "./App.css";

async function show_in_folder(path) {
  await tauri.invoke('show_in_folder', {path});
}

function File({ file: { name, path, children, createdAt }, depth, insertColumn, setPreview }) {
  return (
    <div className="btn" onClick={() => !!children ? insertColumn(depth, children) : setPreview({ path })}>
      <small style={{fontFamily: "monospace"}}>{format(createdAt, "EEEEEE MM.dd")}</small>
      &nbsp;{name}
      &nbsp;<span title='show in finder' onClick={e => {e.preventDefault(); show_in_folder(path); return false}}>⎆</span>
      &nbsp;{children ? "▶" : ""}
    </div>
  )
}

function Column({ files, depth, insertColumn, setPreview }) {
  return (
    <div className='col'>
      {files.sort((a, b) => b.createdAt - a.createdAt).map(file => {
        if (file.name === ".DS_Store") return null
        return <File key={file.name} file={file} depth={depth + 1} insertColumn={insertColumn} setPreview={setPreview}/>
      })}
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
    return () => {}
  }, [columns, setColumns])

  useEffect(() => {
    if (!preview.path || !!preview.text) return () => {}
    async function fetchData() {
      const text = await readTextFile(preview.path)
      setPreview({ ...preview, text })
    }
    fetchData()
    return () => {}
  }, [preview, setPreview])
  
  return (
    <div className='row'>
      {columns.map((files, i) => files && <Column key={i} files={files} depth={0} insertColumn={insertColumn} setPreview={setPreview} />)}
      {preview.text && <div className='col'>{preview.text}</div>}
    </div>
  )
}

function App() {
  return (
    <div className="container">
      <Log />
    </div>
  )
}

export default App;
