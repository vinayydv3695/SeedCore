import { create } from "zustand";
import { Settings } from "../types";
import { api } from "../lib/api";

interface DiskSpace {
    free: number;
    total: number;
}

interface SettingsStore {
    settings: Settings | null;
    diskSpace: DiskSpace | null;
    isLoading: boolean;
    error: string | null;

    fetchSettings: () => Promise<void>;
    fetchDiskSpace: (path: string) => Promise<void>;
    updateSettings: (newSettings: Settings) => Promise<void>;
}

export const useSettingsStore = create<SettingsStore>((set) => ({
    settings: null,
    diskSpace: null,
    isLoading: false,
    error: null,

    fetchSettings: async () => {
        set({ isLoading: true, error: null });
        try {
            const settings = await api.getSettings();
            set({ settings, isLoading: false });
        } catch (err) {
            const message = err instanceof Error ? err.message : "Failed to load settings";
            set({ error: message, isLoading: false });
        }
    },

    fetchDiskSpace: async (path: string) => {
        try {
            const free = await api.getAvailableDiskSpace(path);
            // We only get free space from API currently
            set({ diskSpace: { free, total: 0 } });
        } catch (err) {
            console.error("Failed to fetch disk space:", err);
        }
    },

    updateSettings: async (newSettings: Settings) => {
        set({ isLoading: true, error: null });
        try {
            await api.updateSettings(newSettings);
            set({ settings: newSettings, isLoading: false });
        } catch (err) {
            const message = err instanceof Error ? err.message : "Failed to save settings";
            set({ error: message, isLoading: false });
            throw err;
        }
    },
}));
