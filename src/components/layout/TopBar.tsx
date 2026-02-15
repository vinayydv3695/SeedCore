import {
    Search,
    Settings,
    Plus,
    ArrowDown,
    ArrowUp,
    Zap
} from "lucide-react";
import { useTorrentStore } from "../../stores/useTorrentStore";
import { useUIStore } from "../../stores/useUIStore";
import { formatSpeed, cn } from "../../lib/utils";
import { Button } from "../ui/Button";
import { Input } from "../ui/Input";
import { ViewToggle } from "../ViewToggle"; // We'll update this later or assume compatibility

export function TopBar() {
    const torrents = useTorrentStore((state) => state.torrents);
    const searchQuery = useTorrentStore((state) => state.searchQuery);
    const setSearchQuery = useTorrentStore((state) => state.setSearchQuery);

    // UI Actions
    const openAddTorrentDialog = useUIStore((state) => state.openAddTorrentDialog);
    const openSettings = useUIStore((state) => state.openSettings);
    const viewMode = useUIStore((state) => state.viewMode);
    const setViewMode = useUIStore((state) => state.setViewMode);

    // Calculate global stats
    const totalDownloadSpeed = torrents.reduce((sum, t) => sum + t.download_speed, 0);
    const totalUploadSpeed = torrents.reduce((sum, t) => sum + t.upload_speed, 0);
    /* const activeTorrents = torrents.filter(
        (t) => t.state === "Downloading" || t.state === "Seeding"
    ).length; */

    return (
        <header className="flex h-16 items-center justify-between border-b border-dark-border bg-dark-surface px-6 py-3 shadow-sm z-30">
            {/* Logo & Branding */}
            <div className="flex items-center gap-3 min-w-[200px]">
                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-primary-hover shadow-glow-sm">
                    <Zap className="h-5 w-5 text-white fill-current" />
                </div>
                <div className="flex flex-col justify-center">
                    <h1 className="text-lg font-bold text-white leading-none tracking-tight">SeedCore</h1>
                    <span className="text-[10px] uppercase font-bold text-primary tracking-widest mt-0.5">Pro</span>
                </div>
            </div>

            {/* Center: Search & Stats */}
            <div className="flex flex-1 items-center justify-center gap-6 max-w-2xl px-4">
                {/* Search Bar */}
                <div className="w-full max-w-sm">
                    <Input
                        placeholder="Search torrents..."
                        icon={<Search className="h-4 w-4" />}
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="bg-dark-bg/50 border-dark-border focus:bg-dark-surface transition-colors"
                    />
                </div>

                {/* Quick Stats (Divider) */}
                <div className="h-4 w-px bg-dark-border hidden md:block" />

                {/* Speeds */}
                <div className="hidden md:flex items-center gap-6 text-xs font-medium">
                    <div className="flex items-center gap-2 text-text-secondary" title="Total Download Speed">
                        <ArrowDown className={cn("h-4 w-4", totalDownloadSpeed > 0 ? "text-primary animate-pulse" : "text-text-tertiary")} />
                        <span className={cn(totalDownloadSpeed > 0 ? "text-text-primary" : "text-text-tertiary")}>
                            {formatSpeed(totalDownloadSpeed)}
                        </span>
                    </div>
                    <div className="flex items-center gap-2 text-text-secondary" title="Total Upload Speed">
                        <ArrowUp className={cn("h-4 w-4", totalUploadSpeed > 0 ? "text-success animate-pulse" : "text-text-tertiary")} />
                        <span className={cn(totalUploadSpeed > 0 ? "text-text-primary" : "text-text-tertiary")}>
                            {formatSpeed(totalUploadSpeed)}
                        </span>
                    </div>
                </div>
            </div>

            {/* Right: Actions */}
            <div className="flex items-center gap-3 min-w-[200px] justify-end">
                <div className="hidden lg:block">
                    <ViewToggle view={viewMode} onChange={setViewMode} />
                </div>

                <div className="h-4 w-px bg-dark-border hidden lg:block" />

                <Button variant="ghost" size="icon" onClick={openSettings} title="Settings">
                    <Settings className="h-5 w-5" />
                </Button>

                <Button onClick={openAddTorrentDialog} leftIcon={<Plus className="h-4 w-4" />}>
                    Add Torrent
                </Button>
            </div>
        </header>
    );
}
