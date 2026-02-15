import { create } from "zustand";
import { TorrentInfo, TorrentState, TorrentConfig } from "../types";
import { api } from "../lib/api";
import { useUIStore } from "./useUIStore";

interface TorrentStoreState {
  // Data
  torrents: TorrentInfo[];
  filteredTorrents: TorrentInfo[];
  isLoading: boolean;
  error: string | null;

  // Filters
  filterStatus: string;
  searchQuery: string;
  selectedParams: {
    category: string | null;
    tags: string[];
  };

  // Selection
  selectedIds: Set<string>;

  // Event Listeners
  unlisten: (() => void) | null;

  // Actions
  fetchTorrents: () => Promise<void>;
  refreshTorrents: () => Promise<void>;

  setFilterStatus: (status: string) => void;
  setSearchQuery: (query: string) => void;
  setCategory: (category: string | null) => void;
  toggleTag: (tag: string) => void;

  toggleSelection: (id: string, multi: boolean) => void;
  selectAll: () => void;
  deselectAll: () => void;

  setupListeners: () => Promise<void>;
  cleanupListeners: () => void;

  // Torrent Actions
  startTorrent: (id: string) => Promise<void>;
  pauseTorrent: (id: string) => Promise<void>;
  removeTorrent: (id: string, deleteFiles: boolean) => Promise<void>;
  addTorrent: (config: TorrentConfig) => Promise<void>;
  setFilePriority: (
    id: string,
    fileIndex: number,
    priority: number,
  ) => Promise<void>;

  // Internal
  applyFilters: () => void;
}

import { listen } from "@tauri-apps/api/event";

