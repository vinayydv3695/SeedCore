import { TorrentInfo, TorrentState } from "../../types";
import { formatBytes, formatSpeed, calculateETA, cn } from "../../lib/utils";
import { useTorrentStore } from "../../stores/useTorrentStore";
import { useUIStore } from "../../stores/useUIStore";
import {
    Download,
    Upload,
    Play,
    Pause,
    Trash2,
} from "lucide-react";
import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";
import { Card } from "../ui/Card";

interface TorrentItemProps {
    torrent: TorrentInfo;
}

export function TorrentItem({ torrent }: TorrentItemProps) {
    const startTorrent = useTorrentStore((state) => state.startTorrent);
    const pauseTorrent = useTorrentStore((state) => state.pauseTorrent);
    const removeTorrent = useTorrentStore((state) => state.removeTorrent);
    const openDetails = useUIStore((state) => state.openDetails);

    // Check if selected? 
    // In card view usually selection is less common, but let's assume we might want it.
    // For now we just focus on actions.

    const progress = torrent.size > 0 ? (torrent.downloaded / torrent.size) * 100 : 0;
    const eta = calculateETA(torrent.size - torrent.downloaded, torrent.download_speed);

    const getStateBadgeVariant = (state: TorrentState) => {
        switch (state) {
            case TorrentState.Downloading: return "default";
            case TorrentState.Seeding: return "success";
            case TorrentState.Paused: return "secondary";
            case TorrentState.Error: return "error";
            case TorrentState.Checking: return "warning";
            default: return "secondary";
        }
    };

    return (
        <Card className="hover:border-primary/50 transition-colors group relative overflow-hidden">
            {/* Progress Background (Optional, maybe too much?) */}
            {/* <div className="absolute bottom-0 left-0 h-1 bg-primary" style={{ width: `${progress}%` }} /> */}

            <div className="p-4 space-y-4">
                <div className="flex justify-between items-start gap-4">
                    <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2 mb-1">
                            <h3 className="font-medium text-text-primary truncate cursor-pointer hover:underline" onClick={() => openDetails(torrent)}>
                                {torrent.name}
                            </h3>
                            <Badge variant={getStateBadgeVariant(torrent.state)} className="text-[10px] px-1.5 py-0 h-5">
                                {torrent.state}
                            </Badge>
                        </div>
                        <div className="text-xs text-text-secondary flex items-center gap-2">
                            <span>{formatBytes(torrent.size)}</span>
                            <span>•</span>
                            <span>{torrent.source}</span>
                        </div>
                    </div>

                    <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                        {torrent.state === TorrentState.Paused || torrent.state === TorrentState.Error ? (
                            <Button variant="ghost" size="icon" className="h-8 w-8 text-success hover:text-success hover:bg-success/10" onClick={() => startTorrent(torrent.id)}>
                                <Play className="h-4 w-4" />
                            </Button>
                        ) : (
                            <Button variant="ghost" size="icon" className="h-8 w-8 text-warning hover:text-warning hover:bg-warning/10" onClick={() => pauseTorrent(torrent.id)}>
                                <Pause className="h-4 w-4" />
                            </Button>
                        )}
                        <Button variant="ghost" size="icon" className="h-8 w-8 text-error hover:text-error hover:bg-error/10" onClick={() => removeTorrent(torrent.id, false)}>
                            <Trash2 className="h-4 w-4" />
                        </Button>
                    </div>
                </div>

                {/* Progress Bar */}
                <div className="space-y-1.5">
                    <div className="flex justify-between text-xs text-text-secondary">
                        <span>{progress.toFixed(1)}%</span>
                        <span>ETA: {eta}</span>
                    </div>
                    <div className="h-2 w-full bg-dark-bg rounded-full overflow-hidden border border-dark-border">
                        <div
                            className="h-full bg-primary relative overflow-hidden transition-all duration-500"
                            style={{ width: `${Math.min(progress, 100)}%` }}
                        >
                            {progress < 100 && <div className="absolute inset-0 bg-white/20 animate-[shimmer_2s_infinite]" />}
                        </div>
                    </div>
                </div>

                {/* Stats */}
                <div className="flex items-center justify-between text-xs pt-2 border-t border-dark-border/50">
                    <div className="flex items-center gap-4">
                        <div className="flex items-center gap-1.5 text-text-secondary" title="Download Speed">
                            <Download className={cn("h-3.5 w-3.5", torrent.download_speed > 0 ? "text-primary" : "text-text-tertiary")} />
                            <span className={torrent.download_speed > 0 ? "text-text-primary font-medium" : ""}>
                                {formatSpeed(torrent.download_speed)}
                            </span>
                        </div>
                        <div className="flex items-center gap-1.5 text-text-secondary" title="Upload Speed">
                            <Upload className={cn("h-3.5 w-3.5", torrent.upload_speed > 0 ? "text-success" : "text-text-tertiary")} />
                            <span className={torrent.upload_speed > 0 ? "text-text-primary font-medium" : ""}>
                                {formatSpeed(torrent.upload_speed)}
                            </span>
                        </div>
                    </div>

                    <div className="text-text-tertiary">
                        Ratio: {(torrent.downloaded > 0 ? torrent.uploaded / torrent.downloaded : 0).toFixed(2)}
                    </div>
                </div>
            </div>
        </Card>
    );
}
