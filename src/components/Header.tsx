import { TorrentInfo } from "../types";
import { formatSpeed, formatBytes } from "../lib/utils";
import { ViewToggle } from "./ViewToggle";

interface HeaderProps {
  torrents: TorrentInfo[];
  onAddTorrent: () => void;
  onOpenSettings: () => void;
  view?: "table" | "cards";
  onViewChange?: (view: "table" | "cards") => void;
}

export function Header({ torrents, onAddTorrent, onOpenSettings, view = "cards", onViewChange }: HeaderProps) {
  // Calculate global stats
  const totalDownloadSpeed = torrents.reduce((sum, t) => sum + t.download_speed, 0);
  const totalUploadSpeed = torrents.reduce((sum, t) => sum + t.upload_speed, 0);
  const totalDownloaded = torrents.reduce((sum, t) => sum + t.downloaded, 0);
  const totalUploaded = torrents.reduce((sum, t) => sum + t.uploaded, 0);
  const activeTorrents = torrents.filter(
    (t) => t.state === "Downloading" || t.state === "Seeding"
  ).length;

  return (
    <header className="border-b border-dark-border bg-dark-surface px-6 py-4">
      <div className="flex items-center justify-between">
        {/* Logo and title */}
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary-hover">
            <LogoIcon />
          </div>
          <div>
            <h1 className="text-xl font-bold text-white">SeedCore</h1>
            <p className="text-xs text-gray-400">
              {activeTorrents} active • {torrents.length} total
            </p>
          </div>
        </div>

        {/* Stats */}
        <div className="hidden items-center gap-8 md:flex">
          <Stat
            label="Download"
            value={formatSpeed(totalDownloadSpeed)}
            icon={<DownloadIcon />}
            color="text-primary"
          />
          <Stat
            label="Upload"
            value={formatSpeed(totalUploadSpeed)}
            icon={<UploadIcon />}
            color="text-success"
          />
          <div className="h-8 w-px bg-dark-border" />
          <Stat
            label="Downloaded"
            value={formatBytes(totalDownloaded)}
            icon={<ArrowDownIcon />}
            color="text-blue-400"
          />
          <Stat
            label="Uploaded"
            value={formatBytes(totalUploaded)}
            icon={<ArrowUpIcon />}
            color="text-green-400"
          />
        </div>

        {/* Action buttons */}
        <div className="flex items-center gap-2">
          {onViewChange && (
            <>
              <ViewToggle view={view} onChange={onViewChange} />
              <div className="h-8 w-px bg-dark-border" />
            </>
          )}
          <button
            onClick={onOpenSettings}
            className="flex items-center gap-2 rounded-lg border border-dark-border px-4 py-2 text-sm font-medium text-gray-300 transition-colors hover:bg-dark-surface-elevated hover:text-white"
            title="Settings"
          >
            <SettingsIcon />
            <span className="hidden lg:inline">Settings</span>
          </button>
          <button
            onClick={onAddTorrent}
            className="flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-hover"
          >
            <PlusIcon />
            <span className="hidden sm:inline">Add Torrent</span>
          </button>
        </div>
      </div>
    </header>
  );
}

interface StatProps {
  label: string;
  value: string;
  icon: React.ReactNode;
  color: string;
}

function Stat({ label, value, icon, color }: StatProps) {
  return (
    <div className="flex items-center gap-2">
      <div className={color}>{icon}</div>
      <div>
        <div className="text-xs text-gray-500">{label}</div>
        <div className="text-sm font-semibold text-white">{value}</div>
      </div>
    </div>
  );
}

function LogoIcon() {
  return (
    <svg className="h-6 w-6 text-white" fill="currentColor" viewBox="0 0 24 24">
      <path d="M12 2L2 7v10c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V7l-10-5zm0 18c-3.31-.91-6-4.63-6-9V8.3l6-3.11v14.82z" />
    </svg>
  );
}

function DownloadIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
    </svg>
  );
}

function UploadIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
    </svg>
  );
}

function ArrowDownIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
    </svg>
  );
}

function ArrowUpIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 10l7-7m0 0l7 7m-7-7v18" />
    </svg>
  );
}

function PlusIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
    </svg>
  );
}

function SettingsIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
      />
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
    </svg>
  );
}
