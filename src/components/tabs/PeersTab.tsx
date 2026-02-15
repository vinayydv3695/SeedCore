import { TorrentInfo } from "../../types";
import { formatSpeed, formatBytes } from "../../lib/utils";
import { Button } from "../ui/Button";
import { Plus, Ban } from "lucide-react";
import { cn } from "../../lib/utils";

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
  // Mock peer data
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
      case "D": return "text-primary";
      case "U": return "text-success";
      case "O": return "text-warning";
      case "S": return "text-error";
      case "E": return "text-info";
      default: return "text-text-tertiary";
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="mb-4 flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <Button size="sm" variant="primary" leftIcon={<Plus className="h-3.5 w-3.5" />}>
            Add Peer
          </Button>
          <Button size="sm" variant="danger" leftIcon={<Ban className="h-3.5 w-3.5" />}>
            Ban Selected
          </Button>
        </div>
        <div className="flex items-center gap-4 text-xs text-text-tertiary font-medium px-2">
          <span>{peers.length} peer{peers.length !== 1 ? "s" : ""}</span>
          <span>Seeds: {peers.filter((p) => p.progress >= 100).length}</span>
        </div>
      </div>

      {/* Peers table */}
      <div className="bg-dark-surface-elevated border border-dark-border rounded-lg overflow-hidden flex-1">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead className="bg-dark-bg/50 border-b border-dark-border text-xs uppercase text-text-tertiary font-medium">
              <tr>
                <th className="px-4 py-3 w-40">IP Address</th>
                <th className="px-4 py-3">Client</th>
                <th className="px-4 py-3 text-center w-24">Flags</th>
                <th className="px-4 py-3 text-right w-28">Progress</th>
                <th className="px-4 py-3 text-right w-24">Down</th>
                <th className="px-4 py-3 text-right w-24">Up</th>
                <th className="px-4 py-3 text-right w-24">Downloaded</th>
                <th className="px-4 py-3 text-right w-24">Uploaded</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-dark-border">
              {peers.map((peer, index) => (
                <tr key={index} className="hover:bg-dark-surface-hover transition-colors">
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2">
                      {peer.country && (
                        <span className="text-lg leading-none" title={peer.country}>
                          {getCountryFlag(peer.country)}
                        </span>
                      )}
                      <span className="font-mono text-xs text-text-primary">
                        {peer.ip}:{peer.port}
                      </span>
                    </div>
                  </td>
                  <td className="px-4 py-3 text-text-secondary text-xs">{peer.client}</td>
                  <td className="px-4 py-3 text-center">
                    <div className="flex justify-center gap-0.5" title={getFlagDescription(peer.flags)}>
                      {peer.flags.split("").map((flag, i) => (
                        <span key={i} className={cn("font-bold font-mono", getFlagColor(flag))}>
                          {flag}
                        </span>
                      ))}
                    </div>
                  </td>
                  <td className="px-4 py-3 text-right">
                    <div className="flex flex-col items-end gap-1">
                      <span className={cn("text-xs font-medium", peer.progress >= 100 ? "text-success" : "text-text-secondary")}>
                        {peer.progress.toFixed(1)}%
                      </span>
                      <div className="w-16 bg-dark-bg rounded-full h-1 overflow-hidden border border-dark-border/30">
                        <div
                          className={cn("h-full rounded-full", peer.progress >= 100 ? "bg-success" : "bg-primary")}
                          style={{ width: `${Math.min(peer.progress, 100)}%` }}
                        />
                      </div>
                    </div>
                  </td>
                  <td className="px-4 py-3 text-right text-primary font-mono text-xs">
                    {peer.downloadSpeed > 0 ? formatSpeed(peer.downloadSpeed) : "-"}
                  </td>
                  <td className="px-4 py-3 text-right text-success font-mono text-xs">
                    {peer.uploadSpeed > 0 ? formatSpeed(peer.uploadSpeed) : "-"}
                  </td>
                  <td className="px-4 py-3 text-right text-text-tertiary text-xs">
                    {peer.downloaded > 0 ? formatBytes(peer.downloaded) : "-"}
                  </td>
                  <td className="px-4 py-3 text-right text-text-tertiary text-xs">
                    {peer.uploaded > 0 ? formatBytes(peer.uploaded) : "-"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Legend */}
      <div className="mt-4 p-3 border-t border-dark-border bg-dark-surface-elevated/50 rounded-lg">
        <div className="flex flex-wrap gap-4 text-[10px] text-text-secondary">
          <div className="flex items-center gap-1">
            <span className="text-primary font-bold">D</span>
            <span>Downloading</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-success font-bold">U</span>
            <span>Uploading</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-text-tertiary font-bold">I</span>
            <span>Interested</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-warning font-bold">O</span>
            <span>Optimistic</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-error font-bold">S</span>
            <span>Snubbed</span>
          </div>
          <div className="flex items-center gap-1">
            <span className="text-info font-bold">E</span>
            <span>Encrypted</span>
          </div>
        </div>
      </div>
    </div>
  );
}

function getCountryFlag(countryCode: string): string {
  const flags: { [key: string]: string } = {
    US: "🇺🇸", CA: "🇨🇦", GB: "🇬🇧", DE: "🇩🇪", FR: "🇫🇷",
    JP: "🇯🇵", CN: "🇨🇳", AU: "🇦🇺", BR: "🇧🇷", IN: "🇮🇳",
  };
  return flags[countryCode] || "🌐";
}
