import { useEffect } from "react";
import { CheckCircle, Download, Circle, Hexagon, Loader2 } from "lucide-react";
import { useNodeStore, type NodeVersion, type InstallProgress } from "../stores/node";
import {
  getNodeVersions,
  installNodeVersion,
  switchNodeVersion,
  listenToEvent,
} from "../lib/tauri";
import toast from "react-hot-toast";

function ProgressBar({ progress }: { progress: InstallProgress }) {
  return (
    <div className="mt-3">
      <div className="flex items-center gap-2 mb-1">
        <Loader2 className="w-3 h-3 animate-spin text-success" />
        <span className="text-xs text-text-secondary">{progress.message}</span>
      </div>
      <div className="w-full h-2 bg-gray-100 rounded-full overflow-hidden">
        <div
          className="h-full bg-success rounded-full transition-all duration-300"
          style={{ width: `${progress.progress}%` }}
        />
      </div>
    </div>
  );
}

function NodeVersionCard({ version }: { version: NodeVersion }) {
  const installing = useNodeStore((s) => s.installing[version.version]);
  const setInstallProgress = useNodeStore((s) => s.setInstallProgress);
  const storeSwitch = useNodeStore((s) => s.switchVersion);
  const isInstalling =
    !!installing && installing.stage !== "complete" && installing.stage !== "error";

  const handleInstall = async () => {
    setInstallProgress(version.version, {
      version: version.version,
      stage: "downloading",
      progress: 0,
      message: "Starting download...",
    });

    try {
      await installNodeVersion(version.version);
      toast.success(`Node.js ${version.version} installed!`);
      setInstallProgress(version.version, null);
      refreshVersions();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setInstallProgress(version.version, {
        version: version.version,
        stage: "error",
        progress: 0,
        message: `Error: ${message}`,
      });
      toast.error(`Failed to install Node.js ${version.version}: ${message}`);
    }
  };

  const handleSwitch = async () => {
    try {
      await switchNodeVersion(version.version);
      storeSwitch(version.version);
      toast.success(`Switched to Node.js ${version.version}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error(`Failed to switch: ${message}`);
    }
  };

  return (
    <div
      className={`bg-surface rounded-xl border p-5 animate-fade-in transition-colors ${
        version.isActive
          ? "border-success shadow-sm shadow-success/10"
          : "border-border hover:border-success/30"
      }`}
    >
      <div className="flex items-center justify-between mb-1">
        <div className="flex items-center gap-3">
          <span className="text-xl font-bold text-text-primary">
            Node {version.version}
          </span>
          {version.isActive && (
            <span className="px-2 py-0.5 text-xs rounded-full bg-success text-white font-medium">
              Active
            </span>
          )}
        </div>
        {version.isInstalled ? (
          <CheckCircle className="w-5 h-5 text-success" />
        ) : (
          <Circle className="w-5 h-5 text-text-muted" />
        )}
      </div>

      {version.fullVersion && (
        <p className="text-xs text-text-muted mb-2 font-mono">
          {version.fullVersion}
        </p>
      )}

      {installing?.stage === "error" && (
        <p className="text-xs text-danger mt-2 mb-2">{installing.message}</p>
      )}

      {isInstalling ? (
        <ProgressBar progress={installing} />
      ) : (
        <div className="flex items-center gap-2 mt-4">
          {version.isInstalled ? (
            <button
              onClick={handleSwitch}
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
            <button
              onClick={handleInstall}
              className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border text-sm font-medium text-text-primary hover:border-success hover:bg-green-50 transition-colors"
            >
              <Download className="w-4 h-4" />
              Install
            </button>
          )}
        </div>
      )}
    </div>
  );
}

async function refreshVersions() {
  try {
    const versions = await getNodeVersions();
    useNodeStore.getState().setVersions(
      versions.map((v) => ({
        version: v.version,
        fullVersion: v.full_version,
        isActive: v.is_active,
        isInstalled: v.is_installed,
        path: v.path,
      }))
    );
  } catch {
    // Not in Tauri environment
  }
}

export default function NodeVersions() {
  const versions = useNodeStore((s) => s.versions);
  const currentVersion = useNodeStore((s) => s.currentVersion);
  const setInstallProgress = useNodeStore((s) => s.setInstallProgress);

  useEffect(() => {
    refreshVersions();

    let unlisten: (() => void) | null = null;
    listenToEvent<InstallProgress>("node-install-progress", (payload) => {
      setInstallProgress(payload.version, payload);
      if (payload.stage === "complete") {
        setTimeout(refreshVersions, 500);
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

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
            Loading Node.js versions...
          </h3>
          <p className="text-text-secondary">
            Available versions will appear here
          </p>
        </div>
      )}
    </div>
  );
}
