import { useTorrentStore } from "../../stores/useTorrentStore";
import { TorrentItem } from "./TorrentItem";
import { HardDrive } from "lucide-react";

export function TorrentList() {
    const filteredTorrents = useTorrentStore((state) => state.filteredTorrents);

    // Stats for the filter pills if we wanted to show them here, 
    // but Sidebar handles main filtering. TopBar handles View Toggle.
    // We can show a simple header or just the grid.
    // Sidebar handles filters. So here we just show the grid.

    return (
        <div className="flex flex-col h-full bg-dark-bg/30 p-4 overflow-y-auto custom-scrollbar">
            {filteredTorrents.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-full text-text-tertiary opacity-60">
                    <HardDrive className="h-16 w-16 mb-4 opacity-50" />
                    <p className="text-lg font-medium">No torrents found</p>
                </div>
            ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                    {filteredTorrents.map((torrent) => (
                        <TorrentItem key={torrent.id} torrent={torrent} />
                    ))}
                </div>
            )}
        </div>
    );
}
