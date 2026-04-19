import { useEffect, useState } from "react";
import {
  CheckCircle,
  Download,
  Circle,
  Loader2,
  Settings2,
  X,
} from "lucide-react";
import { usePhpStore, type PhpVersion, type InstallProgress } from "../stores/php";
import {
  getPhpVersions,
  installPhpVersion,
  switchPhpVersion,
  getPhpExtensions,
  togglePhpExtension,
  restartNginx,
  listenToEvent,
} from "../lib/tauri";
import toast from "react-hot-toast";

// Extensions that Laravel requires — shown first and highlighted
const LARAVEL_REQUIRED = new Set([
  "bcmath", "ctype", "curl", "dom", "fileinfo", "filter",
  "gd", "iconv", "intl", "mbstring", "openssl", "pdo",
  "pdo_mysql", "pdo_pgsql", "pdo_sqlite", "phar",
  "session", "tokenizer", "xml", "xmlwriter", "zip",
]);

interface ExtensionInfo {
  name: string;
  enabled: boolean;
}

function ExtensionsModal({
  version,
  onClose,
}: {
  version: string;
  onClose: () => void;
}) {
  const [extensions, setExtensions] = useState<ExtensionInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [toggling, setToggling] = useState<string | null>(null);
  const [filter, setFilter] = useState("");
  const [changed, setChanged] = useState(false);

  const refresh = async () => {
    setLoading(true);
    try {
      const exts = await getPhpExtensions(version);
      setExtensions(exts);
    } catch {
      toast.error("Failed to load extensions");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
  }, [version]);

  const handleToggle = async (ext: ExtensionInfo) => {
    setToggling(ext.name);
    try {
      await togglePhpExtension(version, ext.name, !ext.enabled);
      setExtensions((prev) =>
        prev.map((e) =>
          e.name === ext.name ? { ...e, enabled: !e.enabled } : e
        )
      );
      toast.success(`${ext.name} ${ext.enabled ? "disabled" : "enabled"}`);
      setChanged(true);
    } catch (err) {
      toast.error(String(err));
    } finally {
      setToggling(null);
    }
  };

  const enableAll = async (names: string[]) => {
    for (const name of names) {
      const ext = extensions.find((e) => e.name === name);
      if (ext && !ext.enabled) {
        await handleToggle(ext);
      }
    }
  };

  const filtered = extensions.filter((e) =>
    e.name.toLowerCase().includes(filter.toLowerCase())
  );

  const laravelExts = filtered.filter((e) => LARAVEL_REQUIRED.has(e.name));
  const otherExts = filtered.filter((e) => !LARAVEL_REQUIRED.has(e.name));
  const missingLaravel = laravelExts.filter((e) => !e.enabled);

  const handleClose = async () => {
    if (changed) {
      toast("Restarting PHP-CGI to apply changes...");
      try {
        await restartNginx();
        toast.success("PHP-CGI restarted with new extensions");
      } catch {
        // Nginx may not be running — that's fine
      }
    }
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-2xl shadow-xl w-full max-w-lg max-h-[80vh] flex flex-col animate-fade-in">
        {/* Header */}
        <div className="flex items-center justify-between p-5 border-b border-border">
          <div>
            <h2 className="text-lg font-bold text-text-primary">
              PHP {version} Extensions
            </h2>
            <p className="text-xs text-text-secondary mt-0.5">
              {extensions.filter((e) => e.enabled).length} of{" "}
              {extensions.length} enabled
              {changed && " — restart pending"}
            </p>
          </div>
          <button
            onClick={handleClose}
            className="p-1.5 rounded-lg hover:bg-gray-100 text-text-muted hover:text-text-primary transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Search + Enable Laravel */}
        <div className="p-4 border-b border-border space-y-3">
          <input
            type="text"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder="Search extensions..."
            className="w-full px-3 py-2 rounded-lg border border-border text-sm focus:outline-none focus:border-primary"
          />
          {missingLaravel.length > 0 && (
            <button
              onClick={() =>
                enableAll(missingLaravel.map((e) => e.name))
              }
              className="w-full px-3 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover transition-colors"
            >
              Enable all Laravel required ({missingLaravel.length} missing)
            </button>
          )}
        </div>

        {/* Extension List */}
        <div className="flex-1 overflow-y-auto p-4">
          {loading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-5 h-5 animate-spin text-primary" />
            </div>
          ) : (
            <>
              {/* Laravel Required */}
              {laravelExts.length > 0 && (
                <div className="mb-4">
                  <h3 className="text-xs font-semibold text-text-secondary uppercase tracking-wide mb-2">
                    Laravel Required
                  </h3>
                  <div className="space-y-1">
                    {laravelExts.map((ext) => (
                      <ExtensionRow
                        key={ext.name}
                        ext={ext}
                        toggling={toggling}
                        onToggle={handleToggle}
                        required
                      />
                    ))}
                  </div>
                </div>
              )}

              {/* Other Extensions */}
              {otherExts.length > 0 && (
                <div>
                  <h3 className="text-xs font-semibold text-text-secondary uppercase tracking-wide mb-2">
                    Other Extensions
                  </h3>
                  <div className="space-y-1">
                    {otherExts.map((ext) => (
                      <ExtensionRow
                        key={ext.name}
                        ext={ext}
                        toggling={toggling}
                        onToggle={handleToggle}
                      />
                    ))}
                  </div>
                </div>
              )}

              {filtered.length === 0 && (
                <p className="text-center text-sm text-text-muted py-8">
                  No extensions found
                </p>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function ExtensionRow({
  ext,
  toggling,
  onToggle,
  required,
}: {
  ext: ExtensionInfo;
  toggling: string | null;
  onToggle: (ext: ExtensionInfo) => void;
  required?: boolean;
}) {
  const isToggling = toggling === ext.name;

  return (
    <div className="flex items-center justify-between py-1.5 px-2 rounded-lg hover:bg-gray-50">
      <div className="flex items-center gap-2">
        <span className="text-sm font-mono text-text-primary">{ext.name}</span>
        {required && !ext.enabled && (
          <span className="px-1.5 py-0.5 text-[10px] rounded bg-amber-100 text-amber-700 font-medium">
            missing
          </span>
        )}
      </div>
      <button
        onClick={() => onToggle(ext)}
        disabled={isToggling}
        className={`relative w-10 h-5 rounded-full transition-colors ${
          ext.enabled ? "bg-primary" : "bg-gray-300"
        } ${isToggling ? "opacity-50" : ""}`}
      >
        <span
          className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform ${
            ext.enabled ? "left-[22px]" : "left-0.5"
          }`}
        />
      </button>
    </div>
  );
}

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

function PhpVersionCard({
  version,
  onManageExtensions,
}: {
  version: PhpVersion;
  onManageExtensions: (version: string) => void;
}) {
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
              <button
                onClick={() => onManageExtensions(version.version)}
                className="flex items-center gap-1.5 px-3 py-2 rounded-lg border border-border text-sm text-text-secondary hover:border-primary hover:text-primary transition-colors"
              >
                <Settings2 className="w-4 h-4" />
                Extensions
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
  const [extensionsVersion, setExtensionsVersion] = useState<string | null>(
    null
  );

  useEffect(() => {
    refreshVersions();

    let unlisten: (() => void) | null = null;

    listenToEvent<InstallProgress>("php-install-progress", (payload) => {
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
          <PhpVersionCard
            key={version.version}
            version={version}
            onManageExtensions={setExtensionsVersion}
          />
        ))}
      </div>

      {/* Extensions Modal */}
      {extensionsVersion && (
        <ExtensionsModal
          version={extensionsVersion}
          onClose={() => setExtensionsVersion(null)}
        />
      )}
    </div>
  );
}
