import { tauri } from '@tauri-apps/api';
import { BaseDirectory, readDir, readTextFile } from '@tauri-apps/api/fs';
import { format } from 'date-fns';
import { useCallback, useEffect, useState } from "react";
import { metadata as readMeta } from "tauri-plugin-fs-extra-api";

async function show_in_folder(path) {
  await tauri.invoke('show_in_folder', {path});
}

function File({ file: { name, path, children, createdAt }, depth, insertColumn, setPreview, preview }) {
  return (
    <div 
      className={`hover:bg-blue-400 hover:text-white cursor-pointer ${preview.path?.includes(path) ? 'bg-blue-400 text-white' : ''}`}
      onClick={() => !!children ? insertColumn(depth, children) : setPreview({ path })}
    >
      <small className='p-1 font-mono'>{format(createdAt, "EEEEEE MM.dd")}</small>
      <span className='p-1'>{name}</span>
      <span className='p-1 cursor-alias' title='show in finder' onClick={e => {e.preventDefault(); show_in_folder(path); return false}}>⎆</span>
      {children ? "▶" : ""}
    </div>
  )
}

function Column({ files, depth, insertColumn, setPreview, preview }) {
  return (
    <div className='flex flex-1 flex-col h-full overflow-auto'>
      {files.sort((a, b) => b.createdAt - a.createdAt).map(file => {
        if (file.name === ".DS_Store") return null
        return <File key={file.name} file={file} depth={depth + 1} insertColumn={insertColumn} setPreview={setPreview} preview={preview}/>
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
    <div className='flex flex-1 flex-row h-full'>
      {columns.map((files, i) => files && <Column key={i} files={files} depth={0} insertColumn={insertColumn} setPreview={setPreview} preview={preview} />)}
      <div className='flex flex-col flex-1 h-full overflow-auto'>{preview.text}</div>
    </div>
  )
}

function App() {
  return (
    <div className="h-screen">
      <Log />
    </div>
  )
}

export default App;
