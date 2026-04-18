import { useState } from "react";
import { FileText, Search, RefreshCw } from "lucide-react";
import { useLogsStore } from "../stores/logs";

export default function Logs() {
  const logFiles = useLogsStore((s) => s.logFiles);
  const entries = useLogsStore((s) => s.entries);
  const selectedFile = useLogsStore((s) => s.selectedFile);
  const setSelectedFile = useLogsStore((s) => s.setSelectedFile);
  const [search, setSearch] = useState("");

  const filteredEntries = entries.filter((e) =>
    search ? e.message.toLowerCase().includes(search.toLowerCase()) : true
  );

  return (
    <div>
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-text-primary">Logs</h1>
          <p className="text-text-secondary mt-1">
            Browse application log files
          </p>
        </div>
        <button className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border text-sm font-medium text-text-primary hover:bg-surface transition-colors">
          <RefreshCw className="w-4 h-4" />
          Refresh
        </button>
      </div>

      <div className="flex gap-6">
        {/* File list */}
        <div className="w-64 shrink-0">
          <h3 className="text-sm font-medium text-text-primary mb-3">
            Log Files
          </h3>
          <div className="space-y-1">
            {logFiles.length > 0 ? (
              logFiles.map((file) => (
                <button
                  key={file.path}
                  onClick={() => setSelectedFile(file.path)}
                  className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                    selectedFile === file.path
                      ? "bg-primary text-white"
                      : "text-text-secondary hover:bg-surface"
                  }`}
                >
                  {file.name}
                </button>
              ))
            ) : (
              <p className="text-sm text-text-muted">No log files found</p>
            )}
          </div>
        </div>

        {/* Log entries */}
        <div className="flex-1">
          <div className="mb-4 relative">
            <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search logs..."
              className="w-full pl-10 pr-4 py-2 rounded-lg border border-border bg-surface text-sm focus:outline-none focus:border-primary"
            />
          </div>

          {filteredEntries.length > 0 ? (
            <div className="bg-surface rounded-xl border border-border divide-y divide-border overflow-hidden">
              {filteredEntries.map((entry, i) => (
                <div key={i} className="px-4 py-2 text-sm hover:bg-surface-secondary">
                  <div className="flex items-center gap-3">
                    {entry.level && (
                      <span
                        className={`px-1.5 py-0.5 rounded text-xs font-mono font-medium ${
                          entry.level === "ERROR"
                            ? "bg-danger/10 text-danger"
                            : entry.level === "WARNING"
                              ? "bg-warning/10 text-warning"
                              : "bg-info/10 text-info"
                        }`}
                      >
                        {entry.level}
                      </span>
                    )}
                    <span className="text-xs text-text-muted">
                      {entry.timestamp}
                    </span>
                  </div>
                  <p className="text-text-primary mt-1 font-mono text-xs">
                    {entry.message}
                  </p>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-16">
              <FileText className="w-12 h-12 text-text-muted mx-auto mb-4" />
              <h3 className="text-lg font-medium text-text-primary mb-2">
                {selectedFile ? "No entries found" : "Select a log file"}
              </h3>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
