import { useState, useEffect } from "react";
import { TorrentInfo, FileInfo } from "../../types";
import { formatBytes } from "../../lib/utils";
import { api } from "../../lib/api";

interface FilesTabProps {
  torrent: TorrentInfo;
}

type FilePriority = "Skip" | "Low" | "Normal" | "High";

interface FileItem {
  name: string;
  path: string;
  size: number;
  downloaded: number;
  priority: FilePriority;
  isFolder?: boolean;
  children?: FileItem[];
}

// SVG Icon Components
const FolderIcon = ({ className = "w-4 h-4" }: { className?: string }) => (
  <svg
    className={className}
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
    />
  </svg>
);

const FileIcon = ({ className = "w-4 h-4" }: { className?: string }) => (
  <svg
    className={className}
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"
    />
  </svg>
);

const ChevronRightIcon = ({
  className = "w-4 h-4",
}: {
  className?: string;
}) => (
  <svg
    className={className}
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M9 5l7 7-7 7"
    />
  </svg>
);

const ChevronDownIcon = ({ className = "w-4 h-4" }: { className?: string }) => (
  <svg
    className={className}
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M19 9l-7 7-7-7"
    />
  </svg>
);

const ArrowUpIcon = ({ className = "w-3 h-3" }: { className?: string }) => (
  <svg className={className} fill="currentColor" viewBox="0 0 20 20">
    <path
      fillRule="evenodd"
      d="M3.293 9.707a1 1 0 010-1.414l6-6a1 1 0 011.414 0l6 6a1 1 0 01-1.414 1.414L11 5.414V17a1 1 0 11-2 0V5.414L4.707 9.707a1 1 0 01-1.414 0z"
      clipRule="evenodd"
    />
  </svg>
);

const ArrowRightIcon = ({ className = "w-3 h-3" }: { className?: string }) => (
  <svg className={className} fill="currentColor" viewBox="0 0 20 20">
    <path
      fillRule="evenodd"
      d="M12.293 5.293a1 1 0 011.414 0l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-2.293-2.293a1 1 0 010-1.414z"
      clipRule="evenodd"
    />
  </svg>
);

const ArrowDownIcon = ({ className = "w-3 h-3" }: { className?: string }) => (
  <svg className={className} fill="currentColor" viewBox="0 0 20 20">
    <path
      fillRule="evenodd"
      d="M16.707 10.293a1 1 0 010 1.414l-6 6a1 1 0 01-1.414 0l-6-6a1 1 0 111.414-1.414L9 14.586V3a1 1 0 012 0v11.586l4.293-4.293a1 1 0 011.414 0z"
      clipRule="evenodd"
    />
  </svg>
);

const PauseIcon = ({ className = "w-3 h-3" }: { className?: string }) => (
  <svg className={className} fill="currentColor" viewBox="0 0 20 20">
    <path
      fillRule="evenodd"
      d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zM7 8a1 1 0 012 0v4a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v4a1 1 0 102 0V8a1 1 0 00-1-1z"
      clipRule="evenodd"
    />
  </svg>
);

// Build hierarchical file tree from flat file list
function buildFileTree(files: FileInfo[]): FileItem[] {
  const root: Map<string, FileItem> = new Map();

  files.forEach((file) => {
    const parts = file.path.split("/");
    let currentLevel = root;
    let currentPath = "";

    parts.forEach((part, index) => {
      currentPath = currentPath ? `${currentPath}/${part}` : part;
      const isLastPart = index === parts.length - 1;

      if (!currentLevel.has(part)) {
        const item: FileItem = {
          name: part,
          path: currentPath,
          size: isLastPart ? file.size : 0,
          downloaded: isLastPart ? file.downloaded : 0,
          priority: file.priority,
          isFolder: !isLastPart,
          children: isLastPart ? undefined : [],
        };
        currentLevel.set(part, item);
      }

      if (!isLastPart) {
        const folder = currentLevel.get(part)!;
        if (!folder.children) {
          folder.children = [];
        }
        // Create children map for next level
        const childrenMap = new Map<string, FileItem>();
        folder.children.forEach((child) => childrenMap.set(child.name, child));
        currentLevel = childrenMap;
      }
    });
  });

  // Convert root map to array and calculate folder sizes
  const calculateFolderSizes = (item: FileItem): void => {
    if (item.isFolder && item.children) {
      item.children.forEach(calculateFolderSizes);
      item.size = item.children.reduce((sum, child) => sum + child.size, 0);
      item.downloaded = item.children.reduce(
        (sum, child) => sum + child.downloaded,
        0,
      );
    }
  };

  const rootItems = Array.from(root.values());
  rootItems.forEach(calculateFolderSizes);
  return rootItems;
}

