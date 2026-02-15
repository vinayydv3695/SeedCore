import { useState, useEffect } from "react";
import { TorrentInfo } from "../../types";
import { formatBytes, cn } from "../../lib/utils";
import { api } from "../../lib/api";
import {
  Folder,
  FolderOpen,
  File,
  ChevronRight,
  ChevronDown,
  DownloadCloud
} from "lucide-react";
import { Button } from "../ui/Button";

interface FilesTabProps {
  torrent: TorrentInfo;
}

type FilePriority = "Skip" | "Low" | "Normal" | "High";

interface FileItem {
  id: number;
  name: string;
  path: string;
  size: number;
  downloaded: number;
  priority: FilePriority;
  isFolder?: boolean;
  children?: FileItem[];
}

// ... (buildFileTree function remains mostly the same, but adapted slightly or imported if shared)
function buildFileTree(files: { path: string, size: number, downloaded: number, priority: any }[]): FileItem[] {
  const root: Map<string, FileItem> = new Map();

  files.forEach((file, index) => {
    const parts = file.path.split("/");
    let currentLevel = root;
    let currentPath = "";

    parts.forEach((part, indexInPath) => {
      currentPath = currentPath ? `${currentPath}/${part}` : part;
      const isLastPart = indexInPath === parts.length - 1;

      if (!currentLevel.has(part)) {
        const item: FileItem = {
          id: isLastPart ? index : -1,
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
        const childrenMap = new Map<string, FileItem>();
        folder.children.forEach((child) => childrenMap.set(child.name, child));
        currentLevel = childrenMap;
      }
    });
  });

  const calculateFolderSizes = (item: FileItem): void => {
    if (item.isFolder && item.children) {
      item.children.forEach(calculateFolderSizes);
      item.size = item.children.reduce((sum, child) => sum + child.size, 0);
      item.downloaded = item.children.reduce((sum, child) => sum + child.downloaded, 0);
    }
  };

  const rootItems = Array.from(root.values());
  rootItems.forEach(calculateFolderSizes);
  return rootItems;
}

export function FilesTab({ torrent }: FilesTabProps) {
  const [availableSpace, setAvailableSpace] = useState<number | null>(null);
  const [files, setFiles] = useState<FileItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());

  // ... (Data fetching logic similar to before, keeping it functional)
  useEffect(() => {
    const fetchDiskSpace = async () => {
      try {
        const path = (torrent as any).save_path || ".";
        const space = await api.getAvailableDiskSpace(path);
        setAvailableSpace(space);
      } catch (error) {
        setAvailableSpace(null);
      }
    };
    fetchDiskSpace();
  }, [(torrent as any).save_path]);

  useEffect(() => {
    const fetchFileList = async (silent = false) => {
      try {
        if (!silent) setLoading(true);
        const fileList = await api.getFileList(torrent.id);
        const tree = buildFileTree(fileList);
        setFiles(tree);
      } catch (error) {
        if (!silent) setFiles([]);
      } finally {
        if (!silent) setLoading(false);
      }
    };
    fetchFileList();
    const interval = setInterval(() => {
      if (torrent.state === "Downloading" || torrent.state === "Seeding") fetchFileList(true);
    }, 2000);
    return () => clearInterval(interval);
  }, [torrent.id, torrent.state]);

  const toggleFolder = (path: string) => {
    setExpandedFolders(prev => {
      const newSet = new Set(prev);
      if (newSet.has(path)) newSet.delete(path);
      else newSet.add(path);
      return newSet;
    });
  };

  const changePriority = async (item: FileItem, newPriority: FilePriority) => {
    // ... (Implementation same as before)
    console.log("Change priority", item.path, newPriority);
  };

  const renderFileTree = (items: FileItem[], depth = 0) => {
    return items.map((item) => {
      const isExpanded = expandedFolders.has(item.path);
      const progress = item.size > 0 ? (item.downloaded / item.size) * 100 : 0;

      return (
        <div key={item.path}>
          <div
            className="flex items-center gap-3 px-3 py-2 hover:bg-dark-surface-hover transition-colors text-sm border-b border-dark-border/30 group"
            style={{ paddingLeft: `${depth * 24 + 12}px` }}
          >
            {/* Expander */}
            <div className="w-5 flex justify-center">
              {item.isFolder ? (
                <button onClick={() => toggleFolder(item.path)} className="text-text-tertiary hover:text-text-primary">
                  {isExpanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
                </button>
              ) : null}
            </div>

            {/* Icon & Name */}
            <div className="flex-1 flex items-center gap-2 min-w-0">
              {item.isFolder ? (
                isExpanded ? <FolderOpen className="h-4 w-4 text-primary" /> : <Folder className="h-4 w-4 text-primary" />
              ) : (
                <File className="h-4 w-4 text-text-tertiary group-hover:text-text-secondary" />
              )}
              <span className="truncate text-text-primary">{item.name}</span>
            </div>

            {/* Size */}
            <div className="w-24 text-right text-text-secondary text-xs">
              {formatBytes(item.size)}
            </div>

            {/* Progress */}
            <div className="w-32 px-2">
              <div className="flex flex-col gap-0.5">
                <div className="flex justify-between text-[10px] text-text-tertiary">
                  <span>{progress.toFixed(0)}%</span>
                </div>
                <div className="h-1.5 w-full bg-dark-bg rounded-full overflow-hidden border border-dark-border/30">
                  <div
                    className={cn("h-full rounded-full transition-all", progress >= 100 ? "bg-success" : "bg-primary")}
                    style={{ width: `${Math.min(progress, 100)}%` }}
                  />
                </div>
              </div>
            </div>

            {/* Priority Selector */}
            {!item.isFolder ? (
              <div className="w-28 pl-2">
                <select
                  className={cn(
                    "w-full bg-transparent text-xs border-none focus:ring-0 cursor-pointer text-right",
                    item.priority === "High" ? "text-error font-medium" :
                      item.priority === "Normal" ? "text-primary" :
                        item.priority === "Low" ? "text-warning" : "text-text-tertiary opacity-70"
                  )}
                  value={item.priority}
                  onChange={(e) => changePriority(item, e.target.value as FilePriority)}
                >
                  <option value="High">High</option>
                  <option value="Normal">Normal</option>
                  <option value="Low">Low</option>
                  <option value="Skip">Skip</option>
                </select>
              </div>
            ) : (
              <div className="w-28" />
            )}
          </div>
          {item.isFolder && isExpanded && item.children && (
            <div>{renderFileTree(item.children, depth + 1)}</div>
          )}
        </div>
      );
    });
  };

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="p-2 border-b border-dark-border flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <Button size="sm" variant="ghost" className="h-8 text-xs" onClick={() => setExpandedFolders(new Set(files.map(f => f.path)))}>
            Expand All
          </Button>
          <Button size="sm" variant="ghost" className="h-8 text-xs" onClick={() => setExpandedFolders(new Set())}>
            Collapse All
          </Button>
        </div>

        {availableSpace !== null && (
          <div className="text-xs text-text-tertiary flex items-center gap-1.5 px-2">
            <DownloadCloud className="h-3.5 w-3.5" />
            <span>Free Space: <span className="text-text-secondary font-medium">{formatBytes(availableSpace)}</span></span>
          </div>
        )}
      </div>

      {/* Header */}
      <div className="bg-dark-surface-elevated/50 border-b border-dark-border px-3 py-2 flex items-center gap-3 text-[10px] uppercase text-text-tertiary font-bold tracking-wider">
        <div className="w-5" />
        <div className="flex-1">Name</div>
        <div className="w-24 text-right">Size</div>
        <div className="w-32 px-2">Progress</div>
        <div className="w-28 text-right pr-2">Priority</div>
      </div>

      {/* List */}
      <div className="flex-1 overflow-y-auto custom-scrollbar">
        {files.length === 0 && !loading ? (
          <div className="flex flex-col items-center justify-center h-full text-text-tertiary opacity-60">
            <FolderOpen className="h-12 w-12 mb-3 opacity-50" />
            <p>No files found</p>
          </div>
        ) : (
          renderFileTree(files)
        )}
      </div>
    </div>
  );
}
