import { useState, useEffect } from "react";
import { TorrentInfo } from "../types";
import { formatSpeed } from "../lib/utils";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";

interface SpeedChartProps {
  torrents: TorrentInfo[];
}

interface DataPoint {
  time: string;
  download: number;
  upload: number;
}

export function SpeedChart({ torrents }: SpeedChartProps) {
  const [data, setData] = useState<DataPoint[]>([]);
  const maxDataPoints = 30; // Keep last 30 data points (1 minute of data at 2s intervals)

  useEffect(() => {
    const now = new Date();
    const timeStr = now.toLocaleTimeString("en-US", { 
      hour12: false, 
      hour: "2-digit", 
      minute: "2-digit", 
      second: "2-digit" 
    });

    const totalDownload = torrents.reduce((sum, t) => sum + t.download_speed, 0);
    const totalUpload = torrents.reduce((sum, t) => sum + t.upload_speed, 0);

    setData((prev) => {
      const newData = [...prev, { time: timeStr, download: totalDownload, upload: totalUpload }];
      
      // Keep only the last maxDataPoints
      if (newData.length > maxDataPoints) {
        return newData.slice(newData.length - maxDataPoints);
      }
      
      return newData;
    });
  }, [torrents]);

  // Custom tooltip
  const CustomTooltip = ({ active, payload }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="rounded-lg border border-dark-border bg-dark-surface p-3 shadow-lg">
          <p className="mb-1 text-xs text-gray-400">{payload[0].payload.time}</p>
          <p className="text-sm font-semibold text-primary">
            Download: {formatSpeed(payload[0].value)}
          </p>
          <p className="text-sm font-semibold text-success">
            Upload: {formatSpeed(payload[1].value)}
          </p>
        </div>
      );
    }
    return null;
  };

  return (
    <div className="rounded-lg border border-dark-border bg-dark-surface p-4">
      <h3 className="mb-4 text-sm font-semibold uppercase tracking-wider text-gray-500">
        Transfer Speed
      </h3>
      
      {data.length > 0 ? (
        <ResponsiveContainer width="100%" height={200}>
          <LineChart data={data}>
            <CartesianGrid strokeDasharray="3 3" stroke="#333" />
            <XAxis 
              dataKey="time" 
              stroke="#666"
              tick={{ fill: '#999', fontSize: 11 }}
              tickFormatter={(value) => {
                // Show only every 5th label to avoid crowding
                const index = data.findIndex(d => d.time === value);
                return index % 5 === 0 ? value.substring(3) : ''; // Show MM:SS
              }}
            />
            <YAxis 
              stroke="#666"
              tick={{ fill: '#999', fontSize: 11 }}
              tickFormatter={(value) => {
                if (value === 0) return '0';
                if (value < 1024) return `${value} B/s`;
                if (value < 1048576) return `${(value / 1024).toFixed(0)} KB/s`;
                return `${(value / 1048576).toFixed(1)} MB/s`;
              }}
            />
            <Tooltip content={<CustomTooltip />} />
            <Line 
              type="monotone" 
              dataKey="download" 
              stroke="#3b82f6" 
              strokeWidth={2}
              dot={false}
              isAnimationActive={false}
            />
            <Line 
              type="monotone" 
              dataKey="upload" 
              stroke="#10b981" 
              strokeWidth={2}
              dot={false}
              isAnimationActive={false}
            />
          </LineChart>
        </ResponsiveContainer>
      ) : (
        <div className="flex h-[200px] items-center justify-center text-sm text-gray-500">
          Waiting for data...
        </div>
      )}
      
      {/* Legend */}
      <div className="mt-4 flex items-center justify-center gap-6 text-sm">
        <div className="flex items-center gap-2">
          <div className="h-3 w-3 rounded-full bg-primary" />
          <span className="text-gray-400">Download</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="h-3 w-3 rounded-full bg-success" />
          <span className="text-gray-400">Upload</span>
        </div>
      </div>
    </div>
  );
}
