import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { message, open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import ReactMarkdown from "react-markdown";
import "./App.css";

interface ProgressEventPayload {
  name: string;
  total_size: string;
  current_size: string;
  progress: string; // Expecting "XX%"
  speed: string;
}

function App() {
  const [pathNotes, setPathNotes] = useState<string>("");
  const [installationDirectory, setInstallationDirectory] = useState("");
  const [progressMessage, setProgressMessage] = useState("");
  const [progress, setProgress] = useState<ProgressEventPayload>();
  const [downloadStatus, setDownloadStatus] = useState<
    "idle" | "updating" | "downloading"
  >("idle");
  const [currentTab, setCurrentTab] = useState<"instructions" | "patch-notes">(
    "instructions"
  );

  // --- Event Listeners ---
  useEffect(() => {
    const unListen = listen<ProgressEventPayload>(
      "download_progress",
      (event) => {
        setProgressMessage(`Downloading: ${event.payload.name}`);
        setProgress(event.payload);
      }
    );
    return () => {
      unListen.then((f) => f());
    };
  }, []);

  useEffect(() => {
    const unListen = listen<string>("get_patch_notes", (event) => {
      setPathNotes(event.payload);
    });
    return () => {
      unListen.then((f) => f());
    };
  }, []);

  // --- Initial Actions ---
  useEffect(() => {
    // Check for updates once on component mount
    void invoke("check_for_updates", { download: false });
    console.log("Initial update check triggered");
  }, []); // Empty dependency array ensures this runs only once

  // --- Async Functions ---
  async function openDialog() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Installation Directory",
      });
      if (typeof selected === "string") {
        console.log("Selected directory:", selected);
        setInstallationDirectory(selected);
      } else {
        // User cancelled or selected nothing
        // setInstallationDirectory(""); // Keep existing path if cancelled
        console.log("No directory selected or selection cancelled");
      }
    } catch (error) {
      console.error("Error opening directory dialog:", error);
      await message("Could not open directory selector.", {
        title: "Error",
        kind: "error",
      });
    }
  }

  async function download() {
    setDownloadStatus("downloading");
    setProgressMessage("Starting download...");
    setProgress(undefined); // Reset progress
    try {
      await invoke("check_for_updates", {
        downloading: true,
        directory: installationDirectory,
      });
      setProgressMessage("Download complete");
      setDownloadStatus("idle");
    } catch (error) {
      console.error("Download failed:", error);
      setProgressMessage("Download failed.");
      await message(`Download failed: ${error}`, {
        title: "Error",
        kind: "error",
      });
      setDownloadStatus("idle"); // Re-enable buttons on failure
    }
  }

  async function getPatchNotes() {
    if (pathNotes !== "" && pathNotes !== "Fetching patch notes...") return; // Avoid re-fetching if already loaded or loading
    setPathNotes("Fetching patch notes...");
    try {
      await invoke("get_patch_notes"); // This triggers the event listener which updates state
    } catch (error) {
      console.error("Failed to fetch patch notes:", error);
      setPathNotes("Failed to load patch notes.");
      await message("Could not fetch patch notes.", {
        title: "Error",
        kind: "error",
      });
    }
  }

  async function checkUpdates() {
    setProgressMessage("Checking for updates...");
    try {
      let result: boolean = await invoke("check_for_updates", {
        downloading: false,
        directory: installationDirectory,
      });
      if (result) {
        setDownloadStatus("downloading");
        await download();
      }
    } catch (error) {
      console.error("Failed to check for updates");
      await message(`Failed to check for updates.${error}`, {
        title: "Error",
        kind: "error",
      });
      setDownloadStatus("idle"); // Re-enable buttons on failure
    }
  }

  // --- Render ---
  return (
    // Main container: Dark background, full height, flex column layout
    <main className="flex flex-col bg-gray-800  text-slate-800 h-screen font-sans">
      {/* Tabs */}
      <div className="border-b border-slate-200 px-2 pt-2">
        {" "}
        {/* Added padding */}
        <nav className="flex space-x-1" aria-label="Tabs">
          <button
            onClick={() => setCurrentTab("instructions")}
            className={`px-4 py-2 text-sm font-medium rounded-t transition-colors duration-150 ease-in-out ${currentTab === "instructions"
              ? "bg-slate-400 border-b-2 border-gray-200 text-400 text-white" // Active tab style
              : "text-slate-200 hover:bg-slate-400 hover:text-slate-200" // Inactive tab style
              }`}
          >
            Instructions
          </button>
          <button
            onClick={() => {
              void getPatchNotes(); // Fetch notes when tab is clicked (if not already fetched)
              setCurrentTab("patch-notes");
            }}
            className={`px-4 py-2 text-sm font-medium rounded-t transition-colors duration-150 ease-in-out ${currentTab === "patch-notes"
              ? "bg-slate-400 border-b-2 border-gray-200 text-400 text-white" // Active tab style
              : "text-slate-200 hover:bg-slate-400 hover:text-slate-200" // Inactive tab style
              }`}
          >
            Patch Notes
          </button>
        </nav>
      </div>

      {/* Instructions Section */}
      <div className="flex-grow p-4 overflow-y-auto bg-slate-200 text-slate-700 ">
        <div hidden={currentTab !== "instructions"}>
          <div className="prose sm:prose prose-invert min-w-full">
            <h2 className="text-lg font-semibold text-slate-700 mb-3">
              Installation Steps
            </h2>
            <ol className="list-decimal pl-5 space-y-3 text-slate-700">
              <li>
                <span className="font-medium">
                  Choose Installation Location:
                </span>
                <span className="text-red-600 ml-1">
                  Do not install in the Elden Ring game folder!
                </span>
              </li>
              <li>
                <span>
                  Click 'Browse' below and select a folder for installation.
                </span>
                <br />
                <span>
                  For updates: Select your existing mod folder.
                  <br />
                  For new installation: Select an empty folder.
                </span>
              </li>
              <li>
                <span>Click 'Check for updates' to begin.</span>
              </li>
              <li>
                <span>
                  Wait for the download to complete. Files will extract
                  automatically.
                </span>
              </li>
              <li className="bg-yellow-50 p-2 rounded">
                <span className="font-semibold">Note:</span> Extraction may take
                several minutes, especially on HDDs. Please don't close the
                application.
              </li>
            </ol>
          </div>
        </div>

        {/* Patch Notes Tab Content */}
        <div hidden={currentTab !== "patch-notes"}>
          {/* Using prose for nice typography, inverted for dark mode */}
          <div className="prose prose-sm sm:prose text-slate-300  max-w-none">
            {" "}
            {/* Adjusted prose size and max-width */}
            {/* Render markdown content */}
            <ReactMarkdown>
              {pathNotes ||
                "Click 'Patch Notes' tab again or wait for notes to load..."}
            </ReactMarkdown>
          </div>
        </div>
      </div>

      {/* Footer Area: Contains progress and controls */}
      <div className="p-4 border-t border-slate-200 bg-gray-800 text-slate-400 ">
        {" "}
        {/* Footer background */}
        {/* Download Progress Section (Conditional) */}
        {downloadStatus === "downloading" && (
          <div className="mb-4">
            {/* Progress Text */}
            <div className="flex justify-between mb-1 text-sm">
              <span className="font-medium text-slate-200">
                {progressMessage}
              </span>
              <span
                hidden={progress?.progress === "N/A"}
                className="font-medium text-slate-300"
              >
                {progress?.progress} {/* e.g., "50%" */}
              </span>
            </div>
            {/* Progress Bar */}
            <div
              hidden={progress?.progress === "N/A"}
              className="w-full bg-slate-600 rounded h-2.5"
            >
              {" "}
              {/* Bar background */}
              <div
                className="bg-indigo-600 h-2.5 rounded transition-width duration-150 ease-linear" // Bar fill
                style={{ width: progress?.progress ?? "0%" }} // Use progress state
              ></div>
            </div>
            {/* Speed/Size Info */}
            <div className="flex justify-between mt-1 text-xs text-slate-400">
              <span hidden={progress?.speed === "N/A"}>
                Speed: {progress?.speed ?? "N/A"}
              </span>
              <span hidden={progress?.total_size === "N/A"}>
                Size:{" "}
                {progress?.total_size
                  ? `${progress.current_size} / ${progress.total_size}`
                  : "Calculating..."}
              </span>
            </div>
          </div>
        )}
        {/* Action Form */}
        <form
          onSubmit={(e) => {
            e.preventDefault();
            if (installationDirectory === "") {
              void message("Please select an installation directory.", {
                title: "Error",
                kind: "error",
              });
              return;
            }
            if (downloadStatus === "idle") {
              void checkUpdates();
            }
            if (downloadStatus === "updating") {
              void download();
            }
            if (downloadStatus === "downloading") {
              void message("Download in progress. Please wait.", {
                title: "Error",
                kind: "error",
              });
              return;
            }
          }}
          className="space-y-3" // Spacing between form elements
        >
          {/* Directory Selection */}
          <div className="flex gap-2 items-center">
            <input
              type="text"
              placeholder="Select installation directory..."
              value={installationDirectory || ""}
              className="flex-1 px-3 py-2 bg-slate-700 border border-slate-600 rounded text-slate-200 placeholder-slate-300 text-sm focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500"
              readOnly // User selects via button
            />
            <button
              onClick={(e) => {
                e.preventDefault(); // Prevent form submission
                void openDialog();
              }}
              disabled={downloadStatus !== "idle"}
              type="button"
              className="px-4 py-2 bg-slate-600 text-slate-200 rounded text-sm font-medium hover:bg-slate-500 focus:outline-none focus:ring-2 focus:ring-slate-500 focus:ring-offset-2 focus:ring-offset-slate-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-150 ease-in-out"
            >
              Browse
            </button>
          </div>

          {/* Primary and Secondary Actions */}
          <div className="flex items-center gap-2">
            {/* Download Button (Primary) */}
            <button
              type="submit"
              disabled={
                installationDirectory === "" || downloadStatus === "downloading"
              }
              className={`w-full py-2 px-4 rounded text-sm font-semibold text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-800 transition-colors duration-150 ease-in-out ${installationDirectory === "" || downloadStatus === "downloading"
                ? "bg-slate-600 cursor-not-allowed opacity-70" // Disabled style
                : "bg-indigo-600 hover:bg-indigo-700 focus:ring-indigo-500" // Enabled style
                }`}
            >
              {downloadStatus === "downloading" && "Processing..."}
              {downloadStatus === "idle" && "Check for updates"}
              {downloadStatus === "updating" && "Download"}
            </button>
          </div>
        </form>
      </div>
    </main>
  );
}

export default App;
