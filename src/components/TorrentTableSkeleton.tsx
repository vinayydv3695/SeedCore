export function TorrentTableSkeleton() {
  return (
    <div className="flex flex-col h-full bg-dark-tertiary rounded-lg overflow-hidden">
      {/* Table header */}
      <div className="bg-dark-secondary border-b border-dark-border">
        <table className="w-full">
          <thead>
            <tr className="text-xs text-gray-400 uppercase">
              <th className="w-8 p-2">
                <div className="w-4 h-4 bg-dark-elevated rounded animate-pulse" />
              </th>
              <th className="text-left p-2">
                <div className="w-16 h-3 bg-dark-elevated rounded animate-pulse" />
              </th>
              <th className="text-right p-2 w-24">
                <div className="w-10 h-3 bg-dark-elevated rounded animate-pulse ml-auto" />
              </th>
              <th className="text-right p-2 w-32">
                <div className="w-16 h-3 bg-dark-elevated rounded animate-pulse ml-auto" />
              </th>
              <th className="text-center p-2 w-24">
                <div className="w-12 h-3 bg-dark-elevated rounded animate-pulse mx-auto" />
              </th>
              <th className="text-right p-2 w-28">
                <div className="w-10 h-3 bg-dark-elevated rounded animate-pulse ml-auto" />
              </th>
              <th className="text-right p-2 w-28">
                <div className="w-8 h-3 bg-dark-elevated rounded animate-pulse ml-auto" />
              </th>
              <th className="text-right p-2 w-24">
                <div className="w-10 h-3 bg-dark-elevated rounded animate-pulse ml-auto" />
              </th>
              <th className="text-right p-2 w-20">
                <div className="w-12 h-3 bg-dark-elevated rounded animate-pulse ml-auto" />
              </th>
              <th className="text-center p-2 w-20">
                <div className="w-12 h-3 bg-dark-elevated rounded animate-pulse mx-auto" />
              </th>
            </tr>
          </thead>
        </table>
      </div>

      {/* Table body skeleton */}
      <div className="flex-1 overflow-hidden">
        <table className="w-full">
          <tbody>
            {Array.from({ length: 8 }).map((_, i) => (
              <tr key={i} className="border-b border-dark-border">
                <td className="p-2 w-8">
                  <div className="w-4 h-4 bg-dark-elevated rounded animate-pulse" />
                </td>
                <td className="p-2">
                  <div
                    className="h-4 bg-dark-elevated rounded animate-pulse"
                    style={{
                      width: `${Math.random() * 200 + 150}px`,
                      animationDelay: `${i * 100}ms`,
                    }}
                  />
                </td>
                <td className="p-2 w-24">
                  <div
                    className="w-16 h-4 bg-dark-elevated rounded animate-pulse ml-auto"
                    style={{ animationDelay: `${i * 100}ms` }}
                  />
                </td>
                <td className="p-2 w-32">
                  <div
                    className="w-full h-2 bg-dark-elevated rounded-full animate-pulse"
                    style={{ animationDelay: `${i * 100}ms` }}
                  />
                </td>
                <td className="p-2 w-24">
                  <div
                    className="w-20 h-6 bg-dark-elevated rounded-full animate-pulse mx-auto"
                    style={{ animationDelay: `${i * 100}ms` }}
                  />
                </td>
                <td className="p-2 w-28">
                  <div
                    className="w-16 h-4 bg-dark-elevated rounded animate-pulse ml-auto"
                    style={{ animationDelay: `${i * 100}ms` }}
                  />
                </td>
                <td className="p-2 w-28">
                  <div
                    className="w-16 h-4 bg-dark-elevated rounded animate-pulse ml-auto"
                    style={{ animationDelay: `${i * 100}ms` }}
                  />
                </td>
                <td className="p-2 w-24">
                  <div
                    className="w-12 h-4 bg-dark-elevated rounded animate-pulse ml-auto"
                    style={{ animationDelay: `${i * 100}ms` }}
                  />
                </td>
                <td className="p-2 w-20">
                  <div
                    className="w-10 h-4 bg-dark-elevated rounded animate-pulse ml-auto"
                    style={{ animationDelay: `${i * 100}ms` }}
                  />
                </td>
                <td className="p-2 w-20">
                  <div
                    className="w-8 h-4 bg-dark-elevated rounded animate-pulse mx-auto"
                    style={{ animationDelay: `${i * 100}ms` }}
                  />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
