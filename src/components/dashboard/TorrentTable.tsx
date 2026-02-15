import { useState, useMemo } from "react";
import { useTorrentStore } from "../../stores/useTorrentStore";
import { useUIStore } from "../../stores/useUIStore";
import { TorrentInfo, TorrentState, DownloadSource } from "../../types";
import { formatBytes, formatSpeed, calculateETA, cn } from "../../lib/utils";
import {
    Play,
    Pause,
    Info,
    Trash2,
    ArrowUp,
    ArrowDown,
    HardDrive,
    Cloud,
    Share2,
} from "lucide-react";
import { Badge } from "../ui/Badge";

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

export function TorrentTable() {
    const torrents = useTorrentStore((state) => state.filteredTorrents);
    const selectedIds = useTorrentStore((state) => state.selectedIds);

    const toggleSelection = useTorrentStore((state) => state.toggleSelection);
    const startTorrent = useTorrentStore((state) => state.startTorrent);
    const pauseTorrent = useTorrentStore((state) => state.pauseTorrent);
    const removeTorrent = useTorrentStore((state) => state.removeTorrent);
    const openDetails = useUIStore((state) => state.openDetails);

    const [sortColumn, setSortColumn] = useState<SortColumn>("name");
    const [sortDirection, setSortDirection] = useState<SortDirection>("asc");
    const [contextMenu, setContextMenu] = useState<{
        x: number;
        y: number;
        torrentId: string;
    } | null>(null);

    const sortedTorrents = useMemo(() => {
        const sorted = [...torrents].sort((a, b) => {
            let comparison = 0;
            switch (sortColumn) {
                case "name": comparison = a.name.localeCompare(b.name); break;
                case "size": comparison = a.size - b.size; break;
                case "progress":
                    const progA = a.size > 0 ? a.downloaded / a.size : 0;
                    const progB = b.size > 0 ? b.downloaded / b.size : 0;
                    comparison = progA - progB;
                    break;
                case "state": comparison = a.state.localeCompare(b.state); break;
                case "source": comparison = a.source.localeCompare(b.source); break;
                case "download_speed": comparison = a.download_speed - b.download_speed; break;
                case "upload_speed": comparison = a.upload_speed - b.upload_speed; break;
                case "eta":
                    // Simple ETA compare
                    // This is a bit rough, ideally calculate seconds
                    const etaA = calculateETA(a.size - a.downloaded, a.download_speed);
                    const etaB = calculateETA(b.size - b.downloaded, b.download_speed);
                    if (etaA === "∞") comparison = 1;
                    else if (etaB === "∞") comparison = -1;
                    else comparison = 0; // Rough approx
                    break;
                case "ratio":
                    const rA = a.downloaded > 0 ? a.uploaded / a.downloaded : 0;
                    const rB = b.downloaded > 0 ? b.uploaded / b.downloaded : 0;
                    comparison = rA - rB;
                    break;
                case "peers": comparison = a.peers - b.peers; break;
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
        // Prevent selection when clicking buttons
        if ((e.target as HTMLElement).closest("button")) return;

        if (e.ctrlKey || e.metaKey) {
            toggleSelection(torrent.id, true);
        } else {
            toggleSelection(torrent.id, false);
        }
    };

    const handleContextMenu = (e: React.MouseEvent, torrentId: string) => {
        e.preventDefault();
        setContextMenu({ x: e.clientX, y: e.clientY, torrentId });
    };

    // Close context menu on click outside
    const handleCloseContextMenu = () => setContextMenu(null);

    const getStateBadgeVariant = (state: TorrentState) => {
        switch (state) {
            case TorrentState.Downloading: return "default";
            case TorrentState.Seeding: return "success";
            case TorrentState.Paused: return "secondary";
            case TorrentState.Error: return "error";
            case TorrentState.Checking: return "warning";
            case TorrentState.Queued: return "info";
            default: return "secondary";
        }
    };

    const getSourceIcon = (source: DownloadSource) => {
        switch (source) {
            case DownloadSource.Cloud: return <Cloud className="h-3 w-3" />;
            case DownloadSource.P2P: return <Share2 className="h-3 w-3" />;
            case DownloadSource.Hybrid: return <Zap className="h-3 w-3" />; // Zap defined below or import
            default: return <HardDrive className="h-3 w-3" />;
        }
    };

    return (
        <div className="flex flex-col h-full bg-dark-bg/30 relative">
            {/* Header */}
            <div className="bg-dark-surface border-b border-dark-border sticky top-0 z-10">
                <div className="grid grid-cols-[40px_minmax(200px,3fr)_100px_140px_100px_100px_100px_100px_80px_60px_60px] text-xs text-text-tertiary">
                    <div className="p-3 flex items-center justify-center">
                        <input type="checkbox" className="rounded border-dark-border bg-dark-bg focus:ring-primary" />
                    </div>

                    <SortableHeader label="Name" column="name" currentSort={sortColumn} direction={sortDirection} onSort={handleSort} />
                    <SortableHeader label="Size" column="size" currentSort={sortColumn} direction={sortDirection} onSort={handleSort} className="justify-end" />
                    <SortableHeader label="Progress" column="progress" currentSort={sortColumn} direction={sortDirection} onSort={handleSort} className="justify-start pl-4" />

                    <SortableHeader label="Status" column="state" currentSort={sortColumn} direction={sortDirection} onSort={handleSort} className="justify-center" />
                    <SortableHeader label="Source" column="source" currentSort={sortColumn} direction={sortDirection} onSort={handleSort} className="justify-center" />

                    <SortableHeader label="Down" column="download_speed" currentSort={sortColumn} direction={sortDirection} onSort={handleSort} className="justify-end" />
                    <SortableHeader label="Up" column="upload_speed" currentSort={sortColumn} direction={sortDirection} onSort={handleSort} className="justify-end" />

                    <SortableHeader label="ETA" column="eta" currentSort={sortColumn} direction={sortDirection} onSort={handleSort} className="justify-end" />
                    <SortableHeader label="Ratio" column="ratio" currentSort={sortColumn} direction={sortDirection} onSort={handleSort} className="justify-end" />
                    <SortableHeader label="Peers" column="peers" currentSort={sortColumn} direction={sortDirection} onSort={handleSort} className="justify-center" />
                </div>
            </div>

            {/* Body */}
            <div className="flex-1 overflow-y-auto custom-scrollbar">
                {sortedTorrents.length === 0 ? (
                    <div className="flex flex-col items-center justify-center h-full text-text-tertiary opacity-60">
                        <HardDrive className="h-16 w-16 mb-4 opacity-50" />
                        <p className="text-lg font-medium">No torrents found</p>
                        <p className="text-sm">Add a torrent to get started</p>
                    </div>
                ) : (
                    sortedTorrents.map(torrent => {
                        const isSelected = selectedIds.has(torrent.id);
                        const progress = torrent.size > 0 ? (torrent.downloaded / torrent.size) * 100 : 0;
                        const ratio = torrent.downloaded > 0 ? torrent.uploaded / torrent.downloaded : 0;
                        const eta = calculateETA(torrent.size - torrent.downloaded, torrent.download_speed);

                        return (
                            <div
                                key={torrent.id}
                                className={cn(
                                    "grid grid-cols-[40px_minmax(200px,3fr)_100px_140px_100px_100px_100px_100px_80px_60px_60px] items-center text-sm border-b border-dark-border hover:bg-dark-surface-hover transition-colors cursor-pointer group",
                                    isSelected && "bg-primary/5 hover:bg-primary/10"
                                )}
                                onClick={(e) => handleRowClick(torrent, e)}
                                onDoubleClick={() => openDetails(torrent)}
                                onContextMenu={(e) => handleContextMenu(e, torrent.id)}
                            >
                                <div className="p-3 flex items-center justify-center">
                                    <input
                                        type="checkbox"
                                        checked={isSelected}
                                        readOnly
                                        className="rounded border-dark-border bg-dark-bg focus:ring-primary"
                                    />
                                </div>

                                <div className="p-2 pl-3 font-medium text-text-primary truncate" title={torrent.name}>
                                    {torrent.name}
                                </div>

                                <div className="p-2 text-right text-text-secondary font-mono text-xs">
                                    {formatBytes(torrent.size)}
                                </div>

                                <div className="p-2 pl-4">
                                    <div className="h-2 w-full bg-dark-bg rounded-full overflow-hidden border border-dark-border/50">
                                        <div
                                            className="h-full bg-primary relative overflow-hidden transition-all duration-500"
                                            style={{ width: `${Math.min(progress, 100)}%` }}
                                        >
                                            {progress < 100 && <div className="absolute inset-0 bg-white/20 animate-[shimmer_2s_infinite]" />}
                                        </div>
                                    </div>
                                    <div className="text-[10px] text-text-tertiary mt-0.5 font-mono text-center">
                                        {progress.toFixed(1)}%
                                    </div>
                                </div>

                                <div className="p-2 flex justify-center">
                                    <Badge variant={getStateBadgeVariant(torrent.state)} className="capitalize">
                                        {torrent.state}
                                    </Badge>
                                </div>

                                <div className="p-2 flex justify-center">
                                    <div className={cn("flex items-center gap-1.5 px-2 py-0.5 rounded text-xs font-medium border bg-dark-bg",
                                        torrent.source === DownloadSource.Cloud ? "text-blue-400 border-blue-500/20" :
                                            torrent.source === DownloadSource.P2P ? "text-purple-400 border-purple-500/20" :
                                                "text-cyan-400 border-cyan-500/20"
                                    )}>
                                        {getSourceIcon(torrent.source)}
                                        <span className="capitalize">{torrent.source}</span>
                                    </div>
                                </div>

                                <div className="p-2 text-right font-mono text-xs text-primary">
                                    {torrent.download_speed > 0 ? formatSpeed(torrent.download_speed) : "-"}
                                </div>

                                <div className="p-2 text-right font-mono text-xs text-success">
                                    {torrent.upload_speed > 0 ? formatSpeed(torrent.upload_speed) : "-"}
                                </div>

                                <div className="p-2 text-right text-text-secondary text-xs">
                                    {eta}
                                </div>

                                <div className="p-2 text-right text-text-secondary text-xs">
                                    {ratio.toFixed(2)}
                                </div>

                                <div className="p-2 text-center text-text-secondary text-xs">
                                    {torrent.peers}
                                </div>
                            </div>
                        );
                    })
                )}
            </div>

            {/* Context Menu Overlay */}
            {contextMenu && (
                <>
                    <div className="fixed inset-0 z-40" onClick={handleCloseContextMenu} />
                    <div
                        className="fixed z-50 min-w-[160px] bg-dark-surface border border-dark-border rounded-lg shadow-xl p-1 animate-scale-in"
                        style={{ left: contextMenu.x, top: contextMenu.y }}
                    >
                        <ContextMenuButton
                            icon={<Play className="h-4 w-4 text-success" />}
                            label="Start"
                            onClick={() => { startTorrent(contextMenu.torrentId); handleCloseContextMenu(); }}
                        />
                        <ContextMenuButton
                            icon={<Pause className="h-4 w-4 text-warning" />}
                            label="Pause"
                            onClick={() => { pauseTorrent(contextMenu.torrentId); handleCloseContextMenu(); }}
                        />
                        <div className="my-1 border-t border-dark-border" />
                        <ContextMenuButton
                            icon={<Info className="h-4 w-4 text-primary" />}
                            label="Details"
                            onClick={() => {
                                const t = torrents.find(t => t.id === contextMenu.torrentId);
                                if (t) openDetails(t);
                                handleCloseContextMenu();
                            }}
                        />
                        <div className="my-1 border-t border-dark-border" />
                        <ContextMenuButton
                            icon={<Trash2 className="h-4 w-4 text-error" />}
                            label="Remove"
                            onClick={() => { removeTorrent(contextMenu.torrentId, false); handleCloseContextMenu(); }}
                            className="text-error hover:bg-error/10"
                        />
                    </div>
                </>
            )}
        </div>
    );
}

function SortableHeader({
    label,
    column,
    currentSort,
    direction,
    onSort,
    className
}: {
    label: string,
    column: SortColumn,
    currentSort: SortColumn,
    direction: SortDirection,
    onSort: (col: SortColumn) => void,
    className?: string
}) {
    return (
        <div
            className={cn("p-3 flex items-center cursor-pointer hover:text-text-primary transition-colors select-none", className)}
            onClick={() => onSort(column)}
        >
            <span>{label}</span>
            <span className="ml-1 w-3 inline-block">
                {currentSort === column && (
                    direction === "asc" ? <ArrowUp className="h-3 w-3" /> : <ArrowDown className="h-3 w-3" />
                )}
            </span>
        </div>
    );
}

function ContextMenuButton({ icon, label, onClick, className }: { icon: React.ReactNode, label: string, onClick: () => void, className?: string }) {
    return (
        <button
            className={cn("flex w-full items-center gap-2 rounded px-2 py-1.5 text-sm text-text-primary hover:bg-dark-elem-hover transition-colors text-left", className)}
            onClick={onClick}
        >
            {icon}
            <span>{label}</span>
        </button>
    )
}

function Zap(props: React.SVGProps<SVGSVGElement>) {
    return (
        <svg
            {...props}
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
        >
            <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
        </svg>
    )
}