export const useTorrentStore = create<TorrentStoreState>((set, get) => ({
  // Initial State
  torrents: [],
  filteredTorrents: [],
  isLoading: false,
  error: null,

  filterStatus: "all",
  searchQuery: "",
  selectedParams: {
    category: null,
    tags: [],
  },

  selectedIds: new Set(),
  unlisten: null,

  // Fetching
  fetchTorrents: async () => {
    set({ isLoading: true, error: null });
    try {
      // Load saved torrents from database (which also creates engines and starts them)
      const torrents = await api.loadSavedTorrents();
      set({ torrents, isLoading: false });
      get().applyFilters();
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Failed to load torrents";
      set({ error: message, isLoading: false });
      useUIStore.getState().addToast("error", message);
    }
  },

  refreshTorrents: async () => {
    try {
      const current = await api.getTorrents();
      set({ torrents: current });
      get().applyFilters();
    } catch (err) {
      console.error("Failed to refresh torrents:", err);
    }
  },

  // Filters
  setFilterStatus: (status) => {
    set({ filterStatus: status });
    get().applyFilters();
  },

  setSearchQuery: (query) => {
    set({ searchQuery: query });
    get().applyFilters();
  },

  setCategory: (category) => {
    set((state) => ({
      selectedParams: { ...state.selectedParams, category },
    }));
    get().applyFilters();
  },

  toggleTag: (tag) => {
    set((state) => {
      const tags = state.selectedParams.tags.includes(tag)
        ? state.selectedParams.tags.filter((t) => t !== tag)
        : [...state.selectedParams.tags, tag];
      return { selectedParams: { ...state.selectedParams, tags } };
    });
    get().applyFilters();
  },

  applyFilters: () => {
    const { torrents, filterStatus, searchQuery, selectedParams } = get();

    console.log(
      "Applying filters - Total torrents:",
      torrents.length,
      "Filter status:",
      filterStatus,
    );

    let filtered = [...torrents];

    // 1. Status Filter
    if (filterStatus !== "all") {
      filtered = filtered.filter((t) => {
        switch (filterStatus) {
          case "downloading":
            return t.state === TorrentState.Downloading;
          case "seeding":
            return t.state === TorrentState.Seeding;
          case "paused":
            return t.state === TorrentState.Paused;
          case "active":
            return (
              t.state === TorrentState.Downloading ||
              t.state === TorrentState.Seeding
            );
          case "completed":
            return t.downloaded >= t.size && t.size > 0;
          case "error":
            return t.state === TorrentState.Error;
          default:
            return true;
        }
      });
    }

    // 2. Search Filter
    if (searchQuery) {
      const lowerQuery = searchQuery.toLowerCase();
      filtered = filtered.filter((t) =>
        t.name.toLowerCase().includes(lowerQuery),
      );
    }

    // 3. Category Filter (Mock)
    if (
      selectedParams.category &&
      selectedParams.category !== "uncategorized"
    ) {
      filtered = []; // No category support yet
    }

    // 4. Tags Filter (Mock)
    if (selectedParams.tags.length > 0) {
      filtered = []; // No tag support yet
    }

    console.log("Filtered torrents:", filtered.length);
    set({ filteredTorrents: filtered });
  },

  // Selection
  toggleSelection: (id, multi) => {
    set((state) => {
      const newSelected = new Set(multi ? state.selectedIds : []);
      if (multi && state.selectedIds.has(id)) {
        newSelected.delete(id);
      } else {
        newSelected.add(id);
      }
      return { selectedIds: newSelected };
    });
  },

  selectAll: () => {
    const ids = new Set(get().filteredTorrents.map((t) => t.id));
    set({ selectedIds: ids });
  },

  deselectAll: () => {
    set({ selectedIds: new Set() });
  },

  // Listeners
  setupListeners: async () => {
    const currentUnlisten = get().unlisten;
    if (currentUnlisten) {
      console.warn("Listeners already setup, skipping duplicate setup");
      return;
    }

    // Initial fetch
    await get().fetchTorrents();

    // Listen for updates
    const unlistenFn = await listen<TorrentInfo>("torrent-update", (event) => {
      try {
        const update = event.payload;
        console.log("Received torrent-update:", update.id, update.name);

        set((state) => {
          const index = state.torrents.findIndex((t) => t.id === update.id);
          let newTorrents;
          if (index !== -1) {
            newTorrents = [...state.torrents];
            newTorrents[index] = update;
          } else {
            console.log("Adding new torrent to list:", update.name);
            newTorrents = [...state.torrents, update];
          }
          return { torrents: newTorrents };
        });
        get().applyFilters();
      } catch (err) {
        console.error("Error processing torrent-update event:", err);
      }
    });

    set({ unlisten: unlistenFn });
  },

  cleanupListeners: () => {
    const { unlisten } = get();
    if (unlisten) {
      unlisten();
      set({ unlisten: null });
    }
  },

  // Actions
  startTorrent: async (id) => {
    try {
      await api.startTorrent(id);
      // No need to refresh, event will update
      useUIStore.getState().addToast("success", "Torrent started");
    } catch (err) {
      const msg =
        err instanceof Error ? err.message : "Failed to start torrent";
      useUIStore.getState().addToast("error", msg);
    }
  },

  pauseTorrent: async (id) => {
    try {
      await api.pauseTorrent(id);
      // No need to refresh
      useUIStore.getState().addToast("info", "Torrent paused");
    } catch (err) {
      const msg =
        err instanceof Error ? err.message : "Failed to pause torrent";
      useUIStore.getState().addToast("error", msg);
    }
  },

  removeTorrent: async (id, deleteFiles) => {
    try {
      await api.removeTorrent(id, deleteFiles);
      set((state) => {
        const newSelected = new Set(state.selectedIds);
        newSelected.delete(id);
        // We should also remove it from the list immediately for better UX
        const newTorrents = state.torrents.filter((t) => t.id !== id);
        return { selectedIds: newSelected, torrents: newTorrents };
      });
      get().applyFilters();
      useUIStore.getState().addToast("success", "Torrent removed");
    } catch (err) {
      const msg =
        err instanceof Error ? err.message : "Failed to remove torrent";
      useUIStore.getState().addToast("error", msg);
    }
  },

  addTorrent: async (config) => {
    try {
      const { source, metadata, cacheStatus } =
        useUIStore.getState().addTorrentModal;

      if (!source || !metadata) {
        throw new Error("No torrent source available");
      }

      // Determine if we should use cloud download
      const useCloud =
        config.downloadMode === "cloud" ||
        (config.downloadMode === "smart" &&
          (cacheStatus?.torbox || cacheStatus?.realDebrid));

      if (useCloud) {
        useUIStore.getState().addToast("info", "Adding to cloud service...");

        let provider = "torbox";
        // Simple logic for provider selection based on cache
        if (config.downloadMode === "smart") {
          if (cacheStatus?.realDebrid) provider = "real-debrid";
          else if (cacheStatus?.torbox) provider = "torbox";
        } else {
          provider = cacheStatus?.realDebrid ? "real-debrid" : "torbox";
          if (config.debridProvider) provider = config.debridProvider;
        }

        const magnetOrHash =
          source.type === "file" ? metadata.infoHash : source.uri;

        await api.addCloudTorrent(
          magnetOrHash,
          provider,
          config.savePath || "/downloads",
        );

        useUIStore
          .getState()
          .addToast("success", `Cloud download started via ${provider}`);
      } else {
        useUIStore.getState().addToast("info", "Adding torrent...");

        if (source.type === "file") {
          await api.addTorrentFile(source.path);
        } else {
          await api.addMagnetLink(source.uri);
        }

        useUIStore
          .getState()
          .addToast("success", `Torrent added: ${metadata.name}`);
      }

      useUIStore.getState().closeAddTorrentModal();

      // Refresh torrents list immediately after adding
      // This ensures the UI updates even if events are delayed
      await get().refreshTorrents();
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to add torrent";
      console.error("Error adding torrent:", err);
      useUIStore.getState().addToast("error", msg);
    }
  },

  setFilePriority: async (torrentId, fileIndex, priority) => {
    try {
      await api.setFilePriority(torrentId, fileIndex, priority);
      // ui update will happen via event or we could optimistically update if we had file list in store
      // for now, just toast
      const priorityName =
        ["Skip", "Low", "Normal", "High", "Critical"][priority] || "Unknown";
      useUIStore
        .getState()
        .addToast("success", `Priority set to ${priorityName}`);
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to set priority";
      useUIStore.getState().addToast("error", msg);
    }
  },
}));