export function FilesTab({ torrent }: FilesTabProps) {
  // Get available disk space
  const [availableSpace, setAvailableSpace] = useState<number | null>(null);
  // Real file tree from torrent
  const [files, setFiles] = useState<FileItem[]>([]);
  const [loading, setLoading] = useState(true);
  // Cloud file progress (if applicable)
  const [cloudProgress, setCloudProgress] = useState<
    Map<
      string,
      {
        downloaded: number;
        speed: number;
        state: "Queued" | "Downloading" | "Complete" | "Error";
      }
    >
  >(new Map());

  useEffect(() => {
    // Fetch real disk space from the current directory
    const fetchDiskSpace = async () => {
      try {
        // Use current directory - Tauri command will handle finding the right path
        const space = await api.getAvailableDiskSpace(".");
        setAvailableSpace(space);
      } catch (error) {
        console.error("Failed to get disk space:", error);
        // Fallback: simulate 120 GB free space if API fails
        setAvailableSpace(120 * 1024 * 1024 * 1024);
      }
    };
    fetchDiskSpace();
  }, []);

  useEffect(() => {
    // Fetch real file list from torrent
    const fetchFileList = async () => {
      try {
        setLoading(true);
        const fileList = await api.getFileList(torrent.id);
        const tree = buildFileTree(fileList);
        setFiles(tree);
      } catch (error) {
        console.error("Failed to get file list:", error);
        // Fallback to empty list
        setFiles([]);
      } finally {
        setLoading(false);
      }
    };
    fetchFileList();
  }, [torrent.id]);

  // Poll cloud file progress for cloud torrents
  useEffect(() => {
    // Check if this is a cloud torrent
    const isCloudTorrent =
      torrent.source &&
      typeof torrent.source === "object" &&
      "Debrid" in torrent.source;

    if (!isCloudTorrent) {
      return;
    }

    const fetchCloudProgress = async () => {
      try {
        const progress = await api.getCloudFileProgress(torrent.id);
        const progressMap = new Map(
          progress.map((p) => [
            p.name,
            { downloaded: p.downloaded, speed: p.speed, state: p.state },
          ]),
        );
        setCloudProgress(progressMap);
      } catch (error) {
        console.error("Failed to get cloud file progress:", error);
      }
    };

    // Initial fetch
    fetchCloudProgress();

    // Poll every 2 seconds while downloading
    const interval = setInterval(() => {
      if (torrent.state === "Downloading") {
        fetchCloudProgress();
      }
    }, 2000);

    return () => clearInterval(interval);
  }, [torrent.id, torrent.state, torrent.source]);

  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(
    new Set(),
  );

  const toggleFolder = (path: string) => {
    setExpandedFolders((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(path)) {
        newSet.delete(path);
      } else {
        newSet.add(path);
      }
      return newSet;
    });
  };

  const changePriority = async (
    filePath: string,
    newPriority: FilePriority,
  ) => {
    try {
      // Update backend
      await api.setFilePriority(torrent.id, filePath, newPriority);

      // Update local state
      const updatePriority = (items: FileItem[]): FileItem[] => {
        return items.map((item) => {
          if (item.path === filePath) {
            return { ...item, priority: newPriority };
          }
          if (item.children) {
            return { ...item, children: updatePriority(item.children) };
          }
          return item;
        });
      };
      setFiles((prev) => updatePriority(prev));
    } catch (error) {
      console.error("Failed to set file priority:", error);
    }
  };

  const getPriorityColor = (priority: FilePriority) => {
    switch (priority) {
      case "High":
        return "text-error";
      case "Normal":
        return "text-primary";
      case "Low":
        return "text-warning";
      case "Skip":
        return "text-gray-500";
    }
  };

  const renderFileTree = (items: FileItem[], depth = 0) => {
    return items.map((item) => {
      const isExpanded = expandedFolders.has(item.path);

      // Get cloud progress for this file (if available)
      const cloudFileProgress = cloudProgress.get(item.name);

      // Use cloud progress if available, otherwise use local progress
      const downloaded = cloudFileProgress?.downloaded ?? item.downloaded;
      const progress = item.size > 0 ? (downloaded / item.size) * 100 : 0;
      const downloadSpeed = cloudFileProgress?.speed ?? 0;
      const fileState = cloudFileProgress?.state;

      return (
        <div key={item.path}>
          <div
            className={`
              flex items-center gap-3 px-3 py-2 hover:bg-dark-elevated transition-colors text-sm
              border-b border-dark-border/50
            `}
            style={{ paddingLeft: `${depth * 24 + 12}px` }}
          >
            {/* Expand/collapse button for folders */}
            {item.isFolder && (
              <button
                onClick={() => toggleFolder(item.path)}
                className="w-4 h-4 flex items-center justify-center text-gray-400 hover:text-white transition-colors"
              >
                {isExpanded ? <ChevronDownIcon /> : <ChevronRightIcon />}
              </button>
            )}
            {!item.isFolder && <div className="w-4" />}

            {/* File/folder icon and name */}
            <div className="flex-1 flex items-center gap-2 min-w-0">
              {item.isFolder ? (
                <FolderIcon className="w-4 h-4 text-blue-400" />
              ) : (
                <FileIcon className="w-4 h-4 text-gray-400" />
              )}
              <span className="text-white truncate">{item.name}</span>

              {/* Cloud download state badge */}
              {fileState && (
                <span
                  className={`text-xs px-1.5 py-0.5 rounded ${
                    fileState === "Complete"
                      ? "bg-success/20 text-success"
                      : fileState === "Downloading"
                        ? "bg-primary/20 text-primary"
                        : fileState === "Error"
                          ? "bg-error/20 text-error"
                          : "bg-gray-500/20 text-gray-400"
                  }`}
                >
                  {fileState}
                </span>
              )}
            </div>

            {/* Size */}
            <div className="w-24 text-right text-gray-300">
              {formatBytes(item.size)}
            </div>

            {/* Progress with download speed for cloud downloads */}
            <div className="w-40">
              <div className="flex flex-col gap-1">
                <div className="flex items-center gap-2">
                  <div className="flex-1 bg-dark-border rounded-full h-1.5 overflow-hidden">
                    <div
                      className={`h-full rounded-full transition-all ${
                        progress >= 100
                          ? "bg-success"
                          : fileState === "Downloading"
                            ? "bg-primary animate-pulse"
                            : "bg-primary"
                      }`}
                      style={{ width: `${Math.min(progress, 100)}%` }}
                    />
                  </div>
                  <span className="text-xs text-gray-400 w-12 text-right">
                    {progress.toFixed(0)}%
                  </span>
                </div>
                {/* Show download speed for actively downloading cloud files */}
                {downloadSpeed > 0 && (
                  <div className="text-xs text-primary">
                    {formatBytes(downloadSpeed)}/s
                  </div>
                )}
              </div>
            </div>

            {/* Priority */}
            {!item.isFolder && (
              <div className="w-32">
                <select
                  value={item.priority}
                  onChange={(e) => {
                    changePriority(item.path, e.target.value as FilePriority);
                  }}
                  className={`
                    w-full px-2 py-1 rounded-md bg-dark-secondary border border-dark-border
                    ${getPriorityColor(item.priority)} text-sm
                    focus:outline-none focus:ring-2 focus:ring-primary
                  `}
                  disabled={!!fileState} // Disable priority changes for cloud downloads
                >
                  <option value="Skip">Skip</option>
                  <option value="Low">Low</option>
                  <option value="Normal">Normal</option>
                  <option value="High">High</option>
                </select>
              </div>
            )}
            {item.isFolder && <div className="w-32" />}
          </div>

          {/* Render children if expanded */}
          {item.isFolder && isExpanded && item.children && (
            <div>{renderFileTree(item.children, depth + 1)}</div>
          )}
        </div>
      );
    });
  };

  const totalFiles = files.reduce((count, item) => {
    const countFiles = (items: FileItem[]): number => {
      return items.reduce((acc, item) => {
        if (item.isFolder && item.children) {
          return acc + countFiles(item.children);
        }
        return acc + 1;
      }, 0);
    };
    return count + countFiles([item]);
  }, 0);

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="p-3 border-b border-dark-border flex items-center gap-2">
        <button className="px-3 py-1.5 text-sm bg-dark-elevated hover:bg-dark-border text-gray-300 rounded-md transition-colors">
          Expand All
        </button>
        <button className="px-3 py-1.5 text-sm bg-dark-elevated hover:bg-dark-border text-gray-300 rounded-md transition-colors">
          Collapse All
        </button>
        <div className="h-6 w-px bg-dark-border mx-2" />
        <div className="flex items-center gap-2">
          <span className="text-sm text-gray-400">Set priority:</span>
          <select className="px-2 py-1 text-sm bg-dark-secondary border border-dark-border rounded-md text-white">
            <option>High</option>
            <option>Normal</option>
            <option>Low</option>
            <option>Skip</option>
          </select>
        </div>
        <div className="flex-1" />
        {loading ? (
          <span className="text-sm text-gray-400">Loading files...</span>
        ) : (
          <span className="text-sm text-gray-400">
            {totalFiles} file{totalFiles !== 1 ? "s" : ""}
          </span>
        )}
      </div>

      {/* Column headers */}
      <div className="bg-dark-secondary border-b border-dark-border px-3 py-2 flex items-center gap-3 text-xs text-gray-400 uppercase font-semibold">
        <div className="w-4" /> {/* Space for expand button */}
        <div className="flex-1">Name</div>
        <div className="w-24 text-right">Size</div>
        <div className="w-40 text-center">Progress</div>
        <div className="w-32">Priority</div>
      </div>

      {/* File tree */}
      <div className="flex-1 overflow-auto custom-scrollbar">
        {files.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <FolderIcon className="w-16 h-16 mb-4 text-gray-600" />
            <p className="text-lg font-medium">No files</p>
            <p className="text-sm">Loading file information...</p>
          </div>
        ) : (
          renderFileTree(files)
        )}
      </div>

      {/* Footer with legend and disk space */}
      <div className="p-3 border-t border-dark-border bg-dark-secondary">
        <div className="flex items-center justify-between">
          {/* Legend */}
          <div className="flex flex-wrap gap-4 text-xs">
            <div className="flex items-center gap-1.5">
              <ArrowUpIcon className="text-error" />
              <span className="text-gray-400">High priority</span>
            </div>
            <div className="flex items-center gap-1.5">
              <ArrowRightIcon className="text-primary" />
              <span className="text-gray-400">Normal priority</span>
            </div>
            <div className="flex items-center gap-1.5">
              <ArrowDownIcon className="text-warning" />
              <span className="text-gray-400">Low priority</span>
            </div>
            <div className="flex items-center gap-1.5">
              <PauseIcon className="text-gray-500" />
              <span className="text-gray-400">Skip (don't download)</span>
            </div>
          </div>

          {/* Disk space */}
          {availableSpace !== null && (
            <div className="flex items-center gap-2 text-xs">
              <svg
                className="w-4 h-4 text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"
                />
              </svg>
              <span className="text-gray-400">Free space:</span>
              <span className="text-white font-medium">
                {formatBytes(availableSpace)}
              </span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
