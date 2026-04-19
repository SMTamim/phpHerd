import { useEffect } from "react";
import {
  CheckCircle,
  Download,
  Circle,
  Loader2,
} from "lucide-react";
import { usePhpStore, type PhpVersion, type InstallProgress } from "../stores/php";
import {
  getPhpVersions,
  installPhpVersion,
  switchPhpVersion,
  listenToEvent,
} from "../lib/tauri";
import toast from "react-hot-toast";

function ProgressBar({ progress }: { progress: InstallProgress }) {
  return (
    <div className="mt-3">
      <div className="flex items-center gap-2 mb-1">
        <Loader2 className="w-3 h-3 animate-spin text-primary" />
        <span className="text-xs text-text-secondary">{progress.message}</span>
      </div>
      <div className="w-full h-2 bg-gray-100 rounded-full overflow-hidden">
        <div
          className="h-full bg-primary rounded-full transition-all duration-300"
          style={{ width: `${progress.progress}%` }}
        />
      </div>
    </div>
  );
}

function PhpVersionCard({ version }: { version: PhpVersion }) {
  const installing = usePhpStore((s) => s.installing[version.version]);
  const setInstallProgress = usePhpStore((s) => s.setInstallProgress);
  const storeSwitch = usePhpStore((s) => s.switchVersion);
  const isInstalling = !!installing && installing.stage !== "complete" && installing.stage !== "error";

  const handleInstall = async () => {
    setInstallProgress(version.version, {
      version: version.version,
      stage: "downloading",
      progress: 0,
      message: "Starting download...",
    });

    try {
      await installPhpVersion(version.version);
      toast.success(`PHP ${version.version} installed!`);
      // Clear progress and refresh version list
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
      toast.error(`Failed to install PHP ${version.version}: ${message}`);
    }
  };

  const handleSwitch = async () => {
    try {
      await switchPhpVersion(version.version);
      storeSwitch(version.version);
      toast.success(`Switched to PHP ${version.version}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error(`Failed to switch: ${message}`);
    }
  };

  return (
    <div
      className={`bg-surface rounded-xl border p-5 animate-fade-in transition-colors ${
        version.isActive
          ? "border-primary shadow-sm shadow-primary/10"
          : "border-border hover:border-primary/30"
      }`}
    >
      <div className="flex items-center justify-between mb-1">
        <div className="flex items-center gap-3">
          <span className="text-xl font-bold text-text-primary">
            PHP {version.version}
          </span>
          {version.isActive && (
            <span className="px-2 py-0.5 text-xs rounded-full bg-primary text-white font-medium">
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
          v{version.fullVersion}
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
            <>
              <button
                onClick={handleSwitch}
                disabled={version.isActive}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  version.isActive
                    ? "bg-gray-100 text-gray-400 cursor-not-allowed"
                    : "bg-primary text-white hover:bg-primary-hover"
                }`}
              >
                {version.isActive ? "Currently Active" : "Switch to This"}
              </button>
            </>
          ) : (
            <button
              onClick={handleInstall}
              className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border text-sm font-medium text-text-primary hover:border-primary hover:bg-primary-light/50 transition-colors"
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
    const versions = await getPhpVersions();
    usePhpStore.getState().setVersions(
      versions.map((v) => ({
        version: v.version,
        fullVersion: v.full_version,
        path: v.path,
        isActive: v.is_active,
        isInstalled: v.is_installed,
      }))
    );
  } catch {
    // Not in Tauri environment
  }
}

export default function PhpVersions() {
  const versions = usePhpStore((s) => s.versions);
  const currentVersion = usePhpStore((s) => s.currentVersion);
  const setInstallProgress = usePhpStore((s) => s.setInstallProgress);

  useEffect(() => {
    refreshVersions();

    // Listen for progress events from the Rust backend
    let unlisten: (() => void) | null = null;

    listenToEvent<InstallProgress>("php-install-progress", (payload) => {
      setInstallProgress(payload.version, payload);
      if (payload.stage === "complete") {
        // Refresh after a short delay to let the filesystem settle
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
        <h1 className="text-2xl font-bold text-text-primary">PHP Versions</h1>
        <p className="text-text-secondary mt-1">
          Manage installed PHP versions. Current:{" "}
          <span className="font-medium text-primary">
            {currentVersion ? `PHP ${currentVersion}` : "None"}
          </span>
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {versions.map((version) => (
          <PhpVersionCard key={version.version} version={version} />
        ))}
      </div>
    </div>
  );
}
