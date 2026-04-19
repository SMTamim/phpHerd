import { useEffect, useState } from "react";
import {
  Globe,
  Code2,
  Database,
  Server,
  CheckCircle,
  XCircle,
  Download,
  Loader2,
  Play,
  Square,
} from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useSitesStore } from "../stores/sites";
import { usePhpStore } from "../stores/php";
import { useServicesStore } from "../stores/services";
import {
  getPhpVersions,
  getNginxStatus,
  installPhpmyadmin,
  listenToEvent,
  installNginx,
  startNginx,
  stopNginx,
  type NginxStatusInfo,
} from "../lib/tauri";
import toast from "react-hot-toast";

function StatCard({
  icon: Icon,
  label,
  value,
  status,
}: {
  icon: React.ElementType;
  label: string;
  value: string | number;
  status?: "running" | "stopped" | "none";
}) {
  return (
    <div className="bg-surface rounded-xl border border-border p-6 animate-fade-in">
      <div className="flex items-center justify-between mb-4">
        <div className="p-2.5 rounded-lg bg-primary-light">
          <Icon className="w-5 h-5 text-primary" />
        </div>
        {status && status !== "none" && (
          <div className="flex items-center gap-1.5">
            {status === "running" ? (
              <CheckCircle className="w-4 h-4 text-success" />
            ) : (
              <XCircle className="w-4 h-4 text-danger" />
            )}
            <span
              className={`text-xs font-medium ${status === "running" ? "text-success" : "text-danger"}`}
            >
              {status === "running" ? "Running" : "Stopped"}
            </span>
          </div>
        )}
      </div>
      <p className="text-2xl font-bold text-text-primary">{value}</p>
      <p className="text-sm text-text-secondary mt-1">{label}</p>
    </div>
  );
}

