import { TorrentInfo } from "../../types";
import { cn } from "../../lib/utils";
import { Info } from "lucide-react";

interface PiecesTabProps {
  torrent: TorrentInfo;
}

export function PiecesTab({ torrent }: PiecesTabProps) {
  // Mock pieces data
  const totalPieces = 512;
  const pieceLength = Math.floor(torrent.size / totalPieces);

  const progress = torrent.size > 0 ? torrent.downloaded / torrent.size : 0;
  const piecesHave = Math.floor(totalPieces * progress);
  const piecesDownloading = Math.min(5, totalPieces - piecesHave);
  const piecesMissing = totalPieces - piecesHave - piecesDownloading;

  const bitfield = Array(totalPieces).fill(0).map((_, i) => {
    if (i < piecesHave) return 1; // Have
    if (i < piecesHave + piecesDownloading) return 2; // Downloading
    return 0; // Missing
  });

  const availability = Array(totalPieces).fill(0).map(() => Math.floor(Math.random() * 10));

  const getPieceColor = (state: number, avail: number) => {
    if (state === 1) return "bg-success";
    if (state === 2) return "bg-warning"; // Downloading
    if (avail >= 5) return "bg-white/20"; // High avail
    if (avail >= 2) return "bg-white/10"; // Med avail
    return "bg-white/5"; // Low avail
  };

  const getPieceTitle = (index: number, state: number, avail: number) => {
    const stateStr = state === 1 ? "Have" : state === 2 ? "Downloading" : "Missing";
    return `Piece ${index}\nState: ${stateStr}\nAvailability: ${avail} peers`;
  };

  return (
    <div className="flex flex-col h-full p-1 space-y-4">
      {/* Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <StatCard label="Total Pieces" value={totalPieces.toLocaleString()} />
        <StatCard label="Have" value={piecesHave.toLocaleString()} valueColor="text-success" subValue={`${((piecesHave / totalPieces) * 100).toFixed(1)}%`} />
        <StatCard label="Downloading" value={piecesDownloading.toLocaleString()} valueColor="text-warning" subValue={`${((piecesDownloading / totalPieces) * 100).toFixed(1)}%`} />
        <StatCard label="Missing" value={piecesMissing.toLocaleString()} valueColor="text-error" subValue={`${((piecesMissing / totalPieces) * 100).toFixed(1)}%`} />
      </div>

      {/* Info Bar */}
      <div className="flex items-center justify-between text-xs text-text-secondary bg-dark-surface-elevated/50 p-2 rounded border border-dark-border/50">
        <div className="flex gap-4">
          <span>Piece Size: <span className="text-text-primary font-mono">{(pieceLength / 1024).toFixed(0)} KB</span></span>
          <span>Last Piece: <span className="text-text-primary font-mono">{((torrent.size % pieceLength || pieceLength) / 1024).toFixed(0)} KB</span></span>
        </div>
        <div className="flex items-center gap-1.5 opacity-60">
          <Info className="h-3 w-3" />
          <span>Rarest First Strategy</span>
        </div>
      </div>

      {/* Grid */}
      <div className="flex-1 overflow-y-auto custom-scrollbar bg-dark-surface-elevated/30 rounded-lg p-3 border border-dark-border/30">
        <div
          className="grid gap-[2px]"
          style={{ gridTemplateColumns: `repeat(auto-fill, minmax(10px, 1fr))` }}
        >
          {bitfield.map((state, index) => (
            <div
              key={index}
              className={cn(
                "aspect-square rounded-[1px] cursor-pointer hover:scale-125 transition-transform hover:z-10 hover:shadow-sm",
                getPieceColor(state, availability[index])
              )}
              title={getPieceTitle(index, state, availability[index])}
            />
          ))}
        </div>
      </div>

      {/* Legend */}
      <div className="flex flex-wrap gap-4 text-[10px] text-text-secondary pt-2 border-t border-dark-border/50">
        <LegendItem color="bg-success" label="Have" />
        <LegendItem color="bg-warning" label="Downloading" />
        <LegendItem color="bg-white/20" label="Missing (High Avail)" />
        <LegendItem color="bg-white/10" label="Missing (Med Avail)" />
        <LegendItem color="bg-white/5" label="Missing (Low Avail)" />
      </div>
    </div>
  );
}

function StatCard({ label, value, valueColor = "text-text-primary", subValue }: { label: string, value: string, valueColor?: string, subValue?: string }) {
  return (
    <div className="bg-dark-surface-elevated p-3 rounded-lg border border-dark-border/50">
      <div className="text-xs text-text-tertiary uppercase tracking-wider mb-1">{label}</div>
      <div className="flex items-baseline gap-2">
        <span className={cn("text-xl font-bold", valueColor)}>{value}</span>
        {subValue && <span className="text-xs text-text-tertiary">{subValue}</span>}
      </div>
    </div>
  )
}

function LegendItem({ color, label }: { color: string, label: string }) {
  return (
    <div className="flex items-center gap-1.5">
      <div className={cn("w-2.5 h-2.5 rounded-sm", color)} />
      <span>{label}</span>
    </div>
  )
}
