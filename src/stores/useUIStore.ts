import { create } from "zustand";
import { persist } from "zustand/middleware";
import { TorrentInfo } from "../types";

export interface Toast {
    id: string;
    type: "success" | "error" | "info" | "warning";
    message: string;
    duration?: number;
}

interface UIStore {
    // View Mode
    viewMode: "table" | "cards";
    setViewMode: (mode: "table" | "cards") => void;

    // Modals / Dialogs
    isAddTorrentOpen: boolean;
    openAddTorrentDialog: () => void;
    closeAddTorrentDialog: () => void;

    isSettingsOpen: boolean;
    openSettings: () => void;
    closeSettings: () => void;

    // Add Torrent Wizard Data
    addTorrentModal: {
        isOpen: boolean;
        metadata: any; // Using any to support both API and Modal specific metadata structures
        source: { type: "file"; path: string } | { type: "magnet"; uri: string } | null;
        cacheStatus: any;
    };
    openAddTorrentModal: (data: {
        metadata: any;
        source: { type: "file"; path: string } | { type: "magnet"; uri: string };
        cacheStatus?: any;
    }) => void;
    closeAddTorrentModal: () => void;

    // Details Panel
    selectedTorrent: TorrentInfo | null;
    openDetails: (torrent: TorrentInfo) => void;
    closeDetails: () => void;

    // Toasts
    toasts: Toast[];
    addToast: (type: Toast["type"], message: string, duration?: number) => void;
    removeToast: (id: string) => void;
}

export const useUIStore = create<UIStore>()(
    persist(
        (set) => ({
            // View Mode
            viewMode: "cards",
            setViewMode: (mode) => set({ viewMode: mode }),

            // Modals
            isAddTorrentOpen: false,
            openAddTorrentDialog: () => set({ isAddTorrentOpen: true }),
            closeAddTorrentDialog: () => set({ isAddTorrentOpen: false }),

            isSettingsOpen: false,
            openSettings: () => set({ isSettingsOpen: true }),
            closeSettings: () => set({ isSettingsOpen: false }),

            // Add Torrent Wizard
            addTorrentModal: {
                isOpen: false,
                metadata: null,
                source: null,
                cacheStatus: {},
            },
            openAddTorrentModal: (data) =>
                set({
                    addTorrentModal: {
                        isOpen: true,
                        metadata: data.metadata,
                        source: data.source,
                        cacheStatus: data.cacheStatus || {},
                    },
                }),
            closeAddTorrentModal: () =>
                set({
                    addTorrentModal: {
                        isOpen: false,
                        metadata: null,
                        source: null,
                        cacheStatus: {},
                    },
                }),

            // Details Panel
            selectedTorrent: null,
            openDetails: (torrent) => set({ selectedTorrent: torrent }),
            closeDetails: () => set({ selectedTorrent: null }),

            // Toasts
            toasts: [],
            addToast: (type, message, duration = 5000) => {
                const id = Math.random().toString(36).substring(7);
                set((state) => ({
                    toasts: [...state.toasts, { id, type, message, duration }],
                }));
            },
            removeToast: (id) =>
                set((state) => ({
                    toasts: state.toasts.filter((t) => t.id !== id),
                })),
        }),
        {
            name: "ui-storage",
            partialize: (state) => ({ viewMode: state.viewMode }), // Only persist viewMode
        },
    ),
);
