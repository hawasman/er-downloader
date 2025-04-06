import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { message, open } from "@tauri-apps/plugin-dialog";
import { exit } from '@tauri-apps/plugin-process';
import { useEffect, useState } from "react";
import ReactMarkdown from "react-markdown";
import "./App.css";

interface ProgressEventPayload {
  total_size: string;
  current_size: string;
  progress: string;
  speed: string;
}

function App() {
  const [pathNotes, setPathNotes] = useState<string>("")
  const [installationDirectory, setInstallationDirectory] = useState('');
  const [progressMessage, setProgressMessage] = useState("")
  const [progress, setProgress] = useState<ProgressEventPayload>()
  const [downloading, setDownloading] = useState(false)
  const [currentTab, setCurrentTab] = useState<"instructions" | "patch-notes">("instructions")

  useEffect(() => {
    const unListen = listen<ProgressEventPayload>('download_progress', (event) => {
      setProgressMessage(`Download progress: ${event.payload.current_size}/${event.payload.total_size} `);
      setProgress(event.payload)
    });
    return () => {
      unListen.then((f) => f());
    };
  }
    , []);

  useEffect(() => {
    const unListen = listen<string>('get_patch_notes', (event) => {
      setPathNotes(event.payload)
    });
    return () => {
      unListen.then((f) => f());
    };
  }
    , []);


  async function openDialog() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select installation directory",
    });
    if (selected) {
      console.log("Selected directory:", selected);
      setInstallationDirectory(selected as string);
    } else {
      setInstallationDirectory('');
      console.log("No directory selected");
    }
  }

  async function download() {
    setDownloading(true)
    setProgressMessage("Downloading...")
    await invoke("download_er")
    setProgressMessage("Download complete")
    extractFile()

  }

  async function extractFile() {
    setDownloading(true)
    setProgressMessage("Extracting...")
    await invoke("extract_file", { extractPath: installationDirectory })
    setProgressMessage("Extraction complete")
    await message('Installation Successful', { title: 'Convergence ER Downloader', kind: 'info' });
    setDownloading(false)
  }

  async function getPatchNotes() {
    if (pathNotes !== "") return
    setPathNotes("Fetching patch notes...")
    await invoke("get_patch_notes")
  }

  return (
    <main className="flex flex-col bg-zinc-800 text-gray-200 h-screen justify-between p-3">
      <div className="flex justify-between items-center w-full">
        <h2 className="text-2xl">Convergence ER Downloader</h2>
        <div className="flex gap-2">
          {/* <button
            className="text-gray-500 hover:text-gray-700"
            onClick={async () => {
              const window = getCurrentWindow();
              await window.minimize();

            }}
          >
            <span className="text-4xl">-</span>
          </button> */}
          <button
            className="text-gray-500 hover:text-gray-700"
            onClick={async () => {
              await exit(0);
            }}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-6 w-6"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>
      </div>
      {/* Tabs */}
      <div className="mb-4"></div>
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex" aria-label="Tabs">
          <button onClick={() => setCurrentTab("instructions")} className={"border-b-3 py-2 px-4 text-sm font-medium" + (currentTab === "instructions" ? "  border-indigo-200 text-indigo-400" : " border-transparent text-gray-200 hover:border-gray-300")}>
            Instructions
          </button>
          <button onClick={() => {
            getPatchNotes();
            setCurrentTab("patch-notes")
          }} className={"border-b-3 py-2 px-4 text-sm font-medium" + (currentTab === "patch-notes" ? "  border-indigo-200 text-indigo-400" : " border-transparent text-gray-200 hover:border-gray-300")}>
            Patch Notes
          </button>
        </nav>
      </div>

      {/* Content */}
      <div hidden={currentTab !== "instructions"} className="mb-auto">
        <div className="prose">
          <h2 className="mb-4"></h2>
          <ol className="text-gray-200 list-decimal px-4">
            <li>Please do not install the mod into the game directory.</li>
            <li>Press the 'Browse' button and navigate to the folder where you want to install the mod files.</li>
            <li>Make sure the old version does not exist in the selected folder</li>
            <li>Press the 'Download and Extract' button. The download process will start.</li>
            <li>When the download is complete, the files will be unpacked. Please wait until it is finished.</li>
            <li>You will see the 'Installation Successful' message.</li>
            <li>THE UNPACKING STEP MIGHT HANG UP. PLEASE BE PATIENT, ESPECIALLY IF YOU ARE ON HDD.</li>
          </ol>
        </div>
      </div>

      {/* TODO: Replace with a markdown version of the path notes from a server */}
      <div hidden={currentTab !== "patch-notes"} className="mb-auto overflow-auto">
        <div className="prose prose-invert">
          <ReactMarkdown>{pathNotes}</ReactMarkdown>
        </div>
      </div>

      {/* Download Progress */}
      {
        downloading && (
          <div className="mb-6">
            <h2 className="text-xl font-semibold mb-4">Download Progress</h2>
          </div>
        )
      }
      {
        downloading && (
          <div className="mb-6">
            <div className="flex justify-between mb-1">
              <span className="text-base font-medium text-blue-700 dark:text-white">{progressMessage}</span>
              <span className="text-sm font-medium text-blue-700 dark:text-white">{progress?.progress}</span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700">
              <div className="bg-blue-600 h-2.5 rounded-full" style={{ width: `${progress?.progress}` }}></div>
            </div>
            <div className="flex justify-between mt-1">
              <span className="text-sm font-medium text-blue-700 dark:text-white">Download speed: {progress?.speed}</span>
              <span className="text-sm font-medium text-blue-700 dark:text-white">Total size: {progress?.total_size}</span>
            </div>
          </div>
        )
      }

      {/* Download Form */}
      <form onSubmit={(e) => {
        e.preventDefault();
        download();
      }} className="space-y-4">
        <div className="flex gap-2 p-2">
          <input
            type="text"
            placeholder="Select installation directory"
            value={installationDirectory || ""}
            className="flex-1 p-2 border border-gray-200 rounded-md"
            readOnly
          />
          <button
            onClick={(e) => {
              e.preventDefault();
              openDialog();
            }}
            disabled={downloading}
            type="button"
            className="px-4 py-2 bg-gray-200 text-gray-700 rounded-md hover:bg-gray-300"
          >
            Browse
          </button>
        </div>
        <button
          type="submit"
          disabled={installationDirectory === '' || downloading}
          className={`w-full py-2 px-4 rounded-md text-white ${(downloading || installationDirectory === '')
            ? 'bg-gray-400 cursor-not-allowed'
            : 'bg-indigo-600 hover:bg-indigo-700'}`}
        >
          Download and Extract
        </button>
      </form>
    </main >
  );
}

export default App;
