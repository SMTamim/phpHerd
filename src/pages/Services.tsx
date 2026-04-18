import { useState } from "react";
import {
  Database,
  Plus,
  Play,
  Square,
  Trash2,
  CheckCircle,
  XCircle,
} from "lucide-react";
import { useServicesStore, type ServiceInstance } from "../stores/services";

const availableServices = [
  { type: "mysql", name: "MySQL", icon: "🐬" },
  { type: "mariadb", name: "MariaDB", icon: "🦭" },
  { type: "postgresql", name: "PostgreSQL", icon: "🐘" },
  { type: "redis", name: "Redis", icon: "🔴" },
  { type: "mongodb", name: "MongoDB", icon: "🍃" },
  { type: "meilisearch", name: "Meilisearch", icon: "🔍" },
  { type: "typesense", name: "Typesense", icon: "⚡" },
  { type: "minio", name: "MinIO", icon: "📦" },
];

function ServiceCard({ service }: { service: ServiceInstance }) {
  const startService = useServicesStore((s) => s.startService);
  const stopService = useServicesStore((s) => s.stopService);
  const isRunning = service.status === "Running";

  return (
    <div className="bg-surface rounded-xl border border-border p-5 animate-fade-in">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-primary-light">
            <Database className="w-4 h-4 text-primary" />
          </div>
          <div>
            <h3 className="font-semibold text-text-primary">
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

      <div className="text-sm text-text-secondary mb-4">
        Port: <span className="font-mono font-medium">{service.port}</span>
      </div>

      <div className="flex items-center gap-2">
        {isRunning ? (
          <button
            onClick={() => stopService(service.id)}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-danger/10 text-danger text-xs font-medium hover:bg-danger/20 transition-colors"
          >
            <Square className="w-3 h-3" />
            Stop
          </button>
        ) : (
          <button
            onClick={() => startService(service.id)}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-success/10 text-success text-xs font-medium hover:bg-success/20 transition-colors"
          >
            <Play className="w-3 h-3" />
            Start
          </button>
        )}
        <button className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-text-secondary text-xs font-medium hover:bg-gray-100 transition-colors">
          <Trash2 className="w-3 h-3" />
          Delete
        </button>
      </div>
    </div>
  );
}

export default function Services() {
  const services = useServicesStore((s) => s.services);
  const [showCreate, setShowCreate] = useState(false);

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

      {/* Available Services */}
      {showCreate && (
        <div className="mb-8 p-6 bg-surface rounded-xl border border-border animate-fade-in">
          <h3 className="text-sm font-medium text-text-primary mb-4">
            Choose a service to add
          </h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            {availableServices.map((svc) => (
              <button
                key={svc.type}
                className="flex items-center gap-2 p-3 rounded-lg border border-border hover:border-primary hover:bg-primary-light/50 transition-colors text-left"
              >
                <span className="text-lg">{svc.icon}</span>
                <span className="text-sm font-medium text-text-primary">
                  {svc.name}
                </span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Services Grid */}
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
