import { CheckCircle, Download, Circle, Hexagon } from "lucide-react";
import { useNodeStore, type NodeVersion } from "../stores/node";

function NodeVersionCard({ version }: { version: NodeVersion }) {
  const switchVersion = useNodeStore((s) => s.switchVersion);

  return (
    <div
      className={`bg-surface rounded-xl border p-5 animate-fade-in transition-colors ${
        version.isActive
          ? "border-success shadow-sm shadow-success/10"
          : "border-border hover:border-success/30"
      }`}
    >
      <div className="flex items-center justify-between mb-3">
        <span className="text-xl font-bold text-text-primary">
          Node {version.version}
        </span>
        <div className="flex items-center gap-2">
          {version.isActive && (
            <span className="px-2 py-0.5 text-xs rounded-full bg-success text-white font-medium">
              Active
            </span>
          )}
          {version.isInstalled ? (
            <CheckCircle className="w-5 h-5 text-success" />
          ) : (
            <Circle className="w-5 h-5 text-text-muted" />
          )}
        </div>
      </div>

      <div className="flex items-center gap-2 mt-4">
        {version.isInstalled ? (
          <button
            onClick={() => switchVersion(version.version)}
            disabled={version.isActive}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              version.isActive
                ? "bg-gray-100 text-gray-400 cursor-not-allowed"
                : "bg-success text-white hover:bg-green-600"
            }`}
          >
            {version.isActive ? "Currently Active" : "Switch to This"}
          </button>
        ) : (
          <button className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border text-sm font-medium text-text-primary hover:border-success transition-colors">
            <Download className="w-4 h-4" />
            Install
          </button>
        )}
      </div>
    </div>
  );
}

export default function NodeVersions() {
  const versions = useNodeStore((s) => s.versions);
  const currentVersion = useNodeStore((s) => s.currentVersion);

  return (
    <div>
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-text-primary">
          Node.js Versions
        </h1>
        <p className="text-text-secondary mt-1">
          Manage installed Node.js versions. Current:{" "}
          <span className="font-medium text-success">
            {currentVersion ? `Node ${currentVersion}` : "None"}
          </span>
        </p>
      </div>

      {versions.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {versions.map((version) => (
            <NodeVersionCard key={version.version} version={version} />
          ))}
        </div>
      ) : (
        <div className="text-center py-16">
          <Hexagon className="w-12 h-12 text-text-muted mx-auto mb-4" />
          <h3 className="text-lg font-medium text-text-primary mb-2">
            No Node.js versions installed
          </h3>
          <p className="text-text-secondary">
            Install a Node.js version to get started
          </p>
        </div>
      )}
    </div>
  );
}
