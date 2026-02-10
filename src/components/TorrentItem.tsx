import { TorrentInfo, TorrentState } from "../types";
import { formatBytes, formatSpeed, formatProgress, calculateETA, cn } from "../lib/utils";
import { api } from "../lib/api";
import { useState } from "react";

interface TorrentItemProps {
  torrent: TorrentInfo;
  onUpdate: () => void;
  onShowDetails: (torrent: TorrentInfo) => void;
}

export function TorrentItem({ torrent, onUpdate, onShowDetails }: TorrentItemProps) {
  const [isDeleting, setIsDeleting] = useState(false);

  const progress = formatProgress(torrent.downloaded, torrent.size);
  const eta = calculateETA(torrent.size - torrent.downloaded, torrent.download_speed);

  const handlePlayPause = async () => {
    try {
      if (torrent.state === TorrentState.Downloading || torrent.state === TorrentState.Seeding) {
        await api.pauseTorrent(torrent.id);
      } else {
        await api.startTorrent(torrent.id);
      }
      onUpdate();
    } catch (error) {
      console.error("Failed to toggle torrent state:", error);
    }
  };

  const handleDelete = async () => {
    if (!confirm(`Delete "${torrent.name}"? Files will not be deleted.`)) {
      return;
    }

    try {
      setIsDeleting(true);
      await api.removeTorrent(torrent.id, false);
      onUpdate();
    } catch (error) {
      console.error("Failed to delete torrent:", error);
      setIsDeleting(false);
    }
  };

  const handleButtonClick = (e: React.MouseEvent, action: () => void) => {
    e.stopPropagation(); // Prevent triggering the card click
    action();
  };

  const getStateColor = () => {
    switch (torrent.state) {
      case TorrentState.Downloading:
        return "text-primary";
      case TorrentState.Seeding:
        return "text-success";
      case TorrentState.Paused:
        return "text-gray-400";
      case TorrentState.Error:
        return "text-error";
      default:
        return "text-gray-400";
    }
  };

  const getProgressColor = () => {
    if (progress >= 100) return "bg-success";
    if (torrent.state === TorrentState.Downloading) return "bg-primary";
    return "bg-gray-500";
  };

  return (
    <div
      className={cn(
        "group relative rounded-lg border bg-dark-surface p-4 transition-all hover:bg-dark-surface-elevated cursor-pointer",
        "border-dark-border",
        isDeleting && "opacity-50 pointer-events-none"
      )}
      onClick={() => onShowDetails(torrent)}
    >
      {/* Header */}
      <div className="mb-3 flex items-start justify-between">
        <div className="flex-1 min-w-0 pr-4">
          <h3 className="truncate text-base font-semibold text-white">
            {torrent.name}
          </h3>
          <div className="mt-1 flex items-center gap-3 text-sm text-gray-400">
            <span className={cn("font-medium", getStateColor())}>
              {torrent.state}
            </span>
            <span>•</span>
            <span>{formatBytes(torrent.size)}</span>
            {torrent.state === TorrentState.Downloading && (
              <>
                <span>•</span>
                <span>ETA: {eta}</span>
              </>
            )}
          </div>
        </div>

        {/* Action buttons */}
        <div className="flex items-center gap-2">
          <button
            onClick={(e) => handleButtonClick(e, handlePlayPause)}
            className="rounded-md p-2 text-gray-400 transition-colors hover:bg-dark-surface-elevated hover:text-white"
            title={torrent.state === TorrentState.Downloading ? "Pause" : "Start"}
          >
            {torrent.state === TorrentState.Downloading || torrent.state === TorrentState.Seeding ? (
              <PauseIcon />
            ) : (
              <PlayIcon />
            )}
          </button>
          <button
            onClick={(e) => handleButtonClick(e, handleDelete)}
            className="rounded-md p-2 text-gray-400 transition-colors hover:bg-dark-surface-elevated hover:text-error"
            title="Delete"
          >
            <TrashIcon />
          </button>
        </div>
      </div>

      {/* Progress bar */}
      <div className="mb-3">
        <div className="h-2 w-full overflow-hidden rounded-full bg-dark-surface-elevated">
          <div
            className={cn("h-full transition-all duration-300", getProgressColor())}
            style={{ width: `${progress}%` }}
          />
        </div>
        <div className="mt-1.5 flex items-center justify-between text-xs text-gray-400">
          <span>{progress.toFixed(1)}%</span>
          <span>
            {formatBytes(torrent.downloaded)} / {formatBytes(torrent.size)}
          </span>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 gap-4 text-sm sm:grid-cols-4">
        <div>
          <div className="text-xs text-gray-500">Download</div>
          <div className="font-medium text-white">
            {formatSpeed(torrent.download_speed)}
          </div>
        </div>
        <div>
          <div className="text-xs text-gray-500">Upload</div>
          <div className="font-medium text-white">
            {formatSpeed(torrent.upload_speed)}
          </div>
        </div>
        <div>
          <div className="text-xs text-gray-500">Peers</div>
          <div className="font-medium text-white">{torrent.peers}</div>
        </div>
        <div>
          <div className="text-xs text-gray-500">Seeds</div>
          <div className="font-medium text-white">{torrent.seeds}</div>
        </div>
      </div>
    </div>
  );
}

// Icons as inline SVGs
function PlayIcon() {
  return (
    <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
      <path d="M6.3 2.841A1.5 1.5 0 004 4.11V15.89a1.5 1.5 0 002.3 1.269l9.344-5.89a1.5 1.5 0 000-2.538L6.3 2.84z" />
    </svg>
  );
}

function PauseIcon() {
  return (
    <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
      <path d="M5.75 3a.75.75 0 00-.75.75v12.5c0 .414.336.75.75.75h1.5a.75.75 0 00.75-.75V3.75A.75.75 0 007.25 3h-1.5zM12.75 3a.75.75 0 00-.75.75v12.5c0 .414.336.75.75.75h1.5a.75.75 0 00.75-.75V3.75a.75.75 0 00-.75-.75h-1.5z" />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
    </svg>
  );
}
