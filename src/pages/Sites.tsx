import { useEffect, useState } from "react";
import {
  Globe,
  Plus,
  FolderOpen,
  Lock,
  Unlock,
  Trash2,
  ExternalLink,
  X,
} from "lucide-react";
import { useSitesStore, type Site } from "../stores/sites";
import {
  getSites,
  getParkedPaths,
  parkDirectory,
  unparkDirectory,
  linkSite,
  unlinkSite,
  secureSite,
  unsecureSite,
  isolateSitePhp,
  pickFolder,
  type SiteInfo,
} from "../lib/tauri";
import { usePhpStore } from "../stores/php";
import { getPhpVersions } from "../lib/tauri";
import toast from "react-hot-toast";

function mapSiteInfo(s: SiteInfo): Site {
  return {
    name: s.name,
    path: s.path,
    url: s.url,
    phpVersion: s.php_version,
    nodeVersion: s.node_version,
    secured: s.secured,
    isParked: s.is_parked,
  };
}

async function refreshAll() {
  try {
    const [sites, parked] = await Promise.all([getSites(), getParkedPaths()]);
    useSitesStore.getState().setSites(sites.map(mapSiteInfo));
    useSitesStore.getState().setParkedPaths(parked);
  } catch {
    // ignore
  }
}

function SiteCard({ site }: { site: Site }) {
  const phpVersions = usePhpStore((s) => s.versions);
  const installedPhp = phpVersions.filter((v) => v.isInstalled);
  const [showIsolate, setShowIsolate] = useState(false);

  const handleSecureToggle = async () => {
    try {
      if (site.secured) {
        await unsecureSite(site.name);
        toast.success(`Removed HTTPS from ${site.name}`);
      } else {
        await secureSite(site.name);
        toast.success(`Secured ${site.name} with HTTPS`);
      }
      refreshAll();
    } catch (err) {
      toast.error(String(err));
    }
  };

  const handleUnlink = async () => {
    try {
      await unlinkSite(site.name);
      toast.success(`Unlinked ${site.name}`);
      refreshAll();
    } catch (err) {
      toast.error(String(err));
    }
  };

  const handleIsolatePhp = async (version: string) => {
    try {
      await isolateSitePhp(site.name, version);
      toast.success(`Isolated ${site.name} to PHP ${version}`);
      setShowIsolate(false);
      refreshAll();
    } catch (err) {
      toast.error(String(err));
    }
  };

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
              target="_blank"
              rel="noreferrer"
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

      <p className="text-xs text-text-secondary mb-3 truncate" title={site.path}>
        {site.path}
      </p>

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

      {/* PHP Isolate Dropdown */}
      {showIsolate && (
        <div className="mt-3 p-3 rounded-lg border border-border bg-white">
          <p className="text-xs font-medium text-text-primary mb-2">
            Select PHP version:
          </p>
          <div className="flex flex-wrap gap-2">
            {installedPhp.length > 0 ? (
              installedPhp.map((v) => (
                <button
                  key={v.version}
                  onClick={() => handleIsolatePhp(v.version)}
                  className={`px-3 py-1 text-xs rounded-lg border transition-colors ${
                    site.phpVersion === v.version
                      ? "border-primary bg-primary text-white"
                      : "border-border hover:border-primary text-text-primary"
                  }`}
                >
                  {v.version}
                </button>
              ))
            ) : (
              <p className="text-xs text-text-muted">
                No PHP versions installed
              </p>
            )}
          </div>
        </div>
      )}

      <div className="flex items-center gap-2 mt-4 pt-3 border-t border-border">
        <button
          onClick={handleSecureToggle}
          className="text-xs text-text-secondary hover:text-primary transition-colors"
        >
          {site.secured ? "Unsecure" : "Secure"}
        </button>
        {!site.isParked && (
          <>
            <span className="text-border">|</span>
            <button
              onClick={() => setShowIsolate(!showIsolate)}
              className="text-xs text-text-secondary hover:text-primary transition-colors"
            >
              Isolate PHP
            </button>
            <span className="text-border">|</span>
            <button
              onClick={handleUnlink}
              className="text-xs text-text-secondary hover:text-danger transition-colors flex items-center gap-1"
            >
              <Trash2 className="w-3 h-3" />
              Unlink
            </button>
          </>
        )}
      </div>
    </div>
  );
}

