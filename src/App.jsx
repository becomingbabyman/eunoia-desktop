import { instantMeiliSearch } from '@meilisearch/instant-meilisearch';
import { tauri } from '@tauri-apps/api';
import { BaseDirectory, readDir, readTextFile } from '@tauri-apps/api/fs';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { format } from 'date-fns';
import 'instantsearch.css/themes/satellite.css';
import { MeiliSearch } from 'meilisearch';
import { useCallback, useEffect, useState } from "react";
import { Highlight, HitsPerPage, InfiniteHits, InstantSearch, SearchBox, Snippet } from 'react-instantsearch';
import { metadata as readMeta } from "tauri-plugin-fs-extra-api";

const searchNodeClient = new MeiliSearch({
  host: 'http://127.0.0.1:7700',
  apiKey: 'e5O4j4KNneiC9Uv7x0VQJTi4wqbX39X_Y7G7hTMe64A',
})

const searchReactClient = instantMeiliSearch(
  'http://127.0.0.1:7700',
  'e5O4j4KNneiC9Uv7x0VQJTi4wqbX39X_Y7G7hTMe64A'
);

async function show_in_folder(path) {
  await tauri.invoke('show_in_folder', {path});
}

function File({ file, depth, insertColumn, setPreview, preview }) {
  return (
    <div 
      className={`flex flex-row items-baseline hover:bg-blue-400 hover:text-white cursor-pointer ${preview.path?.includes(file.path) ? 'bg-blue-400 text-white' : ''}`}
      onClick={() => !!file.children ? insertColumn(depth, file.children) : setPreview(file)}
    >
      <small className='p-1 font-mono'>{format(file.createdAt, "EEEEEE MM.dd")}</small>
      <span className='p-1'>{file.name}</span>
      <span className="flex flex-auto justify-end pr-2">
        <span className='p-1 cursor-alias' title='show in finder' onClick={e => {e.preventDefault(); show_in_folder(file.path); return false}}>⎆</span>
        {file.children ? <span className='p-1'>&rarr;</span> : ""}
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

function Preview({ file }) {
  const mediaPath = eunoiaToOriginalPath(file.path)
  return (
    <div className='flex flex-col flex-1 h-full'>
      {mediaPath && <>
        <div className='flex flex-row m-2 items-center'>
          <Player mediaPath={mediaPath} />
          <span className='flex p-1 ml-2 cursor-alias' title='show media in finder' onClick={e => {e.preventDefault(); show_in_folder(mediaPath); return false}}>⎆</span>
        </div>
        <div className='flex flex-col flex-1 h-full overflow-auto p-2 m-2 mt-0 bg-slate-100 rounded'>
          {file.text}
        </div>
      </>}
    </div>
  )
}

function filesToIndexDocuments(files) {
  return files.reduce((acc, file) => {
    if (file.children) return [...acc, ...filesToIndexDocuments(file.children)]
    return [...acc, file]
  }, [])
}

async function addFilesToIndex(files) {
  const index = searchNodeClient.index('files')
  const documents = filesToIndexDocuments(files)
  console.log( documents)
  const response = await index.addDocuments(documents)
  console.log("added files to index", response)
  setTimeout(async () => {
    console.log('task status', await index.getTask(response.taskUid))
    console.log("test search", await searchNodeClient.index("files").search("music"))
  }, 500)
}

const HOHit = ({ setPreview }) => (
  function Hit({ hit }) {
    return (
      <article onClick={() => setPreview(hit)} className='cursor-pointer'>
        <h1>
          <Highlight attribute="name" hit={hit} />
        </h1>
        <Snippet hit={hit} attribute="text" />
      </article>
    );
  }
)

function Search({ setPreview }) {
  const [indexCreated, setIndexCreated] = useState(false)

  useEffect(() => {
    async function fetchData() {
      console.log("creating index")
      // await searchNodeClient.index("files").delete()
      const r = await searchNodeClient.createIndex("files", { primaryKey: "id" })
      console.log("index created", r)
      setIndexCreated(true)
    }
    fetchData()
  }, [setIndexCreated])

  if (!indexCreated) return "loading search.."
  return (
    <div className='flex flex-col flex-1 h-full overflow-auto border-r-2'>
      <InstantSearch
        indexName="files"
        searchClient={searchReactClient}
        // initialUiState={{ files: { hitsPerPage: 10 } }}
      >
        <SearchBox />
        <InfiniteHits showPrevious={false} hitComponent={HOHit({ setPreview })} />
        <HitsPerPage
          items={[
            { label: '5 hits per page', value: 5, default: true },
            { label: '10 hits per page', value: 10 },
          ]}
        />
      </InstantSearch>
    </div>
  )
}

async function mergeMeta(file) {
  const metadata = await readMeta(file.path);
  return { ...file, metadata }
}

async function addMeta(files) {
  return await Promise.all(files.map(async file => {
    file.id = file.path.replace(/[^a-zA-Z0-9-_]/g, '_') // convert path to a meilisearch-friendly id
    // NOTE: TODO: PERFORMANCE: consider only pulling in metadata on children lazily when their column is rendered
    if (file.children) file.children = await addMeta(file.children)
    // if (!file.path) return file
    const meta = await readMeta(file.path)
    if (!file.children) file.text = await readTextFile(file.path)
    // console.log({ ...file, ...meta })
    return { ...file, ...meta }
  }));
}

function filterFiles(files) {
  return files.reduce((acc, file) => {
    if (file.name === ".DS_Store") return acc
    // if (!file.name.includes(".txt")) return acc
    if (file.children) file.children = filterFiles(file.children)
    acc.push(file)
    return acc
  }, [])
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
      const filteredFiles = filterFiles(files)
      const filesWithMeta = await addMeta(filteredFiles)
      addFilesToIndex(filesWithMeta)
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
      <Search setPreview={setPreview} />
      {columns.map((files, i) => files && <Column key={i} files={files} depth={0} insertColumn={insertColumn} setPreview={setPreview} preview={preview} />)}
      <Preview file={preview} />
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
