import { useEffect } from "react";
import {
  Globe,
  Code2,
  Database,
  Server,
  Activity,
  CheckCircle,
  XCircle,
} from "lucide-react";
import { useSitesStore } from "../stores/sites";
import { usePhpStore } from "../stores/php";
import { useServicesStore } from "../stores/services";
import { getPhpVersions } from "../lib/tauri";

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
  const sites = useSitesStore((s) => s.sites);
  const currentPhp = usePhpStore((s) => s.currentVersion);
  const setVersions = usePhpStore((s) => s.setVersions);
  const services = useServicesStore((s) => s.services);
  const runningServices = services.filter((s) => s.status === "Running");

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
  }, []);

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
        <StatCard icon={Server} label="Nginx" value="Ready" status="stopped" />
      </div>

      {/* Quick Actions */}
      <div className="bg-surface rounded-xl border border-border p-6">
        <h2 className="text-lg font-semibold text-text-primary mb-4">
          Quick Actions
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <button className="flex items-center gap-3 p-4 rounded-lg border border-border hover:border-primary hover:bg-primary-light/50 transition-colors text-left">
            <Activity className="w-5 h-5 text-primary" />
            <div>
              <p className="text-sm font-medium text-text-primary">
                Start All Services
              </p>
              <p className="text-xs text-text-secondary">
                Start Nginx, PHP-FPM, and DNS
              </p>
            </div>
          </button>
          <button className="flex items-center gap-3 p-4 rounded-lg border border-border hover:border-primary hover:bg-primary-light/50 transition-colors text-left">
            <Globe className="w-5 h-5 text-primary" />
            <div>
              <p className="text-sm font-medium text-text-primary">
                Add New Site
              </p>
              <p className="text-xs text-text-secondary">
                Link or park a project directory
              </p>
            </div>
          </button>
          <button className="flex items-center gap-3 p-4 rounded-lg border border-border hover:border-primary hover:bg-primary-light/50 transition-colors text-left">
            <Database className="w-5 h-5 text-primary" />
            <div>
              <p className="text-sm font-medium text-text-primary">
                Add Service
              </p>
              <p className="text-xs text-text-secondary">
                MySQL, Redis, PostgreSQL, etc.
              </p>
            </div>
          </button>
        </div>
      </div>
    </div>
  );
}
