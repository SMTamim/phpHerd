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
  Users,
  HardDrive,
} from "lucide-react";
import { useServicesStore, type ServiceInstance } from "../stores/services";
import {
  getServices,
  getAvailableServices,
  createService,
  startService as tauriStartService,
  stopService as tauriStopService,
  deleteService as tauriDeleteService,
  listDbUsers,
  createDbUser,
  dropDbUser,
  listDatabases,
  createDatabase,
  dropDatabase,
  grantDbAccess,
  revokeDbAccess,
  listUserGrants,
  type DbUser,
  type DbName,
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

const DB_TYPES = ["mysql", "mariadb", "postgresql"];

function DatabasePanel({ service }: { service: ServiceInstance }) {
  const [tab, setTab] = useState<"users" | "databases" | "access">("users");
  const [users, setUsers] = useState<DbUser[]>([]);
  const [databases, setDatabases] = useState<DbName[]>([]);
  const [newUser, setNewUser] = useState("");
  const [newPass, setNewPass] = useState("");
  const [newDb, setNewDb] = useState("");
  const [grantUser, setGrantUser] = useState("");
  const [grantDb, setGrantDb] = useState("");
  const [userGrants, setUserGrants] = useState<Record<string, string[]>>({});
  const [loading, setLoading] = useState(false);

  const refresh = async () => {
    try {
      const [u, d] = await Promise.all([
        listDbUsers(service.serviceType, service.version, service.port),
        listDatabases(service.serviceType, service.version, service.port),
      ]);
      setUsers(u);
      setDatabases(d);

      // Load grants for non-system users
      const grants: Record<string, string[]> = {};
      for (const user of u) {
        if (user.username !== "root" && user.username !== "postgres" && user.username !== "mysql.sys" && user.username !== "mysql.session" && user.username !== "mysql.infoschema") {
          try {
            grants[user.username] = await listUserGrants(
              service.serviceType, service.version, service.port, user.username
            );
          } catch {
            grants[user.username] = [];
          }
        }
      }
      setUserGrants(grants);
    } catch (err) {
      toast.error(`Failed to query ${service.serviceType}: ${err}`);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  const handleCreateUser = async () => {
    if (!newUser.trim()) return;
    setLoading(true);
    try {
      await createDbUser({
        service_type: service.serviceType,
        version: service.version,
        port: service.port,
        username: newUser.trim(),
        password: newPass || newUser.trim(),
      });
      toast.success(`User '${newUser}' created`);
      setNewUser("");
      setNewPass("");
      refresh();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleDropUser = async (username: string) => {
    try {
      await dropDbUser({
        service_type: service.serviceType,
        version: service.version,
        port: service.port,
        username,
      });
      toast.success(`User '${username}' dropped`);
      refresh();
    } catch (err) {
      toast.error(String(err));
    }
  };

  const handleCreateDb = async () => {
    if (!newDb.trim()) return;
    setLoading(true);
    try {
      await createDatabase({
        service_type: service.serviceType,
        version: service.version,
        port: service.port,
        db_name: newDb.trim(),
      });
      toast.success(`Database '${newDb}' created`);
      setNewDb("");
      refresh();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleDropDb = async (name: string) => {
    try {
      await dropDatabase(service.serviceType, service.version, service.port, name);
      toast.success(`Database '${name}' dropped`);
      refresh();
    } catch (err) {
      toast.error(String(err));
    }
  };

  const handleGrant = async () => {
    if (!grantUser || !grantDb) return;
    setLoading(true);
    try {
      await grantDbAccess({
        service_type: service.serviceType,
        version: service.version,
        port: service.port,
        username: grantUser,
        db_name: grantDb,
      });
      toast.success(`Granted '${grantUser}' access to '${grantDb}'`);
      setGrantUser("");
      setGrantDb("");
      refresh();
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleRevoke = async (username: string, dbName: string) => {
    try {
      await revokeDbAccess({
        service_type: service.serviceType,
        version: service.version,
        port: service.port,
        username,
        db_name: dbName,
      });
      toast.success(`Revoked '${username}' access from '${dbName}'`);
      refresh();
    } catch (err) {
      toast.error(String(err));
    }
  };

  const systemDbs = ["mysql", "information_schema", "performance_schema", "sys", "postgres", "template0", "template1"];
  const systemUsers = ["root", "postgres", "mysql.sys", "mysql.session", "mysql.infoschema"];
  const nonSystemUsers = users.filter((u) => !systemUsers.includes(u.username));
  const nonSystemDbs = databases.filter((d) => !systemDbs.includes(d.name));

  return (
    <div className="mt-4 pt-4 border-t border-border">
      {/* Tabs */}
      <div className="flex gap-1 mb-3">
        <button
          onClick={() => setTab("users")}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
            tab === "users" ? "bg-primary text-white" : "bg-gray-100 text-text-secondary hover:bg-gray-200"
          }`}
        >
          <Users className="w-3 h-3" />
          Users
        </button>
        <button
          onClick={() => setTab("databases")}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
            tab === "databases" ? "bg-primary text-white" : "bg-gray-100 text-text-secondary hover:bg-gray-200"
          }`}
        >
          <HardDrive className="w-3 h-3" />
          Databases
        </button>
        <button
          onClick={() => setTab("access")}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
            tab === "access" ? "bg-primary text-white" : "bg-gray-100 text-text-secondary hover:bg-gray-200"
          }`}
        >
          <Database className="w-3 h-3" />
          Access
        </button>
      </div>

      {tab === "users" && (
        <div>
          {/* Create user form */}
          <div className="flex gap-2 mb-3">
            <input
              type="text"
              value={newUser}
              onChange={(e) => setNewUser(e.target.value)}
              placeholder="Username"
              className="flex-1 px-2 py-1.5 rounded-lg border border-border text-xs focus:outline-none focus:border-primary"
            />
            <input
              type="text"
              value={newPass}
              onChange={(e) => setNewPass(e.target.value)}
              placeholder="Password"
              className="flex-1 px-2 py-1.5 rounded-lg border border-border text-xs focus:outline-none focus:border-primary"
            />
            <button
              onClick={handleCreateUser}
              disabled={loading || !newUser.trim()}
              className="px-3 py-1.5 rounded-lg bg-primary text-white text-xs font-medium hover:bg-primary-hover disabled:opacity-50"
            >
              {loading ? <Loader2 className="w-3 h-3 animate-spin" /> : "Add"}
            </button>
          </div>
          {/* User list */}
          <div className="space-y-1 max-h-40 overflow-y-auto">
            {users.map((u) => (
              <div
                key={`${u.username}@${u.host}`}
                className="flex items-center justify-between py-1 px-2 rounded hover:bg-gray-50 text-xs"
              >
                <span className="font-mono text-text-primary">
                  {u.username}
                  <span className="text-text-muted">@{u.host}</span>
                </span>
                {u.username !== "root" && u.username !== "postgres" && (
                  <button
                    onClick={() => handleDropUser(u.username)}
                    className="text-danger hover:underline"
                  >
                    Drop
                  </button>
                )}
              </div>
            ))}
            {users.length === 0 && (
              <p className="text-xs text-text-muted py-2">No users found. Is the service running?</p>
            )}
          </div>
        </div>
      )}

      {tab === "databases" && (
        <div>
          {/* Create database form */}
          <div className="flex gap-2 mb-3">
            <input
              type="text"
              value={newDb}
              onChange={(e) => setNewDb(e.target.value)}
              placeholder="Database name"
              className="flex-1 px-2 py-1.5 rounded-lg border border-border text-xs focus:outline-none focus:border-primary"
            />
            <button
              onClick={handleCreateDb}
              disabled={loading || !newDb.trim()}
              className="px-3 py-1.5 rounded-lg bg-primary text-white text-xs font-medium hover:bg-primary-hover disabled:opacity-50"
            >
              {loading ? <Loader2 className="w-3 h-3 animate-spin" /> : "Create"}
            </button>
          </div>
          {/* Database list */}
          <div className="space-y-1 max-h-40 overflow-y-auto">
            {databases.map((db) => (
              <div
                key={db.name}
                className="flex items-center justify-between py-1 px-2 rounded hover:bg-gray-50 text-xs"
              >
                <span className="font-mono text-text-primary">{db.name}</span>
                {!systemDbs.includes(db.name) && (
                  <button
                    onClick={() => handleDropDb(db.name)}
                    className="text-danger hover:underline"
                  >
                    Drop
                  </button>
                )}
              </div>
            ))}
            {databases.length === 0 && (
              <p className="text-xs text-text-muted py-2">No databases found. Is the service running?</p>
            )}
          </div>
        </div>
      )}

      {tab === "access" && (
        <div>
          {/* Grant access form */}
          <div className="flex gap-2 mb-3">
            <select
              value={grantUser}
              onChange={(e) => setGrantUser(e.target.value)}
              className="flex-1 px-2 py-1.5 rounded-lg border border-border text-xs focus:outline-none focus:border-primary"
            >
              <option value="">Select user...</option>
              {nonSystemUsers.map((u) => (
                <option key={u.username} value={u.username}>
                  {u.username}
                </option>
              ))}
            </select>
            <select
              value={grantDb}
              onChange={(e) => setGrantDb(e.target.value)}
              className="flex-1 px-2 py-1.5 rounded-lg border border-border text-xs focus:outline-none focus:border-primary"
            >
              <option value="">Select database...</option>
              {nonSystemDbs.map((d) => (
                <option key={d.name} value={d.name}>
                  {d.name}
                </option>
              ))}
            </select>
            <button
              onClick={handleGrant}
              disabled={loading || !grantUser || !grantDb}
              className="px-3 py-1.5 rounded-lg bg-success text-white text-xs font-medium hover:bg-green-600 disabled:opacity-50"
            >
              Grant
            </button>
          </div>

          {/* Current grants per user */}
          <div className="space-y-2 max-h-48 overflow-y-auto">
            {nonSystemUsers.length === 0 && (
              <p className="text-xs text-text-muted py-2">Create a user first to manage access.</p>
            )}
            {nonSystemUsers.map((u) => (
              <div key={u.username} className="p-2 rounded-lg bg-gray-50">
                <p className="text-xs font-medium text-text-primary mb-1">
                  {u.username}
                </p>
                <div className="flex flex-wrap gap-1">
                  {(userGrants[u.username] || []).length > 0 ? (
                    (userGrants[u.username] || []).map((db) => (
                      <span
                        key={db}
                        className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-primary/10 text-primary text-xs"
                      >
                        {db}
                        <button
                          onClick={() => handleRevoke(u.username, db)}
                          className="hover:text-danger"
                        >
                          <X className="w-3 h-3" />
                        </button>
                      </span>
                    ))
                  ) : (
                    <span className="text-xs text-text-muted">No specific database grants</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function ServiceCard({ service }: { service: ServiceInstance }) {
  const [loading, setLoading] = useState(false);
  const [showManage, setShowManage] = useState(false);
  const isRunning = service.status === "Running";
  const isDatabase = DB_TYPES.includes(service.serviceType);

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
        {isDatabase && isRunning && (
          <button
            onClick={() => setShowManage(!showManage)}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
              showManage
                ? "bg-primary text-white"
                : "bg-primary/10 text-primary hover:bg-primary/20"
            }`}
          >
            <Users className="w-3 h-3" />
            Manage
          </button>
        )}
      </div>

      {showManage && isRunning && <DatabasePanel service={service} />}
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
