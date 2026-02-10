import { useState, useMemo } from "react";
import { TorrentInfo, TorrentState, DownloadSource } from "../types";
import { formatBytes, formatSpeed, calculateETA } from "../lib/utils";
import { PlayIcon, PauseIcon, InfoIcon, TrashIcon } from "./Icons";

interface TorrentTableProps {
  torrents: TorrentInfo[];
  selectedIds: Set<string>;
  onSelect: (id: string, multi: boolean) => void;
  onShowDetails: (torrent: TorrentInfo) => void;
  onStart: (id: string) => void;
  onPause: (id: string) => void;
  onRemove: (id: string) => void;
}

type SortColumn =
  | "name"
  | "size"
  | "progress"
  | "state"
  | "source"
  | "download_speed"
  | "upload_speed"
  | "eta"
  | "ratio"
  | "peers";
type SortDirection = "asc" | "desc";

export function TorrentTable({
  torrents,
  selectedIds,
  onSelect,
  onShowDetails,
  onStart,
  onPause,
  onRemove,
}: TorrentTableProps) {
  const [sortColumn, setSortColumn] = useState<SortColumn>("name");
  const [sortDirection, setSortDirection] = useState<SortDirection>("asc");
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    torrentId: string;
  } | null>(null);

  // Sort torrents
  const sortedTorrents = useMemo(() => {
    const sorted = [...torrents].sort((a, b) => {
      let comparison = 0;

      switch (sortColumn) {
        case "name":
          comparison = a.name.localeCompare(b.name);
          break;
        case "size":
          comparison = a.size - b.size;
          break;
        case "progress":
          const progressA = a.size > 0 ? a.downloaded / a.size : 0;
          const progressB = b.size > 0 ? b.downloaded / b.size : 0;
          comparison = progressA - progressB;
          break;
        case "state":
          comparison = a.state.localeCompare(b.state);
          break;
        case "source":
          comparison = a.source.localeCompare(b.source);
          break;
        case "download_speed":
          comparison = a.download_speed - b.download_speed;
          break;
        case "upload_speed":
          comparison = a.upload_speed - b.upload_speed;
          break;
        case "eta": {
          const etaA = calculateETA(a.size - a.downloaded, a.download_speed);
          const etaB = calculateETA(b.size - b.downloaded, b.download_speed);
          comparison =
            (etaA === "∞" ? Infinity : parseInt(etaA)) -
            (etaB === "∞" ? Infinity : parseInt(etaB));
          break;
        }
        case "ratio": {
          const ratioA = a.downloaded > 0 ? a.uploaded / a.downloaded : 0;
          const ratioB = b.downloaded > 0 ? b.uploaded / a.downloaded : 0;
          comparison = ratioA - ratioB;
          break;
        }
        case "peers":
          comparison = a.peers - b.peers;
          break;
      }

      return sortDirection === "asc" ? comparison : -comparison;
    });

    return sorted;
  }, [torrents, sortColumn, sortDirection]);

  const handleSort = (column: SortColumn) => {
    if (sortColumn === column) {
      setSortDirection(sortDirection === "asc" ? "desc" : "asc");
    } else {
      setSortColumn(column);
      setSortDirection("asc");
    }
  };

  const handleRowClick = (torrent: TorrentInfo, e: React.MouseEvent) => {
    if (e.ctrlKey || e.metaKey) {
      onSelect(torrent.id, true);
    } else if (e.shiftKey) {
      // TODO: Implement shift-select range
      onSelect(torrent.id, true);
    } else {
      onSelect(torrent.id, false);
    }
  };

  const handleDoubleClick = (torrent: TorrentInfo) => {
    onShowDetails(torrent);
  };

  const handleContextMenu = (e: React.MouseEvent, torrentId: string) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, torrentId });
  };

  const handleCloseContextMenu = () => {
    setContextMenu(null);
  };

  const getStateBadgeClasses = (state: TorrentState) => {
    switch (state) {
      case TorrentState.Downloading:
        return "bg-primary/20 text-primary border-primary/30";
      case TorrentState.Seeding:
        return "bg-success/20 text-success border-success/30";
      case TorrentState.Paused:
        return "bg-gray-500/20 text-gray-400 border-gray-500/30";
      case TorrentState.Checking:
        return "bg-warning/20 text-warning border-warning/30";
      case TorrentState.Error:
        return "bg-error/20 text-error border-error/30";
      case TorrentState.Queued:
        return "bg-blue-500/20 text-blue-400 border-blue-500/30";
      default:
        return "bg-gray-500/20 text-gray-400 border-gray-500/30";
    }
  };

  const getProgressColor = (progress: number) => {
    if (progress >= 100) return "bg-success";
    if (progress >= 50) return "bg-primary";
    return "bg-warning";
  };

  const getSourceBadgeClasses = (source: DownloadSource) => {
    switch (source) {
      case DownloadSource.Cloud:
        return "bg-blue-500/20 text-blue-400 border-blue-500/30";
      case DownloadSource.P2P:
        return "bg-purple-500/20 text-purple-400 border-purple-500/30";
      case DownloadSource.Hybrid:
        return "bg-cyan-500/20 text-cyan-400 border-cyan-500/30";
      default:
        return "bg-gray-500/20 text-gray-400 border-gray-500/30";
    }
  };

  const getSourceIcon = (source: DownloadSource) => {
    switch (source) {
      case DownloadSource.Cloud:
        return (
          <svg
            className="w-3 h-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z"
            />
          </svg>
        );
      case DownloadSource.P2P:
        return (
          <svg
            className="w-3 h-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
            />
          </svg>
        );
      case DownloadSource.Hybrid:
        return (
          <svg
            className="w-3 h-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M13 10V3L4 14h7v7l9-11h-7z"
            />
          </svg>
        );
      default:
        return null;
    }
  };

  const SortIcon = ({ column }: { column: SortColumn }) => {
    if (sortColumn !== column) {
      return <span className="text-gray-600">↕</span>;
    }
    return sortDirection === "asc" ? (
      <span className="text-primary">↑</span>
    ) : (
      <span className="text-primary">↓</span>
    );
  };

  return (
    <div className="flex flex-col h-full bg-dark-tertiary rounded-lg overflow-hidden">
      {/* Table header */}
      <div className="bg-dark-secondary border-b border-dark-border">
        <table className="w-full">
          <thead>
            <tr className="text-xs text-gray-400 uppercase">
              <th className="w-8 p-2">
                <input type="checkbox" className="rounded" />
              </th>
              <th
                className="text-left p-2 cursor-pointer hover:bg-dark-elevated transition-colors"
                onClick={() => handleSort("name")}
              >
                <div className="flex items-center gap-1">
                  <span>Name</span>
                  <SortIcon column="name" />
                </div>
              </th>
              <th
                className="text-right p-2 cursor-pointer hover:bg-dark-elevated transition-colors w-24"
                onClick={() => handleSort("size")}
              >
                <div className="flex items-center justify-end gap-1">
                  <span>Size</span>
                  <SortIcon column="size" />
                </div>
              </th>
              <th
                className="text-right p-2 cursor-pointer hover:bg-dark-elevated transition-colors w-32"
                onClick={() => handleSort("progress")}
              >
                <div className="flex items-center justify-end gap-1">
                  <span>Progress</span>
                  <SortIcon column="progress" />
                </div>
              </th>
              <th
                className="text-center p-2 cursor-pointer hover:bg-dark-elevated transition-colors w-24"
                onClick={() => handleSort("state")}
              >
                <div className="flex items-center justify-center gap-1">
                  <span>Status</span>
                  <SortIcon column="state" />
                </div>
              </th>
              <th
                className="text-center p-2 cursor-pointer hover:bg-dark-elevated transition-colors w-24"
                onClick={() => handleSort("source")}
              >
                <div className="flex items-center justify-center gap-1">
                  <span>Source</span>
                  <SortIcon column="source" />
                </div>
              </th>
              <th
                className="text-right p-2 cursor-pointer hover:bg-dark-elevated transition-colors w-28"
                onClick={() => handleSort("download_speed")}
              >
                <div className="flex items-center justify-end gap-1">
                  <span>Down</span>
                  <SortIcon column="download_speed" />
                </div>
              </th>
              <th
                className="text-right p-2 cursor-pointer hover:bg-dark-elevated transition-colors w-28"
                onClick={() => handleSort("upload_speed")}
              >
                <div className="flex items-center justify-end gap-1">
                  <span>Up</span>
                  <SortIcon column="upload_speed" />
                </div>
              </th>
              <th
                className="text-right p-2 cursor-pointer hover:bg-dark-elevated transition-colors w-24"
                onClick={() => handleSort("eta")}
              >
                <div className="flex items-center justify-end gap-1">
                  <span>ETA</span>
                  <SortIcon column="eta" />
                </div>
              </th>
              <th
                className="text-right p-2 cursor-pointer hover:bg-dark-elevated transition-colors w-20"
                onClick={() => handleSort("ratio")}
              >
                <div className="flex items-center justify-end gap-1">
                  <span>Ratio</span>
                  <SortIcon column="ratio" />
                </div>
              </th>
              <th
                className="text-center p-2 cursor-pointer hover:bg-dark-elevated transition-colors w-20"
                onClick={() => handleSort("peers")}
              >
                <div className="flex items-center justify-center gap-1">
                  <span>Peers</span>
                  <SortIcon column="peers" />
                </div>
              </th>
            </tr>
          </thead>
        </table>
      </div>

      {/* Table body */}
      <div className="flex-1 overflow-y-auto custom-scrollbar">
        {sortedTorrents.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <svg
              className="w-24 h-24 mb-6 text-gray-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
              />
            </svg>
            <h3 className="text-xl font-semibold text-white mb-2">
              No torrents yet
            </h3>
            <p className="text-sm text-gray-500 mb-6">
              Add a torrent file or magnet link to get started
            </p>
            <div className="flex gap-3 text-xs text-gray-600">
              <div className="flex items-center gap-2">
                <kbd className="px-2 py-1 bg-dark-elevated rounded border border-dark-border font-mono">
                  Ctrl
                </kbd>
                <span>+</span>
                <kbd className="px-2 py-1 bg-dark-elevated rounded border border-dark-border font-mono">
                  N
                </kbd>
                <span>to add torrent</span>
              </div>
              <div className="flex items-center gap-2">
                <kbd className="px-2 py-1 bg-dark-elevated rounded border border-dark-border font-mono">
                  Ctrl
                </kbd>
                <span>+</span>
                <kbd className="px-2 py-1 bg-dark-elevated rounded border border-dark-border font-mono">
                  V
                </kbd>
                <span>to paste magnet</span>
              </div>
            </div>
          </div>
        ) : (
          <table className="w-full">
            <tbody>
              {sortedTorrents.map((torrent) => {
                const isSelected = selectedIds.has(torrent.id);
                const progress =
                  torrent.size > 0
                    ? (torrent.downloaded / torrent.size) * 100
                    : 0;
                const ratio =
                  torrent.downloaded > 0
                    ? torrent.uploaded / torrent.downloaded
                    : 0;
                const eta = calculateETA(
                  torrent.size - torrent.downloaded,
                  torrent.download_speed,
                );

                return (
                  <tr
                    key={torrent.id}
                    className={`
                      border-b border-dark-border text-sm cursor-pointer
                      transition-colors duration-150
                      ${isSelected ? "bg-primary/20" : "hover:bg-dark-elevated"}
                    `}
                    onClick={(e) => handleRowClick(torrent, e)}
                    onDoubleClick={() => handleDoubleClick(torrent)}
                    onContextMenu={(e) => handleContextMenu(e, torrent.id)}
                  >
                    <td className="p-2 w-8">
                      <input
                        type="checkbox"
                        checked={isSelected}
                        onChange={() => onSelect(torrent.id, true)}
                        className="rounded"
                        onClick={(e) => e.stopPropagation()}
                      />
                    </td>
                    <td
                      className="p-2 text-white truncate max-w-xs group relative"
                      title={torrent.name}
                    >
                      <span className="truncate">{torrent.name}</span>
                      {/* Tooltip on hover */}
                      <div className="absolute left-0 top-full mt-1 px-3 py-2 bg-dark-elevated border border-dark-border rounded-lg shadow-xl z-50 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 pointer-events-none whitespace-normal max-w-md">
                        <p className="text-sm font-medium">{torrent.name}</p>
                      </div>
                    </td>
                    <td className="p-2 text-right text-gray-300 w-24">
                      {formatBytes(torrent.size)}
                    </td>
                    <td className="p-2 w-32">
                      <div className="flex flex-col gap-1">
                        <div className="w-full bg-dark-border rounded-full h-2 overflow-hidden relative">
                          <div
                            className={`h-full rounded-full transition-all duration-500 ease-out ${getProgressColor(
                              progress,
                            )}`}
                            style={{ width: `${Math.min(progress, 100)}%` }}
                          >
                            {/* Animated shine effect */}
                            {progress < 100 && progress > 0 && (
                              <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer" />
                            )}
                          </div>
                          {/* Percentage text overlay */}
                          <div className="absolute inset-0 flex items-center justify-center">
                            <span className="text-[10px] font-semibold text-white drop-shadow-[0_1px_2px_rgba(0,0,0,0.8)]">
                              {progress.toFixed(1)}%
                            </span>
                          </div>
                        </div>
                      </div>
                    </td>
                    <td className="p-2 text-center w-24">
                      <span
                        className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${getStateBadgeClasses(
                          torrent.state,
                        )}`}
                      >
                        {torrent.state}
                      </span>
                    </td>
                    <td className="p-2 text-center w-24">
                      <span
                        className={`inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-medium border ${getSourceBadgeClasses(
                          torrent.source,
                        )}`}
                        title={`Download source: ${torrent.source}`}
                      >
                        {getSourceIcon(torrent.source)}
                        {torrent.source}
                      </span>
                    </td>
                    <td className="p-2 text-right text-primary w-28">
                      {torrent.download_speed > 0
                        ? formatSpeed(torrent.download_speed)
                        : "-"}
                    </td>
                    <td className="p-2 text-right text-success w-28">
                      {torrent.upload_speed > 0
                        ? formatSpeed(torrent.upload_speed)
                        : "-"}
                    </td>
                    <td className="p-2 text-right text-gray-300 w-24">{eta}</td>
                    <td className="p-2 text-right text-gray-300 w-20">
                      {ratio.toFixed(2)}
                    </td>
                    <td className="p-2 text-center text-gray-300 w-20">
                      {torrent.peers}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </div>

      {/* Context Menu */}
      {contextMenu && (
        <>
          <div
            className="fixed inset-0 z-40"
            onClick={handleCloseContextMenu}
          />
          <div
            className="fixed z-50 bg-dark-secondary border border-dark-border rounded-lg shadow-xl py-1 min-w-[180px]"
            style={{ left: contextMenu.x, top: contextMenu.y }}
          >
            <button
              onClick={() => {
                onStart(contextMenu.torrentId);
                handleCloseContextMenu();
              }}
              className="w-full px-4 py-2.5 text-left text-sm text-white hover:bg-dark-elevated transition-colors flex items-center gap-3"
            >
              <PlayIcon size={16} className="text-success" />
              <span>Start</span>
            </button>
            <button
              onClick={() => {
                onPause(contextMenu.torrentId);
                handleCloseContextMenu();
              }}
              className="w-full px-4 py-2.5 text-left text-sm text-white hover:bg-dark-elevated transition-colors flex items-center gap-3"
            >
              <PauseIcon size={16} className="text-warning" />
              <span>Pause</span>
            </button>
            <div className="border-t border-dark-border my-1" />
            <button
              onClick={() => {
                const torrent = torrents.find(
                  (t) => t.id === contextMenu.torrentId,
                );
                if (torrent) onShowDetails(torrent);
                handleCloseContextMenu();
              }}
              className="w-full px-4 py-2.5 text-left text-sm text-white hover:bg-dark-elevated transition-colors flex items-center gap-3"
            >
              <InfoIcon size={16} className="text-primary" />
              <span>Details</span>
            </button>
            <div className="border-t border-dark-border my-1" />
            <button
              onClick={() => {
                onRemove(contextMenu.torrentId);
                handleCloseContextMenu();
              }}
              className="w-full px-4 py-2.5 text-left text-sm text-error hover:bg-dark-elevated transition-colors flex items-center gap-3"
            >
              <TrashIcon size={16} className="text-error" />
              <span>Remove</span>
            </button>
          </div>
        </>
      )}
    </div>
  );
}
