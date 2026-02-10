import { useEffect, useState } from "react";
import { TorrentInfo } from "./types";
import { api } from "./lib/api";
import { Header } from "./components/Header";
import { Sidebar } from "./components/Sidebar";
import { TorrentTable } from "./components/TorrentTable";
import { TorrentList } from "./components/TorrentList";
import { AddTorrentDialog } from "./components/AddTorrentDialog";
import {
  AddTorrentModal,
  TorrentConfig,
  TorrentMetadata as ModalMetadata,
} from "./components/AddTorrentModal";
import { SettingsDialog } from "./components/SettingsDialog";
import { BottomPanel } from "./components/BottomPanel";
import { ToastContainer } from "./components/Toast";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useToast } from "./hooks/useToast";
import { open } from "@tauri-apps/plugin-dialog";

type ViewMode = "table" | "cards";

function App() {
  const [torrents, setTorrents] = useState<TorrentInfo[]>([]);
  const [filteredTorrents, setFilteredTorrents] = useState<TorrentInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [showAddModal, setShowAddModal] = useState(false);
  const [torrentMetadata, setTorrentMetadata] = useState<ModalMetadata | null>(
    null,
  );
  const [torrentSource, setTorrentSource] = useState<{
    type: "file" | "magnet";
    value: string;
  } | null>(null);
  const [cacheStatus, setCacheStatus] = useState<{
    torbox?: boolean;
    realDebrid?: boolean;
  }>({});
  const [showSettingsDialog, setShowSettingsDialog] = useState(false);
  const [selectedTorrent, setSelectedTorrent] = useState<TorrentInfo | null>(
    null,
  );
  const [error, setError] = useState<string | null>(null);

  // Toast notifications
  const toast = useToast();

  // View state
  const [viewMode, setViewMode] = useState<ViewMode>("table");
  const [selectedFilter, setSelectedFilter] = useState("all");
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [selectedTorrentIds, setSelectedTorrentIds] = useState<Set<string>>(
    new Set(),
  );

  // Load torrents on mount
  useEffect(() => {
    loadTorrents();
  }, []);

  // Auto-refresh torrents every 2 seconds
  useEffect(() => {
    const interval = setInterval(() => {
      refreshTorrents();
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  // Filter torrents when filter or data changes
  useEffect(() => {
    let filtered = [...torrents];

    // Apply status filter
    if (selectedFilter !== "all") {
      filtered = filtered.filter((t) => {
        switch (selectedFilter) {
          case "downloading":
            return t.state === "Downloading";
          case "seeding":
            return t.state === "Seeding";
          case "paused":
            return t.state === "Paused";
          case "active":
            return t.state === "Downloading" || t.state === "Seeding";
          case "completed":
            return t.downloaded >= t.size && t.size > 0;
          case "error":
            return t.state === "Error";
          default:
            return true;
        }
      });
    }

    // Apply category filter (mock for now)
    if (selectedCategory && selectedCategory !== "uncategorized") {
      filtered = []; // No torrents have categories yet
    }

    // Apply tag filter (mock for now)
    if (selectedTags.length > 0) {
      filtered = []; // No torrents have tags yet
    }

    setFilteredTorrents(filtered);
  }, [torrents, selectedFilter, selectedCategory, selectedTags]);

  const loadTorrents = async () => {
    try {
      setIsLoading(true);
      setError(null);

      // Try to load saved torrents first
      const saved = await api.loadSavedTorrents();
      setTorrents(saved);

      // Then get current state
      const current = await api.getTorrents();
      setTorrents(current);
    } catch (err) {
      console.error("Failed to load torrents:", err);
      setError(err instanceof Error ? err.message : "Failed to load torrents");
    } finally {
      setIsLoading(false);
    }
  };

  const refreshTorrents = async () => {
    try {
      const current = await api.getTorrents();
      setTorrents(current);
    } catch (err) {
      console.error("Failed to refresh torrents:", err);
    }
  };

  const handleAddTorrentFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Torrent",
            extensions: ["torrent"],
          },
        ],
      });

      if (selected && typeof selected === "string") {
        // Parse metadata
        toast.info("Parsing torrent...");
        const metadata = await api.parseTorrentFile(selected);

        // Check cache if debrid is enabled
        const cache = await checkTorrentCache(metadata.info_hash);
        if (cache) {
          setCacheStatus(cache);
        }

        // Convert to modal format
        const modalMetadata = {
          name: metadata.name,
          infoHash: metadata.info_hash,
          totalSize: metadata.total_size,
          files: metadata.files.map((f, i) => ({
            path: f.path,
            size: f.size,
            index: i,
          })),
          creationDate: metadata.creation_date || undefined,
          comment: metadata.comment || undefined,
        };

        setTorrentMetadata(modalMetadata);
        setTorrentSource({ type: "file", value: selected });
        setShowAddModal(true);
      }
    } catch (err) {
      console.error("Failed to process torrent file:", err);
      toast.error(
        err instanceof Error ? err.message : "Failed to process torrent file",
      );
    }
  };

  const checkTorrentCache = async (infoHash: string) => {
    try {
      const cacheResults = await api.checkTorrentCache(infoHash);
      return {
        torbox: cacheResults["Torbox"]?.is_cached || false,
        realDebrid: cacheResults["RealDebrid"]?.is_cached || false,
      };
    } catch (err) {
      console.error("Failed to check cache:", err);
      return undefined;
    }
  };

  const handleConfirmAddTorrent = async (config: TorrentConfig) => {
    try {
      if (!torrentSource || !torrentMetadata) {
        throw new Error("No torrent source available");
      }

      console.log("Torrent config:", config);

      // Determine if we should use cloud download
      const useCloud =
        config.downloadMode === "cloud" ||
        (config.downloadMode === "smart" &&
          (cacheStatus?.torbox || cacheStatus?.realDebrid));

      if (useCloud) {
        // Cloud download flow
        toast.info("Adding to cloud service...");

        // Determine which provider to use
        let provider = "torbox";
        if (config.downloadMode === "smart") {
          // Use first available cached provider
          if (cacheStatus?.realDebrid) {
            provider = "real-debrid";
          } else if (cacheStatus?.torbox) {
            provider = "torbox";
          }
        } else {
          // User explicitly chose cloud mode, use preferred provider
          // TODO: Get from settings
          provider = cacheStatus?.realDebrid ? "real-debrid" : "torbox";
        }

        // Get magnet URI or hash
        let magnetOrHash: string;
        if (torrentSource.type === "file") {
          // For file, use info hash
          magnetOrHash = torrentMetadata.infoHash;
        } else {
          // For magnet, use the magnet URI directly
          magnetOrHash = torrentSource.value;
        }

        await api.addCloudTorrent(
          magnetOrHash,
          provider,
          config.savePath || "/downloads",
        );

        toast.success(`Cloud download started via ${provider}`);
      } else {
        // P2P download flow
        toast.info("Adding torrent...");

        if (torrentSource.type === "file") {
          await api.addTorrentFile(torrentSource.value);
        } else {
          await api.addMagnetLink(torrentSource.value);
        }

        toast.success(`Torrent added: ${torrentMetadata.name}`);
      }

      setShowAddModal(false);
      setTorrentMetadata(null);
      setTorrentSource(null);
      setCacheStatus({});
      refreshTorrents();
    } catch (err) {
      console.error("Failed to add torrent:", err);
      toast.error(err instanceof Error ? err.message : "Failed to add torrent");
    }
  };

  const handleTorrentAdded = () => {
    refreshTorrents();
  };

  const handleSelectTorrent = (id: string, multi: boolean) => {
    if (multi) {
      const newSelected = new Set(selectedTorrentIds);
      if (newSelected.has(id)) {
        newSelected.delete(id);
      } else {
        newSelected.add(id);
      }
      setSelectedTorrentIds(newSelected);
    } else {
      setSelectedTorrentIds(new Set([id]));
    }
  };

  const handleTagToggle = (tag: string) => {
    setSelectedTags((prev) =>
      prev.includes(tag) ? prev.filter((t) => t !== tag) : [...prev, tag],
    );
  };

  const handleStartTorrent = async (id: string) => {
    try {
      await api.startTorrent(id);
      await refreshTorrents();
      toast.success("Torrent started");
    } catch (err) {
      console.error("Failed to start torrent:", err);
      toast.error(
        err instanceof Error ? err.message : "Failed to start torrent",
      );
    }
  };

  const handlePauseTorrent = async (id: string) => {
    try {
      await api.pauseTorrent(id);
      await refreshTorrents();
      toast.info("Torrent paused");
    } catch (err) {
      console.error("Failed to pause torrent:", err);
      toast.error(
        err instanceof Error ? err.message : "Failed to pause torrent",
      );
    }
  };

  const handleRemoveTorrent = async (id: string) => {
    try {
      await api.removeTorrent(id, false);
      await refreshTorrents();
      toast.success("Torrent removed");
    } catch (err) {
      console.error("Failed to remove torrent:", err);
      toast.error(
        err instanceof Error ? err.message : "Failed to remove torrent",
      );
    }
  };

  // Keyboard shortcuts
  useKeyboardShortcuts([
    {
      key: "n",
      ctrl: true,
      description: "Add new torrent",
      action: () => setShowAddDialog(true),
    },
    {
      key: ",",
      ctrl: true,
      description: "Open settings",
      action: () => setShowSettingsDialog(true),
    },
    {
      key: "r",
      ctrl: true,
      description: "Refresh torrent list",
      action: () => refreshTorrents(),
    },
    {
      key: "Escape",
      description: "Close dialogs",
      action: () => {
        setShowAddDialog(false);
        setShowSettingsDialog(false);
        setSelectedTorrent(null);
      },
    },
    {
      key: "t",
      ctrl: true,
      description: "Toggle view mode",
      action: () => setViewMode((v) => (v === "table" ? "cards" : "table")),
    },
  ]);

  if (isLoading) {
    return (
      <div className="flex h-screen items-center justify-center bg-dark-bg">
        <div className="text-center">
          <div className="mb-4 inline-block h-12 w-12 animate-spin rounded-full border-4 border-gray-600 border-t-primary" />
          <p className="text-gray-400">Loading SeedCore...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-screen items-center justify-center bg-dark-bg">
        <div className="max-w-md rounded-xl border border-dark-border bg-dark-surface p-6 text-center">
          <div className="mb-4 text-4xl">⚠️</div>
          <h2 className="mb-2 text-xl font-bold text-white">Error</h2>
          <p className="mb-4 text-sm text-gray-400">{error}</p>
          <button
            onClick={loadTorrents}
            className="rounded-lg bg-primary px-4 py-2 text-sm font-medium text-white hover:bg-primary-hover"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen flex-col bg-dark-bg text-white overflow-hidden">
      <Header
        torrents={torrents}
        onAddTorrent={handleAddTorrentFile}
        onOpenSettings={() => setShowSettingsDialog(true)}
        view={viewMode}
        onViewChange={setViewMode}
      />

      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <Sidebar
          torrents={torrents}
          selectedFilter={selectedFilter}
          selectedCategory={selectedCategory}
          selectedTags={selectedTags}
          onFilterChange={setSelectedFilter}
          onCategoryChange={setSelectedCategory}
          onTagToggle={handleTagToggle}
        />

        {/* Main content area with bottom panel */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Top section: Torrent list/table */}
          <main className="flex-1 overflow-hidden p-6">
            {viewMode === "table" ? (
              <TorrentTable
                torrents={filteredTorrents}
                selectedIds={selectedTorrentIds}
                onSelect={handleSelectTorrent}
                onShowDetails={setSelectedTorrent}
                onStart={handleStartTorrent}
                onPause={handlePauseTorrent}
                onRemove={handleRemoveTorrent}
              />
            ) : (
              <div className="h-full overflow-hidden">
                <TorrentList
                  torrents={filteredTorrents}
                  onUpdate={refreshTorrents}
                  onShowDetails={setSelectedTorrent}
                />
              </div>
            )}
          </main>

          {/* Bottom panel: Details tabs */}
          <BottomPanel
            torrent={selectedTorrent}
            isOpen={selectedTorrent !== null}
            onClose={() => setSelectedTorrent(null)}
          />
        </div>
      </div>

      <AddTorrentDialog
        isOpen={showAddDialog}
        onClose={() => setShowAddDialog(false)}
        onAdded={handleTorrentAdded}
      />

      {torrentMetadata && (
        <AddTorrentModal
          isOpen={showAddModal}
          onClose={() => {
            setShowAddModal(false);
            setTorrentMetadata(null);
            setTorrentSource(null);
          }}
          onConfirm={handleConfirmAddTorrent}
          metadata={torrentMetadata}
          defaultSavePath="/home/downloads"
          cacheStatus={cacheStatus}
        />
      )}

      <SettingsDialog
        isOpen={showSettingsDialog}
        onClose={() => setShowSettingsDialog(false)}
      />

      {/* Toast notifications */}
      <ToastContainer toasts={toast.toasts} onClose={toast.removeToast} />
    </div>
  );
}

export default App;
