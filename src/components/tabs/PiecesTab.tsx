import { TorrentInfo } from "../../types";

interface PiecesTabProps {
  torrent: TorrentInfo;
}

export function PiecesTab({ torrent }: PiecesTabProps) {
  // Mock pieces data - will be replaced with real data in Phase 8C
  const totalPieces = 512; // Example: 512 pieces
  const pieceLength = Math.floor(torrent.size / totalPieces);
  
  // Generate mock bitfield (0=missing, 1=have, 2=downloading)
  const progress = torrent.size > 0 ? torrent.downloaded / torrent.size : 0;
  const piecesHave = Math.floor(totalPieces * progress);
  const piecesDownloading = Math.min(5, totalPieces - piecesHave); // Mock 5 downloading
  const piecesMissing = totalPieces - piecesHave - piecesDownloading;

  const bitfield = Array(totalPieces)
    .fill(0)
    .map((_, i) => {
      if (i < piecesHave) return 1; // Have
      if (i < piecesHave + piecesDownloading) return 2; // Downloading
      return 0; // Missing
    });

  // Mock availability (how many peers have each piece)
  const availability = Array(totalPieces)
    .fill(0)
    .map(() => Math.floor(Math.random() * 10));

  const getPieceColor = (state: number, avail: number) => {
    if (state === 1) return "bg-success"; // Have
    if (state === 2) return "bg-warning"; // Downloading
    
    // Missing pieces - color by availability
    if (avail >= 5) return "bg-gray-600";
    if (avail >= 2) return "bg-gray-700";
    return "bg-gray-800"; // Rare
  };

  const getPieceTitle = (index: number, state: number, avail: number) => {
    const stateStr = state === 1 ? "Have" : state === 2 ? "Downloading" : "Missing";
    return `Piece ${index}\nState: ${stateStr}\nAvailability: ${avail} peers`;
  };

  // Calculate grid dimensions for roughly square layout
  const columns = Math.ceil(Math.sqrt(totalPieces));

  return (
    <div className="flex flex-col h-full p-4">
      {/* Stats */}
      <div className="grid grid-cols-4 gap-4 mb-4">
        <StatCard
          label="Total Pieces"
          value={totalPieces.toLocaleString()}
          color="text-gray-300"
        />
        <StatCard
          label="Have"
          value={piecesHave.toLocaleString()}
          color="text-success"
          percentage={(piecesHave / totalPieces) * 100}
        />
        <StatCard
          label="Downloading"
          value={piecesDownloading.toLocaleString()}
          color="text-warning"
          percentage={(piecesDownloading / totalPieces) * 100}
        />
        <StatCard
          label="Missing"
          value={piecesMissing.toLocaleString()}
          color="text-error"
          percentage={(piecesMissing / totalPieces) * 100}
        />
      </div>

      {/* Piece info */}
      <div className="mb-4 p-3 bg-dark-secondary rounded-lg">
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div className="flex justify-between">
            <span className="text-gray-400">Piece size:</span>
            <span className="text-white">{(pieceLength / 1024).toFixed(1)} KB</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-400">Last piece size:</span>
            <span className="text-white">
              {((torrent.size % pieceLength) / 1024 || pieceLength / 1024).toFixed(1)} KB
            </span>
          </div>
        </div>
      </div>

      {/* Pieces map */}
      <div className="flex-1 overflow-auto custom-scrollbar">
        <div
          className="grid gap-0.5"
          style={{
            gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
          }}
        >
          {bitfield.map((state, index) => (
            <div
              key={index}
              className={`aspect-square rounded-sm transition-colors cursor-pointer hover:opacity-80 ${getPieceColor(
                state,
                availability[index]
              )}`}
              title={getPieceTitle(index, state, availability[index])}
            />
          ))}
        </div>
      </div>

      {/* Legend */}
      <div className="mt-4 p-3 border-t border-dark-border">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <h4 className="text-xs font-semibold text-gray-400 uppercase mb-2">
              Piece State
            </h4>
            <div className="space-y-1.5">
              <LegendItem color="bg-success" label="Have" />
              <LegendItem color="bg-warning" label="Downloading" />
              <LegendItem color="bg-gray-600" label="Missing (High availability)" />
              <LegendItem color="bg-gray-700" label="Missing (Medium availability)" />
              <LegendItem color="bg-gray-800" label="Missing (Low availability)" />
            </div>
          </div>
          <div>
            <h4 className="text-xs font-semibold text-gray-400 uppercase mb-2">
              Download Strategy
            </h4>
            <div className="text-sm text-gray-300 space-y-1">
              <p>Current: Rarest First</p>
              <p className="text-xs text-gray-500">
                Prioritizes rare pieces to improve swarm health
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

interface StatCardProps {
  label: string;
  value: string;
  color: string;
  percentage?: number;
}

function StatCard({ label, value, color, percentage }: StatCardProps) {
  return (
    <div className="bg-dark-secondary rounded-lg p-3">
      <div className="text-xs text-gray-400 mb-1">{label}</div>
      <div className={`text-2xl font-bold ${color}`}>{value}</div>
      {percentage !== undefined && (
        <div className="text-xs text-gray-500 mt-1">{percentage.toFixed(1)}%</div>
      )}
    </div>
  );
}

interface LegendItemProps {
  color: string;
  label: string;
}

function LegendItem({ color, label }: LegendItemProps) {
  return (
    <div className="flex items-center gap-2 text-sm">
      <div className={`w-4 h-4 rounded-sm ${color}`} />
      <span className="text-gray-300">{label}</span>
    </div>
  );
}
