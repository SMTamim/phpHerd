import { useEffect, useState } from "react";
import {
  Database,
  Plus,
  Play,
  Square,
  Trash2,
  CheckCircle,
  XCircle,
  X,
  Loader2,
} from "lucide-react";
import { useServicesStore, type ServiceInstance } from "../stores/services";
import {
  getServices,
  getAvailableServices,
  createService,
  startService as tauriStartService,
  stopService as tauriStopService,
  deleteService as tauriDeleteService,
  type ServiceInfoData,
  type AvailableServiceData,
  listenToEvent,
} from "../lib/tauri";
import toast from "react-hot-toast";

function mapService(s: ServiceInfoData): ServiceInstance {
  return {
    id: s.id,
    serviceType: s.service_type,
    version: s.version,
    port: s.port,
    status: s.status === "Running" ? "Running" : "Stopped",
    dataDir: s.data_dir,
  };
}

async function refreshServices() {
  try {
    const services = await getServices();
    useServicesStore.getState().setServices(services.map(mapService));
  } catch {
    // ignore
  }
}

function ServiceCard({ service }: { service: ServiceInstance }) {
  const [loading, setLoading] = useState(false);
  const isRunning = service.status === "Running";

  const handleStart = async () => {
    setLoading(true);
    try {
      await tauriStartService(service.id);
      toast.success(`${service.serviceType} started`);
      await refreshServices();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleStop = async () => {
    setLoading(true);
    try {
      await tauriStopService(service.id);
      toast.success(`${service.serviceType} stopped`);
      await refreshServices();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    setLoading(true);
    try {
      await tauriDeleteService(service.id);
      toast.success(`${service.serviceType} deleted`);
      await refreshServices();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="bg-surface rounded-xl border border-border p-5 animate-fade-in">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-primary-light">
            <Database className="w-4 h-4 text-primary" />
          </div>
          <div>
            <h3 className="font-semibold text-text-primary capitalize">
              {service.serviceType}
            </h3>
            <p className="text-xs text-text-secondary">v{service.version}</p>
          </div>
        </div>
        <div className="flex items-center gap-1.5">
          {isRunning ? (
            <CheckCircle className="w-4 h-4 text-success" />
          ) : (
            <XCircle className="w-4 h-4 text-text-muted" />
          )}
          <span
            className={`text-xs font-medium ${isRunning ? "text-success" : "text-text-muted"}`}
          >
            {service.status}
          </span>
        </div>
      </div>

      <div className="text-sm text-text-secondary mb-1">
        Port: <span className="font-mono font-medium">{service.port}</span>
      </div>
      <p className="text-xs text-text-muted truncate mb-4" title={service.dataDir}>
        {service.dataDir}
      </p>

      <div className="flex items-center gap-2">
        {isRunning ? (
          <button
            onClick={handleStop}
            disabled={loading}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-danger/10 text-danger text-xs font-medium hover:bg-danger/20 transition-colors disabled:opacity-50"
          >
            {loading ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              <Square className="w-3 h-3" />
            )}
            Stop
          </button>
        ) : (
          <button
            onClick={handleStart}
            disabled={loading}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-success/10 text-success text-xs font-medium hover:bg-success/20 transition-colors disabled:opacity-50"
          >
            {loading ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              <Play className="w-3 h-3" />
            )}
            Start
          </button>
        )}
        <button
          onClick={handleDelete}
          disabled={loading}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-text-secondary text-xs font-medium hover:bg-gray-100 transition-colors disabled:opacity-50"
        >
          <Trash2 className="w-3 h-3" />
          Delete
        </button>
      </div>
    </div>
  );
}

function CreateServicePanel({ onClose }: { onClose: () => void }) {
  const [available, setAvailable] = useState<AvailableServiceData[]>([]);
  const [selected, setSelected] = useState<AvailableServiceData | null>(null);
  const [version, setVersion] = useState("");
  const [port, setPort] = useState("");
  const [creating, setCreating] = useState(false);
  const [progressMsg, setProgressMsg] = useState("");

  useEffect(() => {
    getAvailableServices()
      .then((data) => setAvailable(data))
      .catch(() => {});

    let unlisten: (() => void) | null = null;
    listenToEvent<{
      service_type: string;
      version: string;
      stage: string;
      progress: number;
      message: string;
    }>("service-download-progress", (payload) => {
      setProgressMsg(payload.message);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const handleSelect = (svc: AvailableServiceData) => {
    setSelected(svc);
    setVersion(svc.versions[svc.versions.length - 1]);
    setPort(String(svc.default_port));
  };

  const handleCreate = async () => {
    if (!selected || !version) return;
    setCreating(true);
    setProgressMsg("Preparing...");
    try {
      await createService({
        service_type: selected.service_type,
        version,
        port: port ? Number(port) : undefined,
      });
      toast.success(`${selected.display_name} v${version} installed and ready!`);
      setProgressMsg("");
      onClose();
      refreshServices();
    } catch (err) {
      toast.error(String(err));
      setProgressMsg("");
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="mb-8 p-6 bg-surface rounded-xl border border-primary animate-fade-in">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-semibold text-text-primary">
          {selected ? `Configure ${selected.display_name}` : "Choose a service to add"}
        </h3>
        <button
          onClick={onClose}
          className="text-text-muted hover:text-text-primary"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {!selected ? (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {available.map((svc) => (
            <button
              key={svc.service_type}
              onClick={() => handleSelect(svc)}
              className="flex flex-col items-center gap-2 p-4 rounded-lg border border-border hover:border-primary hover:bg-primary-light/50 transition-colors"
            >
              <Database className="w-6 h-6 text-primary" />
              <span className="text-sm font-medium text-text-primary">
                {svc.display_name}
              </span>
              <span className="text-xs text-text-muted">
                Port {svc.default_port}
              </span>
            </button>
          ))}
        </div>
      ) : (
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-medium text-text-secondary mb-1">
                Version
              </label>
              <select
                value={version}
                onChange={(e) => setVersion(e.target.value)}
                className="w-full px-3 py-2 rounded-lg border border-border bg-white text-sm text-text-primary focus:outline-none focus:border-primary"
              >
                {selected.versions.map((v) => (
                  <option key={v} value={v}>
                    {v}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-xs font-medium text-text-secondary mb-1">
                Port
              </label>
              <input
                type="number"
                value={port}
                onChange={(e) => setPort(e.target.value)}
                className="w-full px-3 py-2 rounded-lg border border-border bg-white text-sm text-text-primary focus:outline-none focus:border-primary"
              />
            </div>
          </div>
          <div className="flex justify-end gap-2">
            <button
              onClick={() => setSelected(null)}
              className="px-4 py-2 rounded-lg text-sm text-text-secondary hover:bg-gray-100 transition-colors"
            >
              Back
            </button>
            <button
              onClick={handleCreate}
              disabled={creating}
              className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover transition-colors disabled:opacity-50"
            >
              {creating ? (
                <span className="flex items-center gap-2">
                  <Loader2 className="w-3 h-3 animate-spin" />
                  {progressMsg || "Creating..."}
                </span>
              ) : (
                `Create ${selected.display_name}`
              )}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default function Services() {
  const services = useServicesStore((s) => s.services);
  const [showCreate, setShowCreate] = useState(false);

  useEffect(() => {
    refreshServices();
  }, []);

  return (
    <div>
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-text-primary">Services</h1>
          <p className="text-text-secondary mt-1">
            Manage databases, caches, and other services
          </p>
        </div>
        <button
          onClick={() => setShowCreate(!showCreate)}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover transition-colors"
        >
          <Plus className="w-4 h-4" />
          Add Service
        </button>
      </div>

      {showCreate && (
        <CreateServicePanel onClose={() => setShowCreate(false)} />
      )}

      {services.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {services.map((service) => (
            <ServiceCard key={service.id} service={service} />
          ))}
        </div>
      ) : (
        <div className="text-center py-16">
          <Database className="w-12 h-12 text-text-muted mx-auto mb-4" />
          <h3 className="text-lg font-medium text-text-primary mb-2">
            No services configured
          </h3>
          <p className="text-text-secondary">
            Add MySQL, Redis, PostgreSQL, or other services
          </p>
        </div>
      )}
    </div>
  );
}
