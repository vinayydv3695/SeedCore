import { useState, useRef, useEffect } from "react";
import { useUIStore } from "../../stores/useUIStore";
import { GeneralTab } from "../tabs/GeneralTab";
import { TrackersTab } from "../tabs/TrackersTab";
import { PeersTab } from "../tabs/PeersTab";
import { PiecesTab } from "../tabs/PiecesTab";
import { FilesTab } from "../tabs/FilesTab";
import {
    Info,
    Globe,
    Users,
    Puzzle,
    FolderOpen,
    Maximize2,
    Minimize2,
    X
} from "lucide-react";
import { cn } from "../../lib/utils";

type TabId = "general" | "trackers" | "peers" | "pieces" | "files";

interface Tab {
    id: TabId;
    label: string;
    icon: React.ElementType;
}

export function BottomPanel() {
    const torrent = useUIStore((state) => state.selectedTorrent);
    const closeDetails = useUIStore((state) => state.closeDetails);

    const [activeTab, setActiveTab] = useState<TabId>("general");
    const [panelHeight, setPanelHeight] = useState(320); // Default height
    const [isResizing, setIsResizing] = useState(false);
    const [isMinimized, setIsMinimized] = useState(false);
    const panelRef = useRef<HTMLDivElement>(null);
    const startY = useRef(0);
    const startHeight = useRef(0);

    const tabs: Tab[] = [
        { id: "general", label: "General", icon: Info },
        { id: "trackers", label: "Trackers", icon: Globe },
        { id: "peers", label: "Peers", icon: Users },
        { id: "pieces", label: "Pieces", icon: Puzzle },
        { id: "files", label: "Files", icon: FolderOpen },
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
        if (!torrent) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.ctrlKey) {
                switch (e.key) {
                    case "1": setActiveTab("general"); break;
                    case "2": setActiveTab("trackers"); break;
                    case "3": setActiveTab("peers"); break;
                    case "4": setActiveTab("pieces"); break;
                    case "5": setActiveTab("files"); break;
                }
            }
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, [torrent]);

    if (!torrent) {
        return null;
    }

    const minimizedHeight = 40;
    const currentHeight = isMinimized ? minimizedHeight : panelHeight;

    return (
        <div
            ref={panelRef}
            className="border-t border-dark-border bg-dark-surface flex flex-col shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.1)] transition-height duration-100 ease-out"
            style={{ height: `${currentHeight}px` }}
        >
            {/* Resize handle */}
            <div
                className={cn(
                    "h-1 w-full cursor-row-resize hover:bg-primary transition-colors",
                    isResizing ? "bg-primary" : "bg-transparent hover:bg-primary/50"
                )}
                onMouseDown={handleResizeStart}
            />

            {/* Header with tabs */}
            <div className="flex items-center justify-between border-b border-dark-border bg-dark-surface px-4 py-2 shrink-0 h-10">
                <div className="flex items-center gap-1">
                    {tabs.map((tab) => (
                        <button
                            key={tab.id}
                            onClick={() => setActiveTab(tab.id)}
                            className={cn(
                                "px-3 py-1.5 rounded-md text-xs font-medium transition-all duration-200 flex items-center gap-2",
                                activeTab === tab.id
                                    ? "bg-primary/10 text-primary"
                                    : "text-text-secondary hover:text-text-primary hover:bg-dark-surface-hover"
                            )}
                        >
                            <tab.icon className="h-3.5 w-3.5" />
                            <span>{tab.label}</span>
                        </button>
                    ))}
                </div>

                <div className="flex items-center gap-1">
                    {/* Minimize/Maximize button */}
                    <button
                        onClick={() => setIsMinimized(!isMinimized)}
                        className="p-1.5 rounded-md text-text-tertiary hover:text-text-primary hover:bg-dark-surface-hover transition-colors"
                        title={isMinimized ? "Maximize" : "Minimize"}
                    >
                        {isMinimized ? <Maximize2 className="h-4 w-4" /> : <Minimize2 className="h-4 w-4" />}
                    </button>

                    {/* Close button */}
                    <button
                        onClick={closeDetails}
                        className="p-1.5 rounded-md text-text-tertiary hover:text-error hover:bg-error/10 transition-colors"
                        title="Close"
                    >
                        <X className="h-4 w-4" />
                    </button>
                </div>
            </div>

            {/* Tab content */}
            {!isMinimized && (
                <div className="flex-1 overflow-hidden bg-dark-bg/50">
                    <div className="h-full overflow-y-auto custom-scrollbar p-4">
                        {activeTab === "general" && <GeneralTab torrent={torrent} />}
                        {activeTab === "trackers" && <TrackersTab torrent={torrent} />}
                        {activeTab === "peers" && <PeersTab torrent={torrent} />}
                        {activeTab === "pieces" && <PiecesTab torrent={torrent} />}
                        {activeTab === "files" && <FilesTab torrent={torrent} />}
                    </div>
                </div>
            )}
        </div>
    );
}
