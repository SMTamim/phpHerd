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

// -- Site commands --
export const getSites = () => invoke<unknown[]>("get_sites");
export const getParkedPaths = () => invoke<string[]>("get_parked_paths");
export const parkDirectory = (path: string) =>
  invoke<void>("park_directory", { path });
export const linkSite = (name: string, path: string) =>
  invoke<void>("link_site", { name, path });
export const unlinkSite = (name: string) =>
  invoke<void>("unlink_site", { name });
export const secureSite = (siteName: string) =>
  invoke<void>("secure_site", { siteName });
export const unsecureSite = (siteName: string) =>
  invoke<void>("unsecure_site", { siteName });

// -- Nginx commands --
export const getNginxStatus = () => invoke<unknown>("get_nginx_status");
export const startNginx = () => invoke<void>("start_nginx");
export const stopNginx = () => invoke<void>("stop_nginx");
export const restartNginx = () => invoke<void>("restart_nginx");

// -- Service commands --
export const getServices = () => invoke<unknown[]>("get_services");
export const createService = (request: {
  service_type: string;
  version: string;
  port?: number;
}) => invoke<unknown>("create_service", { request });
export const startService = (serviceId: string) =>
  invoke<void>("start_service", { serviceId });
export const stopService = (serviceId: string) =>
  invoke<void>("stop_service", { serviceId });

// -- Settings commands --
export const getSettings = () => invoke<unknown>("get_settings");
export const updateSettings = (settings: unknown) =>
  invoke<void>("update_settings", { settings });

// -- Event listening --
export async function listenToEvent<T>(
  event: string,
  callback: (payload: T) => void,
): Promise<() => void> {
  const { listen } = await import("@tauri-apps/api/event");
  const unlisten = await listen<T>(event, (e) => callback(e.payload));
  return unlisten;
}
