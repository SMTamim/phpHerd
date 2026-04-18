import { useState } from "react";
import {
  Globe,
  Plus,
  FolderOpen,
  Lock,
  Unlock,
  Trash2,
  ExternalLink,
} from "lucide-react";
import { useSitesStore, type Site } from "../stores/sites";

function SiteCard({ site }: { site: Site }) {
  return (
    <div className="bg-surface rounded-xl border border-border p-5 animate-fade-in hover:border-primary/30 transition-colors">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-lg bg-primary-light">
            <Globe className="w-4 h-4 text-primary" />
          </div>
          <div>
            <h3 className="font-semibold text-text-primary">{site.name}</h3>
            <a
              href={site.url}
              className="text-xs text-primary hover:underline flex items-center gap-1"
            >
              {site.url}
              <ExternalLink className="w-3 h-3" />
            </a>
          </div>
        </div>
        <div className="flex items-center gap-1">
          {site.secured ? (
            <Lock className="w-4 h-4 text-success" />
          ) : (
            <Unlock className="w-4 h-4 text-text-muted" />
          )}
        </div>
      </div>

      <p className="text-xs text-text-secondary mb-3 truncate">{site.path}</p>

      <div className="flex items-center gap-2 flex-wrap">
        {site.phpVersion && (
          <span className="px-2 py-0.5 text-xs rounded-full bg-indigo-50 text-indigo-600 font-medium">
            PHP {site.phpVersion}
          </span>
        )}
        {site.nodeVersion && (
          <span className="px-2 py-0.5 text-xs rounded-full bg-green-50 text-green-600 font-medium">
            Node {site.nodeVersion}
          </span>
        )}
        {site.isParked && (
          <span className="px-2 py-0.5 text-xs rounded-full bg-amber-50 text-amber-600 font-medium">
            Parked
          </span>
        )}
      </div>

      <div className="flex items-center gap-2 mt-4 pt-3 border-t border-border">
        <button className="text-xs text-text-secondary hover:text-primary transition-colors">
          {site.secured ? "Unsecure" : "Secure"}
        </button>
        <span className="text-border">|</span>
        <button className="text-xs text-text-secondary hover:text-primary transition-colors">
          Isolate PHP
        </button>
        <span className="text-border">|</span>
        <button className="text-xs text-text-secondary hover:text-danger transition-colors flex items-center gap-1">
          <Trash2 className="w-3 h-3" />
          Unlink
        </button>
      </div>
    </div>
  );
}

export default function Sites() {
  const sites = useSitesStore((s) => s.sites);
  const parkedPaths = useSitesStore((s) => s.parkedPaths);
  const [showLinkForm, setShowLinkForm] = useState(false);

  return (
    <div>
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-text-primary">Sites</h1>
          <p className="text-text-secondary mt-1">
            Manage your local development sites
          </p>
        </div>
        <div className="flex gap-3">
          <button className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border text-sm font-medium text-text-primary hover:bg-surface transition-colors">
            <FolderOpen className="w-4 h-4" />
            Park Directory
          </button>
          <button
            onClick={() => setShowLinkForm(!showLinkForm)}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover transition-colors"
          >
            <Plus className="w-4 h-4" />
            Link Site
          </button>
        </div>
      </div>

      {/* Parked Paths */}
      {parkedPaths.length > 0 && (
        <div className="mb-6 p-4 rounded-lg bg-surface border border-border">
          <h3 className="text-sm font-medium text-text-primary mb-2">
            Parked Directories
          </h3>
          <div className="space-y-1">
            {parkedPaths.map((path) => (
              <div
                key={path}
                className="flex items-center justify-between py-1"
              >
                <span className="text-sm text-text-secondary">{path}</span>
                <button className="text-xs text-danger hover:underline">
                  Remove
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Sites Grid */}
      {sites.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {sites.map((site) => (
            <SiteCard key={site.name} site={site} />
          ))}
        </div>
      ) : (
        <div className="text-center py-16">
          <Globe className="w-12 h-12 text-text-muted mx-auto mb-4" />
          <h3 className="text-lg font-medium text-text-primary mb-2">
            No sites yet
          </h3>
          <p className="text-text-secondary mb-4">
            Park a directory or link a project to get started
          </p>
        </div>
      )}
    </div>
  );
}
