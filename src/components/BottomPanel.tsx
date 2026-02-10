import { useState, useRef, useEffect } from "react";
import { TorrentInfo } from "../types";
import { GeneralTab } from "./tabs/GeneralTab";
import { TrackersTab } from "./tabs/TrackersTab";
import { PeersTab } from "./tabs/PeersTab";
import { PiecesTab } from "./tabs/PiecesTab";
import { FilesTab } from "./tabs/FilesTab";

interface BottomPanelProps {
  torrent: TorrentInfo | null;
  isOpen: boolean;
  onClose: () => void;
}

type TabId = "general" | "trackers" | "peers" | "pieces" | "files";

interface Tab {
  id: TabId;
  label: string;
  icon: React.ReactNode;
}

// Tab icons as SVG components
const InfoIcon = () => (
  <svg
    className="w-4 h-4"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
    />
  </svg>
);

const GlobeIcon = () => (
  <svg
    className="w-4 h-4"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"
    />
  </svg>
);

const UsersIcon = () => (
  <svg
    className="w-4 h-4"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z"
    />
  </svg>
);

const PuzzleIcon = () => (
  <svg
    className="w-4 h-4"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z"
    />
  </svg>
);

const FolderIcon = () => (
  <svg
    className="w-4 h-4"
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

export function BottomPanel({ torrent, isOpen, onClose }: BottomPanelProps) {
  const [activeTab, setActiveTab] = useState<TabId>("general");
  const [panelHeight, setPanelHeight] = useState(280); // Default height in pixels
  const [isResizing, setIsResizing] = useState(false);
  const [isMinimized, setIsMinimized] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);
  const startY = useRef(0);
  const startHeight = useRef(0);

  const tabs: Tab[] = [
    { id: "general", label: "General", icon: <InfoIcon /> },
    { id: "trackers", label: "Trackers", icon: <GlobeIcon /> },
    { id: "peers", label: "Peers", icon: <UsersIcon /> },
    { id: "pieces", label: "Pieces", icon: <PuzzleIcon /> },
    { id: "files", label: "Files", icon: <FolderIcon /> },
  ];

  // Handle resize start
  const handleResizeStart = (e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
    startY.current = e.clientY;
    startHeight.current = panelHeight;
  };

  // Handle resize
  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      const deltaY = startY.current - e.clientY;
      const newHeight = Math.max(
        150,
        Math.min(600, startHeight.current + deltaY),
      );
      setPanelHeight(newHeight);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isResizing]);

  // Keyboard shortcuts for tab switching
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey) {
        switch (e.key) {
          case "1":
            setActiveTab("general");
            break;
          case "2":
            setActiveTab("trackers");
            break;
          case "3":
            setActiveTab("peers");
            break;
          case "4":
            setActiveTab("pieces");
            break;
          case "5":
            setActiveTab("files");
            break;
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen]);

  if (!isOpen || !torrent) {
    return null;
  }

  const minimizedHeight = 40;
  const currentHeight = isMinimized ? minimizedHeight : panelHeight;

  return (
    <div
      ref={panelRef}
      className="border-t border-dark-border bg-dark-tertiary flex flex-col"
      style={{ height: `${currentHeight}px` }}
    >
      {/* Resize handle */}
      <div
        className={`
          h-1 w-full cursor-row-resize hover:bg-primary transition-colors
          ${isResizing ? "bg-primary" : "bg-dark-border"}
        `}
        onMouseDown={handleResizeStart}
      />

      {/* Header with tabs */}
      <div className="flex items-center justify-between border-b border-dark-border bg-dark-secondary px-4 py-2">
        <div className="flex items-center gap-1">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`
                px-4 py-1.5 rounded-md text-sm font-medium transition-all duration-200
                flex items-center gap-2
                ${
                  activeTab === tab.id
                    ? "bg-dark-tertiary text-white shadow-sm"
                    : "text-gray-400 hover:text-white hover:bg-dark-elevated"
                }
              `}
            >
              <span>{tab.icon}</span>
              <span>{tab.label}</span>
            </button>
          ))}
        </div>

        <div className="flex items-center gap-2">
          {/* Minimize/Maximize button */}
          <button
            onClick={() => setIsMinimized(!isMinimized)}
            className="p-1.5 rounded-md text-gray-400 hover:text-white hover:bg-dark-elevated transition-colors"
            title={isMinimized ? "Maximize" : "Minimize"}
          >
            {isMinimized ? (
              <svg
                className="w-4 h-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M5 15l7-7 7 7"
                />
              </svg>
            ) : (
              <svg
                className="w-4 h-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M19 9l-7 7-7-7"
                />
              </svg>
            )}
          </button>

          {/* Close button */}
          <button
            onClick={onClose}
            className="p-1.5 rounded-md text-gray-400 hover:text-white hover:bg-dark-elevated transition-colors"
            title="Close"
          >
            <svg
              className="w-4 h-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>
      </div>

      {/* Tab content */}
      {!isMinimized && (
        <div className="flex-1 overflow-auto custom-scrollbar">
          {activeTab === "general" && <GeneralTab torrent={torrent} />}
          {activeTab === "trackers" && <TrackersTab torrent={torrent} />}
          {activeTab === "peers" && <PeersTab torrent={torrent} />}
          {activeTab === "pieces" && <PiecesTab torrent={torrent} />}
          {activeTab === "files" && <FilesTab torrent={torrent} />}
        </div>
      )}
    </div>
  );
}
