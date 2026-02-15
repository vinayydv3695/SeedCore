// TypeScript types for SeedCore

export enum TorrentState {
  Downloading = "Downloading",
  Seeding = "Seeding",
  Paused = "Paused",
  Checking = "Checking",
  Error = "Error",
  Queued = "Queued",
}

export enum DownloadSource {
  P2P = "P2P",
  Cloud = "Cloud",
  Hybrid = "Hybrid",
}

export interface TorrentInfo {
  id: string;
  name: string;
  size: number;
  downloaded: number;
  uploaded: number;
  state: TorrentState;
  download_speed: number;
  upload_speed: number;
  peers: number;
  seeds: number;
  source: DownloadSource;
}

export interface Settings {
  download_limit: number;
  upload_limit: number;
  max_active_downloads: number;
  max_active_uploads: number;
  listen_port: number;
  enable_dht: boolean;
  enable_pex: boolean;
  dark_mode: boolean;
  download_path: string;
  // Auto-cleanup settings
  cleanup_enabled: boolean;
  cleanup_ratio: number;
  cleanup_time: number;
  cleanup_mode: string;
  // Bandwidth scheduler settings
  bandwidth_scheduler_enabled: boolean;
  bandwidth_schedule: BandwidthRule[];
}

export interface BandwidthRule {
  start_time: string;
  end_time: string;
  days: number[]; // 0=Sunday, 6=Saturday
  download_limit: number;
  upload_limit: number;
  enabled: boolean;
}

export interface AppStats {
  total_download_speed: number;
  total_upload_speed: number;
  total_downloaded: number;
  total_uploaded: number;
  active_torrents: number;
  total_torrents: number;
}

// Peer monitoring types
export interface PeerInfo {
  ip: string;
  port: number;
  client: string;
  flags: string;
  progress: number;
  download_speed: number;
  upload_speed: number;
  downloaded: number;
  uploaded: number;
}

// Tracker monitoring types
export type TrackerStatus = "Working" | "Updating" | "Error" | "Disabled";

export interface TrackerInfo {
  url: string;
  status: TrackerStatus;
  message: string;
  peers: number;
  seeds: number;
  leechers: number;
  downloaded: number;
  last_announce: number | null;
  next_announce: number | null;
}

// Pieces monitoring types
export interface PiecesInfo {
  total_pieces: number;
  pieces_have: number;
  pieces_downloading: number;
  bitfield: number[]; // 0=missing, 1=have, 2=downloading
  availability: number[]; // How many peers have each piece
}

// File monitoring types
export type FilePriority = "high" | "normal" | "low" | "skip"; // Updated to match AddTorrentModal lower case usage

export interface FileInfo {
  path: string;
  size: number;
  downloaded: number;
  priority: "Skip" | "Low" | "Normal" | "High"; // API uses Capitalized?
  is_folder: boolean;
}

// Note: FileInfo priority from API is capitalized "Skip" etc.
// But AddTorrentModal uses lowercase "skip".
// We might need to handle conversion.
// For now, keeping FileInfo as per previous file content which had "Skip" etc.
// But FilePriority type export I changed to lowercase to match AddTorrentModal usage in TorrentConfig.

// Torrent metadata (before adding to client)
export interface TorrentMetadata {
  name: string;
  info_hash: string;
  total_size: number;
  files: FileInfo[];
  announce: string;
  creation_date: number | null;
  comment: string | null;
  created_by: string | null;
}

// Debrid types
export interface DebridSettings {
  enable_debrid: boolean;
  debrid_preference: string[]; // ["torbox", "real-debrid"]
  smart_mode_enabled: boolean;
}

export interface CredentialStatus {
  provider: string;
  is_configured: boolean;
  is_valid: boolean | null;
  last_validated: number | null;
}

export interface CachedFile {
  id: number;
  name: string;
  size: number;
  selected: boolean;
}

export interface CacheStatus {
  is_cached: boolean;
  files: CachedFile[];
  instant_download: boolean;
}

export interface DebridFile {
  id: string;
  name: string;
  size: number;
  download_link: string | null;
  stream_link: string | null;
  mime_type: string | null;
}

export type DebridStatus =
  | "WaitingFilesSelection"
  | "Queued"
  | "Downloading"
  | "Downloaded"
  | "Compressing"
  | "Uploading"
  | "Error"
  | "Dead"
  | "MagnetConversion";

export interface DebridProgress {
  torrent_id: string;
  status: DebridStatus;
  progress: number; // 0-100
  speed: number; // bytes/sec
  downloaded: number;
  total_size: number;
  seeders: number | null;
  eta: number | null; // seconds
}

// Configuration types
export type DownloadMode = "smart" | "cloud" | "p2p" | "hybrid";

export interface FileSelection {
  selected: boolean;
  priority: FilePriority;
}

export interface TorrentConfig {
  savePath: string;
  useIncompletePath: boolean;
  incompletePath?: string;
  category: string;
  tags: string[];
  startImmediately: boolean;
  addToQueue: boolean;
  sequentialDownload: boolean;
  downloadFirstLast: boolean;
  skipHashCheck: boolean;
  downloadMode: DownloadMode;
  debridProvider?: string;
  selectedFiles: number[];
  filePriorities: { [index: number]: FilePriority };
}
