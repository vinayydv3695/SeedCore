import { TorrentInfo } from "../../types";

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
  // Mock tracker data - will be replaced with real data in Phase 8C
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

  const getStatusColor = (status: TrackerInfo["status"]) => {
    switch (status) {
      case "Working":
        return "text-success";
      case "Updating":
        return "text-info";
      case "Error":
        return "text-error";
      case "Disabled":
        return "text-gray-500";
      default:
        return "text-gray-400";
    }
  };

  const getStatusIcon = (status: TrackerInfo["status"]) => {
    switch (status) {
      case "Working":
        return "✅";
      case "Updating":
        return "🔄";
      case "Error":
        return "❌";
      case "Disabled":
        return "⏸️";
      default:
        return "❓";
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="p-3 border-b border-dark-border flex items-center gap-2">
        <button className="px-3 py-1.5 text-sm bg-primary hover:bg-primary-hover text-white rounded-md transition-colors">
          Add Tracker
        </button>
        <button className="px-3 py-1.5 text-sm bg-dark-elevated hover:bg-dark-border text-gray-300 rounded-md transition-colors">
          Force Announce
        </button>
        <div className="flex-1" />
        <span className="text-sm text-gray-400">
          {trackers.length} tracker{trackers.length !== 1 ? "s" : ""}
        </span>
      </div>

      {/* Trackers table */}
      <div className="flex-1 overflow-auto custom-scrollbar">
        {trackers.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <div className="text-6xl mb-4">🌐</div>
            <p className="text-lg font-medium">No trackers</p>
            <p className="text-sm">Add a tracker to get started</p>
          </div>
        ) : (
          <table className="w-full">
            <thead className="bg-dark-secondary sticky top-0 z-10">
              <tr className="text-xs text-gray-400 uppercase">
                <th className="text-left p-3 font-semibold">Tracker URL</th>
                <th className="text-center p-3 font-semibold w-32">Status</th>
                <th className="text-center p-3 font-semibold w-24">Peers</th>
                <th className="text-center p-3 font-semibold w-24">Seeds</th>
                <th className="text-center p-3 font-semibold w-24">Leechers</th>
                <th className="text-center p-3 font-semibold w-28">Downloaded</th>
                <th className="text-center p-3 font-semibold w-32">Last Announce</th>
                <th className="text-center p-3 font-semibold w-32">Next Announce</th>
              </tr>
            </thead>
            <tbody>
              {trackers.map((tracker, index) => (
                <tr
                  key={index}
                  className="border-b border-dark-border hover:bg-dark-elevated transition-colors text-sm"
                >
                  <td className="p-3">
                    <div className="flex flex-col">
                      <span className="text-white font-mono text-xs break-all">
                        {tracker.url}
                      </span>
                      <span className="text-gray-500 text-xs mt-0.5">
                        {tracker.message}
                      </span>
                    </div>
                  </td>
                  <td className="p-3 text-center">
                    <div className="flex items-center justify-center gap-1">
                      <span>{getStatusIcon(tracker.status)}</span>
                      <span className={`font-medium ${getStatusColor(tracker.status)}`}>
                        {tracker.status}
                      </span>
                    </div>
                  </td>
                  <td className="p-3 text-center text-gray-300">
                    {tracker.peers > 0 ? tracker.peers : "-"}
                  </td>
                  <td className="p-3 text-center text-success">
                    {tracker.seeds > 0 ? tracker.seeds : "-"}
                  </td>
                  <td className="p-3 text-center text-primary">
                    {tracker.leechers > 0 ? tracker.leechers : "-"}
                  </td>
                  <td className="p-3 text-center text-gray-300">
                    {tracker.downloaded > 0 ? tracker.downloaded.toLocaleString() : "-"}
                  </td>
                  <td className="p-3 text-center text-gray-400 text-xs">
                    {tracker.lastAnnounce}
                  </td>
                  <td className="p-3 text-center text-gray-400 text-xs">
                    {tracker.nextAnnounce}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