export default function Dashboard() {
  const navigate = useNavigate();
  const sites = useSitesStore((s) => s.sites);
  const currentPhp = usePhpStore((s) => s.currentVersion);
  const setVersions = usePhpStore((s) => s.setVersions);
  const services = useServicesStore((s) => s.services);
  const runningServices = services.filter((s) => s.status === "Running");

  const [nginx, setNginx] = useState<NginxStatusInfo | null>(null);
  const [nginxLoading, setNginxLoading] = useState(false);
  const [pmaInstalling, setPmaInstalling] = useState(false);
  const [pmaProgress, setPmaProgress] = useState("");

  const refreshNginx = async () => {
    try {
      const status = await getNginxStatus();
      setNginx(status);
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    getPhpVersions()
      .then((versions) =>
        setVersions(
          versions.map((v) => ({
            version: v.version,
            fullVersion: v.full_version,
            path: v.path,
            isActive: v.is_active,
            isInstalled: v.is_installed,
          }))
        )
      )
      .catch(() => {});
    refreshNginx();

    let unlisten: (() => void) | null = null;
    listenToEvent<{ stage: string; progress: number; message: string }>(
      "phpmyadmin-install-progress",
      (payload) => setPmaProgress(payload.message)
    ).then((fn) => { unlisten = fn; });

    return () => { unlisten?.(); };
  }, []);

  const handleNginxToggle = async () => {
    if (!nginx) return;
    setNginxLoading(true);

    try {
      if (!nginx.installed) {
        toast("Installing Nginx...");
        await installNginx();
        toast.success("Nginx installed!");
        await refreshNginx();
      } else if (nginx.running) {
        await stopNginx();
        toast.success("Nginx stopped");
        await refreshNginx();
      } else {
        await startNginx();
        toast.success("Nginx started!");
        await refreshNginx();
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      toast.error(msg);
    } finally {
      setNginxLoading(false);
    }
  };

  const handlePmaInstall = async () => {
    setPmaInstalling(true);
    setPmaProgress("Starting...");
    try {
      await installPhpmyadmin();
      toast.success("phpMyAdmin installed at pma.test!");
      setPmaProgress("");
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      toast.error(msg);
      setPmaProgress("");
    } finally {
      setPmaInstalling(false);
    }
  };

  const nginxStatus = nginx?.running ? "running" : "stopped";
  const nginxValue = !nginx
    ? "..."
    : !nginx.installed
      ? "Not Installed"
      : nginx.running
        ? `Running${nginx.version ? ` (${nginx.version})` : ""}`
        : "Stopped";

  return (
    <div>
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-text-primary">Dashboard</h1>
        <p className="text-text-secondary mt-1">
          Overview of your development environment
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <StatCard icon={Globe} label="Active Sites" value={sites.length} />
        <StatCard icon={Code2} label="PHP Version" value={currentPhp || "N/A"} />
        <StatCard
          icon={Database}
          label="Services"
          value={`${runningServices.length}/${services.length}`}
        />
        <StatCard
          icon={Server}
          label="Nginx"
          value={nginxValue}
          status={nginx ? nginxStatus : "none"}
        />
      </div>

      {/* Nginx Controls */}
      <div className="bg-surface rounded-xl border border-border p-6 mb-8">
        <h2 className="text-lg font-semibold text-text-primary mb-4">
          Nginx Web Server
        </h2>
        <div className="flex items-center gap-4">
          {nginx && !nginx.installed ? (
            <button
              onClick={handleNginxToggle}
              disabled={nginxLoading}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-white hover:bg-primary-hover transition-colors disabled:opacity-50"
            >
              {nginxLoading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Download className="w-4 h-4" />
              )}
              Install Nginx
            </button>
          ) : (
            <>
              <button
                onClick={handleNginxToggle}
                disabled={nginxLoading}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg text-white transition-colors disabled:opacity-50 ${
                  nginx?.running
                    ? "bg-danger hover:bg-red-600"
                    : "bg-success hover:bg-green-600"
                }`}
              >
                {nginxLoading ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : nginx?.running ? (
                  <Square className="w-4 h-4" />
                ) : (
                  <Play className="w-4 h-4" />
                )}
                {nginx?.running ? "Stop" : "Start"}
              </button>
              {nginx?.version && (
                <span className="text-sm text-text-muted font-mono">
                  v{nginx.version}
                </span>
              )}
            </>
          )}
        </div>
      </div>

      {/* Quick Actions */}
      <div className="bg-surface rounded-xl border border-border p-6">
        <h2 className="text-lg font-semibold text-text-primary mb-4">
          Quick Actions
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <button
            onClick={() => navigate("/sites")}
            className="flex items-center gap-3 p-4 rounded-lg border border-border hover:border-primary hover:bg-primary-light/50 transition-colors text-left"
          >
            <Globe className="w-5 h-5 text-primary" />
            <div>
              <p className="text-sm font-medium text-text-primary">
                Manage Sites
              </p>
              <p className="text-xs text-text-secondary">
                Link or park project directories
              </p>
            </div>
          </button>
          <button
            onClick={() => navigate("/php")}
            className="flex items-center gap-3 p-4 rounded-lg border border-border hover:border-primary hover:bg-primary-light/50 transition-colors text-left"
          >
            <Code2 className="w-5 h-5 text-primary" />
            <div>
              <p className="text-sm font-medium text-text-primary">
                PHP Versions
              </p>
              <p className="text-xs text-text-secondary">
                Install and switch PHP versions
              </p>
            </div>
          </button>
          <button
            onClick={() => navigate("/services")}
            className="flex items-center gap-3 p-4 rounded-lg border border-border hover:border-primary hover:bg-primary-light/50 transition-colors text-left"
          >
            <Database className="w-5 h-5 text-primary" />
            <div>
              <p className="text-sm font-medium text-text-primary">
                Services
              </p>
              <p className="text-xs text-text-secondary">
                MySQL, Redis, PostgreSQL, etc.
              </p>
            </div>
          </button>
          <button
            onClick={handlePmaInstall}
            disabled={pmaInstalling}
            className="flex items-center gap-3 p-4 rounded-lg border border-border hover:border-primary hover:bg-primary-light/50 transition-colors text-left disabled:opacity-60"
          >
            {pmaInstalling ? (
              <Loader2 className="w-5 h-5 text-primary animate-spin" />
            ) : (
              <Download className="w-5 h-5 text-primary" />
            )}
            <div>
              <p className="text-sm font-medium text-text-primary">
                {pmaInstalling ? "Installing..." : "phpMyAdmin"}
              </p>
              <p className="text-xs text-text-secondary">
                {pmaInstalling ? pmaProgress : "Install at pma.test"}
              </p>
            </div>
          </button>
        </div>
      </div>
    </div>
  );
}
