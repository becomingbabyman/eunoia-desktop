import { BaseDirectory, readDir, readTextFile } from '@tauri-apps/api/fs';
import { useCallback, useEffect, useState } from "react";
import "./App.css";

function File({ file: { name, path, children }, depth, insertColumn, setPreview }) {
  return (
    <div className="btn" onClick={() => !!children ? insertColumn(depth, children) : setPreview({ path })}>
      {name} {children ? ">" : ""}
    </div>
  )
}

function Column({ files, depth, insertColumn, setPreview }) {
  return (
    <div className='col'>
      {files.map(file => {
        if (file.name === ".DS_Store") return null
        return <File key={file.name} file={file} depth={depth + 1} insertColumn={insertColumn} setPreview={setPreview}/>
      })}
    </div>
  )
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
      setColumns([files])
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
