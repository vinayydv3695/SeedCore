import { useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { TopBar } from "./components/layout/TopBar";
import { Sidebar } from "./components/layout/Sidebar";
import { BottomPanel } from "./components/layout/BottomPanel";
import { TorrentTable } from "./components/dashboard/TorrentTable";
import { TorrentList } from "./components/dashboard/TorrentList";
import { AddTorrentModal } from "./components/AddTorrentModal";
import { SettingsDialog } from "./components/SettingsDialog";
import { Toaster } from "./components/Toast";
import { api } from "./lib/api";
import { useTorrentStore } from "./stores/useTorrentStore";
import { useUIStore } from "./stores/useUIStore";
import { useSettingsStore } from "./stores/useSettingsStore";
import { TorrentConfig } from "./types";

function App() {
  // Stores
  const viewMode = useUIStore((state) => state.viewMode);

  const addTorrentModal = useUIStore((state) => state.addTorrentModal);
  const openAddTorrentModal = useUIStore((state) => state.openAddTorrentModal);
  const closeAddTorrentModal = useUIStore((state) => state.closeAddTorrentModal);

  const isAddTorrentOpen = useUIStore((state) => state.isAddTorrentOpen);
  const closeAddTorrentDialog = useUIStore((state) => state.closeAddTorrentDialog);

  const isSettingsOpen = useUIStore((state) => state.isSettingsOpen);
  const closeSettings = useUIStore((state) => state.closeSettings);


  const setupListeners = useTorrentStore((state) => state.setupListeners);
  const cleanupListeners = useTorrentStore((state) => state.cleanupListeners);
  const addTorrent = useTorrentStore((state) => state.addTorrent);

  const settings = useSettingsStore((state) => state.settings);
  const diskSpace = useSettingsStore((state) => state.diskSpace);
  const fetchSettings = useSettingsStore((state) => state.fetchSettings);
  const fetchDiskSpace = useSettingsStore((state) => state.fetchDiskSpace);

  // Initial load
  useEffect(() => {
    fetchSettings();
    setupListeners();

    return () => {
      cleanupListeners();
    };
  }, []);

  // Check disk space when settings change (download path)
  useEffect(() => {
    if (settings?.download_path) {
      fetchDiskSpace(settings.download_path);
    }
  }, [settings?.download_path]);

  // Handle file processing (File Drop or Open File)
  const processFile = useCallback(async (path: string) => {
    try {
      useUIStore.getState().addToast("info", "Parsing torrent...");
      const metadata = await api.parseTorrentFile(path);

      // Check cache
      let cacheStatus = {};
      try {
        const cacheResults = await api.checkTorrentCache(metadata.info_hash);
        cacheStatus = {
          torbox: cacheResults["Torbox"]?.is_cached || false,
          realDebrid: cacheResults["RealDebrid"]?.is_cached || false,
        };
      } catch (e) {
        console.error("Failed to check cache:", e);
      }

      openAddTorrentModal({
        metadata: {
          ...metadata,
          // Map API metadata to UI metadata types if needed, or matched
          files: metadata.files.map((f, i) => ({
            path: f.path,
            size: f.size,
            index: i,
          }))
        },
        source: { type: "file", path },
        cacheStatus
      });
    } catch (err) {
      console.error("Failed to process torrent file:", err);
      useUIStore.getState().addToast("error", "Failed to process torrent file");
    }
  }, [openAddTorrentModal]);

  // Handle "Add Torrent" button click - opens OS file dialog then processes
  const handleAddTorrentClick = useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Torrent Files",
            extensions: ["torrent"],
          },
        ],
      });

      if (selected && typeof selected === "string") {
        processFile(selected);
      }
    } catch (err) {
      console.error("Failed to open file dialog:", err);
      useUIStore.getState().addToast("error", "Failed to open file dialog");
    }
  }, [processFile]);

  // When TopBar's "Add Torrent" button is clicked, trigger file dialog
  useEffect(() => {
    if (isAddTorrentOpen) {
      closeAddTorrentDialog(); // Reset the flag immediately
      handleAddTorrentClick();
    }
  }, [isAddTorrentOpen, closeAddTorrentDialog, handleAddTorrentClick]);

  // Event Listeners
  useEffect(() => {
    const unlistenFileDrop = listen("tauri://file-drop", (event) => {
      const paths = event.payload as string[];
      if (paths && paths.length > 0) {
        processFile(paths[0]);
      }
    });

    const unlistenMenuOpen = listen("menu-open-file", async () => {
      handleAddTorrentClick();
    });

    return () => {
      unlistenFileDrop.then((fn) => fn());
      unlistenMenuOpen.then((fn) => fn());
    };
  }, []);

  const handleTorrentConfirm = async (config: TorrentConfig) => {
    await addTorrent(config);
  };

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-dark-bg font-sans text-text-primary">
      {/* Sidebar */}
      <div className="shrink-0 h-full">
        <Sidebar />
      </div>

      <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
        {/* TopBar */}
        <TopBar />

        {/* Main Content Area */}
        <main className="relative flex flex-1 flex-col overflow-hidden bg-dark-bg p-4">
          <div className="flex-1 overflow-hidden rounded-lg border border-dark-border bg-dark-tertiary shadow-sm relative">
            {/* Background Pattern or Empty State could go here */}
            {viewMode === "table" ? (
              <TorrentTable />
            ) : (
              <div className="h-full overflow-hidden">
                <TorrentList />
              </div>
            )}
          </div>

          <BottomPanel />
        </main>
      </div>

      {/* Modals & Overlays */}
      {addTorrentModal.isOpen && addTorrentModal.metadata && (
        <AddTorrentModal
          isOpen={true}
          onClose={closeAddTorrentModal}
          onConfirm={handleTorrentConfirm}
          metadata={addTorrentModal.metadata}
          defaultSavePath={settings?.download_path || "/downloads"}
          availableDiskSpace={diskSpace?.free}
          cacheStatus={addTorrentModal.cacheStatus}
        />
      )}

      {isSettingsOpen && (
        <SettingsDialog
          isOpen={true}
          onClose={closeSettings}
        />
      )}

      <Toaster />
    </div>
  );
}

export default App;
