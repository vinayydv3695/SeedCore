// Tauri API wrapper for type-safe command invocations
import { invoke } from "@tauri-apps/api/core";
import {
  TorrentInfo,
  TorrentMetadata,
  Settings,
  DebridSettings,
  CredentialStatus,
  CacheStatus,
  DebridFile,
  DebridProgress,
} from "../types";

export const api = {
  // Torrent operations
  async getTorrents(): Promise<TorrentInfo[]> {
    return invoke("get_torrents");
  },

  async parseTorrentFile(filePath: string): Promise<TorrentMetadata> {
    return invoke("parse_torrent_file", { filePath });
  },

  async parseMagnetLink(magnetUri: string): Promise<TorrentMetadata> {
    return invoke("parse_magnet_link", { magnetUri });
  },

  async addTorrentFile(filePath: string): Promise<string> {
    return invoke("add_torrent_file", { filePath });
  },

  async addMagnetLink(magnetUri: string): Promise<string> {
    return invoke("add_magnet_link", { magnetUri });
  },

  async addCloudTorrent(
    magnetOrHash: string,
    provider: string,
    savePath: string,
  ): Promise<string> {
    return invoke("add_cloud_torrent", { magnetOrHash, provider, savePath });
  },

  async removeTorrent(torrentId: string, deleteFiles: boolean): Promise<void> {
    return invoke("remove_torrent", { torrentId, deleteFiles });
  },

  async startTorrent(torrentId: string): Promise<void> {
    return invoke("start_torrent", { torrentId });
  },

  async pauseTorrent(torrentId: string): Promise<void> {
    return invoke("pause_torrent", { torrentId });
  },

  async getTorrentDetails(torrentId: string): Promise<TorrentInfo> {
    return invoke("get_torrent_details", { torrentId });
  },

  async loadSavedTorrents(): Promise<TorrentInfo[]> {
    return invoke("load_saved_torrents");
  },

  // Settings operations
  async getSettings(): Promise<Settings> {
    return invoke("get_settings");
  },

  async updateSettings(settings: Settings): Promise<void> {
    return invoke("update_settings", { settings });
  },

  // App info
  async getVersion(): Promise<string> {
    return invoke("get_version");
  },

  async greet(name: string): Promise<string> {
    return invoke("greet", { name });
  },

  // Torrent monitoring operations
  async getPeerList(torrentId: string): Promise<
    {
      ip: string;
      port: number;
      client: string;
      flags: string;
      progress: number;
      download_speed: number;
      upload_speed: number;
      downloaded: number;
      uploaded: number;
    }[]
  > {
    return invoke("get_peer_list", { torrentId });
  },

  async getTrackerList(torrentId: string): Promise<
    {
      url: string;
      status: string;
      message: string;
      peers: number;
      seeds: number;
      leechers: number;
      downloaded: number;
      last_announce: number | null;
      next_announce: number | null;
    }[]
  > {
    return invoke("get_tracker_list", { torrentId });
  },

  async getPiecesInfo(torrentId: string): Promise<{
    total_pieces: number;
    pieces_have: number;
    pieces_downloading: number;
    bitfield: number[];
    availability: number[];
  }> {
    return invoke("get_pieces_info", { torrentId });
  },

  async getFileList(torrentId: string): Promise<
    {
      path: string;
      size: number;
      downloaded: number;
      priority: "Skip" | "Low" | "Normal" | "High";
      is_folder: boolean;
    }[]
  > {
    return invoke("get_file_list", { torrentId });
  },

  async setFilePriority(
    torrentId: string,
    filePath: string,
    priority: "Skip" | "Low" | "Normal" | "High",
  ): Promise<void> {
    return invoke("set_file_priority", { torrentId, filePath, priority });
  },

  // Debrid - Master Password operations
  async checkMasterPasswordSet(): Promise<boolean> {
    return invoke("check_master_password_set");
  },

  async setMasterPassword(password: string): Promise<void> {
    return invoke("set_master_password", { password });
  },

  async unlockWithMasterPassword(password: string): Promise<boolean> {
    return invoke("unlock_with_master_password", { password });
  },

  async changeMasterPassword(
    oldPassword: string,
    newPassword: string,
  ): Promise<void> {
    return invoke("change_master_password", { oldPassword, newPassword });
  },

  async lockDebridServices(): Promise<void> {
    return invoke("lock_debrid_services");
  },

  // Debrid - Credential Management
  async saveDebridCredentials(provider: string, apiKey: string): Promise<void> {
    return invoke("save_debrid_credentials", { provider, apiKey });
  },

  async getDebridCredentialsStatus(): Promise<CredentialStatus[]> {
    return invoke("get_debrid_credentials_status");
  },

  async deleteDebridCredentials(provider: string): Promise<void> {
    return invoke("delete_debrid_credentials", { provider });
  },

  async validateDebridProvider(provider: string): Promise<boolean> {
    return invoke("validate_debrid_provider", { provider });
  },

  // Debrid - Cache Check
  async checkTorrentCache(
    infoHash: string,
  ): Promise<Record<string, CacheStatus>> {
    return invoke("check_torrent_cache", { infoHash });
  },

  async getPreferredCachedProvider(infoHash: string): Promise<string | null> {
    return invoke("get_preferred_cached_provider", { infoHash });
  },

  // Debrid - Torrent Management
  async addMagnetToDebrid(magnet: string, provider: string): Promise<string> {
    return invoke("add_magnet_to_debrid", { magnet, provider });
  },

  async addTorrentFileToDebrid(
    filePath: string,
    provider: string,
  ): Promise<string> {
    return invoke("add_torrent_file_to_debrid", { filePath, provider });
  },

  async selectDebridFiles(
    torrentId: string,
    provider: string,
    fileIndices: number[],
  ): Promise<void> {
    return invoke("select_debrid_files", { torrentId, provider, fileIndices });
  },

  async getDebridDownloadLinks(
    torrentId: string,
    provider: string,
  ): Promise<DebridFile[]> {
    return invoke("get_debrid_download_links", { torrentId, provider });
  },

  async listDebridTorrents(provider: string): Promise<DebridProgress[]> {
    return invoke("list_debrid_torrents", { provider });
  },

  async deleteDebridTorrent(
    torrentId: string,
    provider: string,
  ): Promise<void> {
    return invoke("delete_debrid_torrent", { torrentId, provider });
  },

  async getCloudFileProgress(torrentId: string): Promise<
    {
      name: string;
      size: number;
      downloaded: number;
      speed: number;
      state: "Queued" | "Downloading" | "Complete" | "Error";
    }[]
  > {
    return invoke("get_cloud_file_progress", { torrentId });
  },

  // Debrid - Settings
  async getDebridSettings(): Promise<DebridSettings> {
    return invoke("get_debrid_settings");
  },

  async updateDebridSettings(settings: DebridSettings): Promise<void> {
    return invoke("update_debrid_settings", { settings });
  },

  // System utilities
  async getAvailableDiskSpace(path: string): Promise<number> {
    return invoke("get_available_disk_space", { path });
  },
};
