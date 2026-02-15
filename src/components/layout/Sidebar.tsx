import {
    Archive,
    CheckCircle2,
    AlertCircle,
    Clock,
    Download,
    Upload,
    FolderOpen,
    Hash,
    HardDrive
} from "lucide-react";
import { useTorrentStore } from "../../stores/useTorrentStore";
import { TorrentState, TorrentInfo } from "../../types";
import { cn } from "../../lib/utils";
import { Badge } from "../ui/Badge";
import { formatBytes } from "../../lib/utils";
import { useSettingsStore } from "../../stores/useSettingsStore";

interface FilterItem {
    id: string;
    label: string;
    icon: React.ElementType;
    count: (torrents: TorrentInfo[]) => number;
}

export function Sidebar() {
    const torrents = useTorrentStore((state) => state.torrents);
    const selectedFilter = useTorrentStore((state) => state.filterStatus);
    const setFilterStatus = useTorrentStore((state) => state.setFilterStatus);

    // Settings store for disk space
    const diskSpace = useSettingsStore((state) => state.diskSpace);

    const filters: FilterItem[] = [
        {
            id: "all",
            label: "All Torrents",
            icon: Archive,
            count: (t) => t.length,
        },
        {
            id: "downloading",
            label: "Downloading",
            icon: Download,
            count: (t) => t.filter((x) => x.state === TorrentState.Downloading).length,
        },
        {
            id: "seeding",
            label: "Seeding",
            icon: Upload,
            count: (t) => t.filter((x) => x.state === TorrentState.Seeding).length,
        },
        {
            id: "active",
            label: "Active",
            icon: Clock,
            count: (t) =>
                t.filter(
                    (x) =>
                        x.state === TorrentState.Downloading ||
                        x.state === TorrentState.Seeding
                ).length,
        },
        {
            id: "completed",
            label: "Completed",
            icon: CheckCircle2,
            count: (t) =>
                t.filter((x) => x.downloaded >= x.size && x.size > 0).length,
        },
        {
            id: "error",
            label: "Errors",
            icon: AlertCircle,
            count: (t) => t.filter((x) => x.state === TorrentState.Error).length,
        },
    ];

    const categories = [
        { id: "movies", name: "Movies", count: 0 },
        { id: "tv", name: "TV Shows", count: 0 },
        { id: "games", name: "Games", count: 0 },
        { id: "music", name: "Music", count: 0 },
        { id: "software", name: "Software", count: 0 },
        { id: "books", name: "Books", count: 0 },
        { id: "uncategorized", name: "Uncategorized", count: torrents.length },
    ];

    return (
        <div className="flex w-64 flex-col border-r border-dark-border bg-dark-surface h-full">
            {/* Filters */}
            <div className="flex-1 overflow-y-auto scrollbar-custom p-4 space-y-6">
                {/* Main Filters */}
                <div className="space-y-1">
                    <h3 className="mb-2 px-2 text-xs font-semibold uppercase tracking-wider text-text-tertiary">
                        Library
                    </h3>
                    {filters.map((filter) => {
                        const isActive = selectedFilter === filter.id;
                        const count = filter.count(torrents);

                        return (
                            <button
                                key={filter.id}
                                onClick={() => setFilterStatus(filter.id)}
                                className={cn(
                                    "flex w-full items-center justify-between rounded-md px-3 py-2 text-sm font-medium transition-all duration-200 group",
                                    isActive
                                        ? "bg-primary/10 text-primary border-l-2 border-primary -ml-[1px]"
                                        : "text-text-secondary hover:bg-dark-surface-hover hover:text-text-primary"
                                )}
                            >
                                <div className="flex items-center gap-3">
                                    <filter.icon className={cn("h-4 w-4", isActive ? "text-primary" : "text-text-tertiary group-hover:text-text-primary")} />
                                    <span>{filter.label}</span>
                                </div>
                                {count > 0 && (
                                    <Badge variant={isActive ? "default" : "secondary"} className={cn("px-1.5 py-0 min-w-[1.25rem] justify-center", !isActive && "bg-dark-surface-active text-text-tertiary")}>
                                        {count}
                                    </Badge>
                                )}
                            </button>
                        );
                    })}
                </div>

                {/* Categories (Mock data for now) */}
                <div className="space-y-1">
                    <div className="flex items-center justify-between px-2 mb-2">
                        <h3 className="text-xs font-semibold uppercase tracking-wider text-text-tertiary">
                            Categories
                        </h3>
                    </div>
                    {categories.map((category) => (
                        <button
                            key={category.id}
                            className="flex w-full items-center justify-between rounded-md px-3 py-2 text-sm font-medium text-text-secondary hover:bg-dark-surface-hover hover:text-text-primary transition-all duration-200 group"
                        >
                            <div className="flex items-center gap-3">
                                <FolderOpen className="h-4 w-4 text-text-tertiary group-hover:text-text-primary" />
                                <span>{category.name}</span>
                            </div>
                        </button>
                    ))}
                </div>

                {/* Tags (Mock data) */}
                <div className="space-y-1">
                    <div className="flex items-center justify-between px-2 mb-2">
                        <h3 className="text-xs font-semibold uppercase tracking-wider text-text-tertiary">
                            Tags
                        </h3>
                    </div>
                    <button className="flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-text-secondary hover:bg-dark-surface-hover hover:text-text-primary transition-all duration-200 group">
                        <Hash className="h-4 w-4 text-text-tertiary group-hover:text-text-primary" />
                        <span>Favorites</span>
                    </button>
                </div>
            </div>

            {/* Footer / Storage */}
            <div className="border-t border-dark-border p-4 bg-dark-bg/30">
                <div className="flex items-center justify-between text-xs text-text-secondary mb-2">
                    <div className="flex items-center gap-2">
                        <HardDrive className="h-3 w-3" />
                        <span>Free Space</span>
                    </div>
                    <span className="font-medium text-text-primary">{diskSpace ? formatBytes(diskSpace.free) : "..."}</span>
                </div>
                {/* Simple progress bar for disk space */}
                {diskSpace && (
                    <div className="h-1.5 w-full bg-dark-border rounded-full overflow-hidden">
                        <div
                            className="h-full bg-primary/50"
                            style={{ width: `${Math.min(((diskSpace.total - diskSpace.free) / diskSpace.total) * 100, 100)}%` }}
                        />
                    </div>
                )}
            </div>
        </div>
    );
}
