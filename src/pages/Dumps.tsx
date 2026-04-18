import { Bug, Trash2, Database, Eye, Send, FileCode } from "lucide-react";
import { useDumpsStore, type DumpEntry } from "../stores/dumps";

const typeIcons: Record<string, React.ElementType> = {
  dump: Bug,
  query: Database,
  view: Eye,
  http: Send,
  log: FileCode,
};

function DumpCard({ dump }: { dump: DumpEntry }) {
  const Icon = typeIcons[dump.type] || Bug;

  return (
    <div className="bg-surface rounded-xl border border-border p-4 animate-fade-in">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <Icon className="w-4 h-4 text-primary" />
          <span className="text-xs font-medium text-primary uppercase">
            {dump.type}
          </span>
          {dump.site && (
            <span className="text-xs text-text-muted">{dump.site}</span>
          )}
        </div>
        <span className="text-xs text-text-muted">{dump.timestamp}</span>
      </div>

      <pre className="text-sm text-text-primary bg-surface-secondary rounded-lg p-3 overflow-x-auto font-mono">
        {dump.content}
      </pre>

      {dump.file && (
        <p className="text-xs text-text-muted mt-2">
          {dump.file}
          {dump.line && `:${dump.line}`}
        </p>
      )}
    </div>
  );
}

export default function Dumps() {
  const dumps = useDumpsStore((s) => s.dumps);
  const clearDumps = useDumpsStore((s) => s.clearDumps);

  return (
    <div>
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-text-primary">Dumps</h1>
          <p className="text-text-secondary mt-1">
            Intercepted dump(), dd(), queries, and more
          </p>
        </div>
        {dumps.length > 0 && (
          <button
            onClick={clearDumps}
            className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border text-sm font-medium text-danger hover:bg-danger/5 transition-colors"
          >
            <Trash2 className="w-4 h-4" />
            Clear All
          </button>
        )}
      </div>

      {dumps.length > 0 ? (
        <div className="space-y-3">
          {dumps.map((dump) => (
            <DumpCard key={dump.id} dump={dump} />
          ))}
        </div>
      ) : (
        <div className="text-center py-16">
          <Bug className="w-12 h-12 text-text-muted mx-auto mb-4" />
          <h3 className="text-lg font-medium text-text-primary mb-2">
            No dumps captured
          </h3>
          <p className="text-text-secondary">
            Dumps from your PHP applications will appear here in real-time
          </p>
        </div>
      )}
    </div>
  );
}