function LinkSiteForm({ onClose }: { onClose: () => void }) {
  const [name, setName] = useState("");
  const [path, setPath] = useState("");

  const handleBrowse = async () => {
    const selected = await pickFolder();
    if (selected) {
      setPath(selected);
      if (!name) {
        // Auto-fill name from folder name
        const parts = selected.replace(/\\/g, "/").split("/");
        setName(parts[parts.length - 1] || "");
      }
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !path.trim()) {
      toast.error("Name and path are required");
      return;
    }
    try {
      await linkSite(name.trim(), path.trim());
      toast.success(`Linked ${name} -> ${path}`);
      onClose();
      refreshAll();
    } catch (err) {
      toast.error(String(err));
    }
  };

  return (
    <div className="mb-6 p-5 rounded-xl bg-surface border border-primary animate-fade-in">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-semibold text-text-primary">
          Link New Site
        </h3>
        <button
          onClick={onClose}
          className="text-text-muted hover:text-text-primary"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
      <form onSubmit={handleSubmit} className="space-y-3">
        <div>
          <label className="block text-xs font-medium text-text-secondary mb-1">
            Site Name (becomes name.test)
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="my-project"
            className="w-full px-3 py-2 rounded-lg border border-border bg-white text-sm text-text-primary focus:outline-none focus:border-primary"
          />
        </div>
        <div>
          <label className="block text-xs font-medium text-text-secondary mb-1">
            Project Path
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              placeholder="C:\Users\you\projects\my-project"
              className="flex-1 px-3 py-2 rounded-lg border border-border bg-white text-sm text-text-primary focus:outline-none focus:border-primary"
            />
            <button
              type="button"
              onClick={handleBrowse}
              className="px-3 py-2 rounded-lg border border-border hover:bg-gray-50 transition-colors"
            >
              <FolderOpen className="w-4 h-4 text-text-secondary" />
            </button>
          </div>
        </div>
        <div className="flex justify-end gap-2 pt-1">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 rounded-lg text-sm text-text-secondary hover:bg-gray-100 transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover transition-colors"
          >
            Link Site
          </button>
        </div>
      </form>
    </div>
  );
}

export default function Sites() {
  const sites = useSitesStore((s) => s.sites);
  const parkedPaths = useSitesStore((s) => s.parkedPaths);
  const setVersions = usePhpStore((s) => s.setVersions);
  const [showLinkForm, setShowLinkForm] = useState(false);

  useEffect(() => {
    refreshAll();
    // Also load PHP versions for the isolate dropdown
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

  const handleParkDirectory = async () => {
    const selected = await pickFolder();
    if (selected) {
      try {
        await parkDirectory(selected);
        toast.success(`Parked directory: ${selected}`);
        refreshAll();
      } catch (err) {
        toast.error(String(err));
      }
    }
  };

  const handleUnpark = async (path: string) => {
    try {
      await unparkDirectory(path);
      toast.success("Removed parked directory");
      refreshAll();
    } catch (err) {
      toast.error(String(err));
    }
  };

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
          <button
            onClick={handleParkDirectory}
            className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border text-sm font-medium text-text-primary hover:bg-surface transition-colors"
          >
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

      {/* Link Site Form */}
      {showLinkForm && (
        <LinkSiteForm onClose={() => setShowLinkForm(false)} />
      )}

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
                <button
                  onClick={() => handleUnpark(path)}
                  className="text-xs text-danger hover:underline"
                >
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
