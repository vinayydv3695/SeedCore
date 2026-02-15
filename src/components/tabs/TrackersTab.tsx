import { TorrentInfo } from "../../types";
import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";
import { Plus, RefreshCw } from "lucide-react";
import { cn } from "../../lib/utils";

interface TrackersTabProps {
  torrent: TorrentInfo;
}

interface TrackerInfo {
  url: string;
  status: "Working" | "Updating" | "Error" | "Disabled";
  message: string;
  peers: number;
  seeds: number;
  leechers: number;
  downloaded: number;
  lastAnnounce: string;
  nextAnnounce: string;
}

export function TrackersTab({ torrent: _torrent }: TrackersTabProps) {
  // Mock tracker data
  const trackers: TrackerInfo[] = [
    {
      url: "udp://tracker.opentrackr.org:1337/announce",
      status: "Working",
      message: "Announce OK",
      peers: 150,
      seeds: 45,
      leechers: 105,
      downloaded: 1250,
      lastAnnounce: "2 min ago",
      nextAnnounce: "28 min",
    },
    {
      url: "udp://tracker.openbittorrent.com:6969/announce",
      status: "Working",
      message: "Announce OK",
      peers: 89,
      seeds: 23,
      leechers: 66,
      downloaded: 890,
      lastAnnounce: "3 min ago",
      nextAnnounce: "27 min",
    },
    {
      url: "udp://exodus.desync.com:6969/announce",
      status: "Updating",
      message: "Announcing...",
      peers: 0,
      seeds: 0,
      leechers: 0,
      downloaded: 0,
      lastAnnounce: "Never",
      nextAnnounce: "Now",
    },
  ];

  const getStatusBadgeVariant = (status: TrackerInfo["status"]) => {
    switch (status) {
      case "Working": return "success";
      case "Updating": return "info";
      case "Error": return "error";
      case "Disabled": return "secondary";
      default: return "secondary";
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="mb-4 flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <Button size="sm" variant="primary" leftIcon={<Plus className="h-3.5 w-3.5" />}>
            Add Tracker
          </Button>
          <Button size="sm" variant="secondary" leftIcon={<RefreshCw className="h-3.5 w-3.5" />}>
            Force Announce
          </Button>
        </div>
        <span className="text-xs text-text-tertiary font-medium px-2">
          {trackers.length} tracker{trackers.length !== 1 ? "s" : ""}
        </span>
      </div>

      {/* Trackers table */}
      <div className="bg-dark-surface-elevated border border-dark-border rounded-lg overflow-hidden flex-1">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead className="bg-dark-bg/50 border-b border-dark-border text-xs uppercase text-text-tertiary font-medium">
              <tr>
                <th className="px-4 py-3">Tracker URL</th>
                <th className="px-4 py-3 text-center w-24">Status</th>
                <th className="px-4 py-3 text-center w-20">Peers</th>
                <th className="px-4 py-3 text-center w-20">Seeds</th>
                <th className="px-4 py-3 text-center w-20">Leech</th>
                <th className="px-4 py-3 text-right w-32">Next Announce</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-dark-border">
              {trackers.map((tracker, index) => (
                <tr key={index} className="hover:bg-dark-surface-hover transition-colors group">
                  <td className="px-4 py-3 min-w-[200px]">
                    <div className="flex flex-col">
                      <span className="text-text-primary font-mono text-xs truncate max-w-[300px]" title={tracker.url}>
                        {tracker.url}
                      </span>
                      <span className={cn(
                        "text-xs mt-0.5",
                        tracker.status === "Error" ? "text-error" : "text-text-tertiary"
                      )}>
                        {tracker.message}
                      </span>
                    </div>
                  </td>
                  <td className="px-4 py-3 text-center">
                    <Badge variant={getStatusBadgeVariant(tracker.status)} className="w-full justify-center">
                      {tracker.status}
                    </Badge>
                  </td>
                  <td className="px-4 py-3 text-center text-text-secondary">
                    {tracker.peers > 0 ? tracker.peers : "-"}
                  </td>
                  <td className="px-4 py-3 text-center text-success">
                    {tracker.seeds > 0 ? tracker.seeds : "-"}
                  </td>
                  <td className="px-4 py-3 text-center text-primary">
                    {tracker.leechers > 0 ? tracker.leechers : "-"}
                  </td>
                  <td className="px-4 py-3 text-right text-text-tertiary font-mono text-xs">
                    {tracker.nextAnnounce}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
