/**
 * Tauri IPC invoke wrappers with type safety.
 */

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke<T>(cmd, args);
}

// -- PHP commands --
export interface PhpVersionInfo {
  version: string;
  full_version: string | null;
  path: string;
  is_active: boolean;
  is_installed: boolean;
}

export const getPhpVersions = () => invoke<PhpVersionInfo[]>("get_php_versions");
export const getCurrentPhpVersion = () => invoke<string>("get_current_php_version");
export const installPhpVersion = (version: string) =>
  invoke<void>("install_php_version", { version });
export const uninstallPhpVersion = (version: string) =>
  invoke<void>("uninstall_php_version", { version });
export const switchPhpVersion = (version: string) =>
  invoke<void>("switch_php_version", { version });
export const getPhpExtensions = (version: string) =>
  invoke<{ name: string; enabled: boolean }[]>("get_php_extensions", { version });
export const togglePhpExtension = (version: string, extension: string, enabled: boolean) =>
  invoke<void>("toggle_php_extension", { version, extension, enabled });

// -- Node commands --
export interface NodeVersionInfo {
  version: string;
  full_version: string | null;
  is_active: boolean;
  is_installed: boolean;
  path: string;
}

export const getNodeVersions = () => invoke<NodeVersionInfo[]>("get_node_versions");
export const getCurrentNodeVersion = () => invoke<string | null>("get_current_node_version");
export const installNodeVersion = (version: string) =>
  invoke<void>("install_node_version", { version });
export const switchNodeVersion = (version: string) =>
  invoke<void>("switch_node_version", { version });

// -- Nginx commands --
export interface NginxStatusInfo {
  running: boolean;
  pid: number | null;
  version: string | null;
  installed: boolean;
}

export const getNginxStatus = () => invoke<NginxStatusInfo>("get_nginx_status");
export const installNginx = () => invoke<void>("install_nginx");
export const startNginx = () => invoke<void>("start_nginx");
export const stopNginx = () => invoke<void>("stop_nginx");
export const restartNginx = () => invoke<void>("restart_nginx");

// -- Site commands --
export const getSites = () => invoke<SiteInfo[]>("get_sites");
export const getParkedPaths = () => invoke<string[]>("get_parked_paths");
export const parkDirectory = (path: string) =>
  invoke<void>("park_directory", { path });
export const unparkDirectory = (path: string) =>
  invoke<void>("unpark_directory", { path });
export const linkSite = (name: string, path: string) =>
  invoke<void>("link_site", { name, path });
export const unlinkSite = (name: string) =>
  invoke<void>("unlink_site", { name });
export const secureSite = (siteName: string) =>
  invoke<void>("secure_site", { siteName });
export const unsecureSite = (siteName: string) =>
  invoke<void>("unsecure_site", { siteName });
export const isolateSitePhp = (siteName: string, phpVersion: string) =>
  invoke<void>("isolate_site_php", { siteName, phpVersion });
export const installPhpmyadmin = () => invoke<void>("install_phpmyadmin");

// -- Service commands --
export interface ServiceInfoData {
  id: string;
  service_type: string;
  version: string;
  port: number;
  status: string;
  data_dir: string;
}

export interface AvailableServiceData {
  service_type: string;
  display_name: string;
  versions: string[];
  default_port: number;
}

export const getServices = () => invoke<ServiceInfoData[]>("get_services");
export const getAvailableServices = () =>
  invoke<AvailableServiceData[]>("get_available_services");
export const createService = (request: {
  service_type: string;
  version: string;
  port?: number;
}) => invoke<ServiceInfoData>("create_service", { request });
export const startService = (serviceId: string) =>
  invoke<void>("start_service", { serviceId });
export const stopService = (serviceId: string) =>
  invoke<void>("stop_service", { serviceId });
export const deleteService = (serviceId: string) =>
  invoke<void>("delete_service", { serviceId });

// -- Settings commands --
export const getSettings = () => invoke<unknown>("get_settings");
export const updateSettings = (settings: unknown) =>
  invoke<void>("update_settings", { settings });

// -- Site commands --
export interface SiteInfo {
  name: string;
  path: string;
  url: string;
  php_version: string | null;
  node_version: string | null;
  secured: boolean;
  is_parked: boolean;
}

// -- Dialog --
export async function pickFolder(): Promise<string | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const result = await open({ directory: true, multiple: false });
  return result as string | null;
}

// -- Event listening --
export async function listenToEvent<T>(
  event: string,
  callback: (payload: T) => void,
): Promise<() => void> {
  const { listen } = await import("@tauri-apps/api/event");
  const unlisten = await listen<T>(event, (e) => callback(e.payload));
  return unlisten;
}
