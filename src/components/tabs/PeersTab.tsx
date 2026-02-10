import { TorrentInfo } from "../../types";
import { formatSpeed, formatBytes } from "../../lib/utils";

interface PeersTabProps {
  torrent: TorrentInfo;
}

interface PeerInfo {
  ip: string;
  port: number;
  client: string;
  flags: string;
  progress: number;
  downloadSpeed: number;
  uploadSpeed: number;
  downloaded: number;
  uploaded: number;
  country?: string;
}

export function PeersTab({ torrent: _torrent }: PeersTabProps) {
  // Mock peer data - will be replaced with real data in Phase 8C
  const peers: PeerInfo[] = [
    {
      ip: "192.168.1.100",
      port: 51413,
      client: "qBittorrent 4.5.0",
      flags: "DUI",
      progress: 45.8,
      downloadSpeed: 524288, // 512 KB/s
      uploadSpeed: 0,
      downloaded: 15728640, // 15 MB
      uploaded: 0,
      country: "US",
    },
    {
      ip: "10.0.0.50",
      port: 6881,
      client: "Transmission 3.0",
      flags: "UE",
      progress: 100,
      downloadSpeed: 0,
      uploadSpeed: 1048576, // 1 MB/s
      downloaded: 0,
      uploaded: 52428800, // 50 MB
      country: "CA",
    },
    {
      ip: "172.16.0.25",
      port: 52000,
      client: "Deluge 2.1.1",
      flags: "D",
      progress: 78.3,
      downloadSpeed: 262144, // 256 KB/s
      uploadSpeed: 0,
      downloaded: 8388608, // 8 MB
      uploaded: 0,
      country: "GB",
    },
    {
      ip: "192.168.100.42",
      port: 51234,
      client: "µTorrent 3.5.5",
      flags: "UO",
      progress: 100,
      downloadSpeed: 0,
      uploadSpeed: 524288, // 512 KB/s
      downloaded: 0,
      uploaded: 31457280, // 30 MB
      country: "DE",
    },
    {
      ip: "10.10.10.10",
      port: 6969,
      client: "SeedCore 0.1.0",
      flags: "DI",
      progress: 23.5,
      downloadSpeed: 1048576, // 1 MB/s
      uploadSpeed: 0,
      downloaded: 20971520, // 20 MB
      uploaded: 0,
      country: "FR",
    },
  ];

  const getFlagDescription = (flags: string) => {
    const descriptions: string[] = [];
    if (flags.includes("D")) descriptions.push("Downloading from peer");
    if (flags.includes("U")) descriptions.push("Uploading to peer");
    if (flags.includes("I")) descriptions.push("Interested");
    if (flags.includes("C")) descriptions.push("Choked");
    if (flags.includes("O")) descriptions.push("Optimistic unchoke");
    if (flags.includes("S")) descriptions.push("Snubbed");
    if (flags.includes("E")) descriptions.push("Encrypted");
    return descriptions.join(", ");
  };

  const getFlagColor = (flag: string) => {
    switch (flag) {
      case "D":
        return "text-primary";
      case "U":
        return "text-success";
      case "O":
        return "text-warning";
      case "S":
        return "text-error";
      case "E":
        return "text-info";
      default:
        return "text-gray-400";
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="p-3 border-b border-dark-border flex items-center gap-2">
        <button className="px-3 py-1.5 text-sm bg-primary hover:bg-primary-hover text-white rounded-md transition-colors">
          Add Peer
        </button>
        <button className="px-3 py-1.5 text-sm bg-dark-elevated hover:bg-dark-border text-gray-300 rounded-md transition-colors">
          Ban Selected
        </button>
        <div className="flex-1" />
        <div className="flex items-center gap-4 text-sm">
          <span className="text-gray-400">
            {peers.length} peer{peers.length !== 1 ? "s" : ""}
          </span>
          <span className="text-gray-400">
            Seeds: {peers.filter((p) => p.progress >= 100).length}
          </span>
        </div>
      </div>

      {/* Peers table */}
      <div className="flex-1 overflow-auto custom-scrollbar">
        {peers.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <div className="text-6xl mb-4">👥</div>
            <p className="text-lg font-medium">No peers connected</p>
            <p className="text-sm">Waiting for peer connections...</p>
          </div>
        ) : (
          <table className="w-full">
            <thead className="bg-dark-secondary sticky top-0 z-10">
              <tr className="text-xs text-gray-400 uppercase">
                <th className="text-left p-3 font-semibold w-36">IP Address</th>
                <th className="text-left p-3 font-semibold">Client</th>
                <th className="text-center p-3 font-semibold w-20">Flags</th>
                <th className="text-right p-3 font-semibold w-24">Progress</th>
                <th className="text-right p-3 font-semibold w-28">Down Speed</th>
                <th className="text-right p-3 font-semibold w-28">Up Speed</th>
                <th className="text-right p-3 font-semibold w-28">Downloaded</th>
                <th className="text-right p-3 font-semibold w-28">Uploaded</th>
              </tr>
            </thead>
            <tbody>
              {peers.map((peer, index) => (
                <tr
                  key={index}
                  className="border-b border-dark-border hover:bg-dark-elevated transition-colors text-sm"
                >
                  <td className="p-3">
                    <div className="flex items-center gap-2">
                      {peer.country && (
                        <span className="text-lg" title={peer.country}>
                          {getCountryFlag(peer.country)}
                        </span>
                      )}
                      <span className="font-mono text-xs text-white">
                        {peer.ip}:{peer.port}
                      </span>
                    </div>
                  </td>
                  <td className="p-3 text-gray-300">{peer.client}</td>
                  <td className="p-3 text-center">
                    <div className="flex justify-center gap-1" title={getFlagDescription(peer.flags)}>
                      {peer.flags.split("").map((flag, i) => (
                        <span key={i} className={`font-semibold ${getFlagColor(flag)}`}>
                          {flag}
                        </span>
                      ))}
                    </div>
                  </td>
                  <td className="p-3 text-right">
                    <div className="flex flex-col items-end gap-1">
                      <span className={peer.progress >= 100 ? "text-success" : "text-gray-300"}>
                        {peer.progress.toFixed(1)}%
                      </span>
                      <div className="w-16 bg-dark-border rounded-full h-1 overflow-hidden">
                        <div
                          className={`h-full rounded-full ${
                            peer.progress >= 100 ? "bg-success" : "bg-primary"
                          }`}
                          style={{ width: `${Math.min(peer.progress, 100)}%` }}
                        />
                      </div>
                    </div>
                  </td>
                  <td className="p-3 text-right text-primary">
                    {peer.downloadSpeed > 0 ? formatSpeed(peer.downloadSpeed) : "-"}
                  </td>
                  <td className="p-3 text-right text-success">
                    {peer.uploadSpeed > 0 ? formatSpeed(peer.uploadSpeed) : "-"}
                  </td>
                  <td className="p-3 text-right text-gray-300">
                    {peer.downloaded > 0 ? formatBytes(peer.downloaded) : "-"}
                  </td>
                  <td className="p-3 text-right text-gray-300">
                    {peer.uploaded > 0 ? formatBytes(peer.uploaded) : "-"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Legend */}
      <div className="p-3 border-t border-dark-border bg-dark-secondary">
        <div className="flex flex-wrap gap-4 text-xs">
          <div className="flex items-center gap-1">
            <span className="text-primary font-semibold">D</span>
            <span className="text-gray-400">= Downloading</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-success font-semibold">U</span>
            <span className="text-gray-400">= Uploading</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-gray-400 font-semibold">I</span>
            <span className="text-gray-400">= Interested</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-warning font-semibold">O</span>
            <span className="text-gray-400">= Optimistic</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-error font-semibold">S</span>
            <span className="text-gray-400">= Snubbed</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-info font-semibold">E</span>
            <span className="text-gray-400">= Encrypted</span>
          </div>
        </div>
      </div>
    </div>
  );
}

function getCountryFlag(countryCode: string): string {
  const flags: { [key: string]: string } = {
    US: "🇺🇸",
    CA: "🇨🇦",
    GB: "🇬🇧",
    DE: "🇩🇪",
    FR: "🇫🇷",
    JP: "🇯🇵",
    CN: "🇨🇳",
    AU: "🇦🇺",
    BR: "🇧🇷",
    IN: "🇮🇳",
  };
  return flags[countryCode] || "🌐";
}
