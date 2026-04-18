import { create } from "zustand";

export interface PhpVersion {
  version: string;
  fullVersion: string | null;
  path: string;
  isActive: boolean;
  isInstalled: boolean;
}

export interface PhpExtension {
  name: string;
  enabled: boolean;
}

export interface InstallProgress {
  version: string;
  stage: "downloading" | "extracting" | "complete" | "error";
  progress: number;
  message: string;
}

interface PhpState {
  versions: PhpVersion[];
  currentVersion: string | null;
  installing: Record<string, InstallProgress>;
  setVersions: (versions: PhpVersion[]) => void;
  setCurrentVersion: (version: string) => void;
  setInstallProgress: (version: string, progress: InstallProgress | null) => void;
  markInstalled: (version: string) => void;
  switchVersion: (version: string) => void;
}

export const usePhpStore = create<PhpState>((set) => ({
  versions: [],
  currentVersion: null,
  installing: {},
  setVersions: (versions) => {
    const active = versions.find((v) => v.isActive);
    set({ versions, currentVersion: active?.version ?? null });
  },
  setCurrentVersion: (currentVersion) => set({ currentVersion }),
  setInstallProgress: (version, progress) =>
    set((state) => {
      const installing = { ...state.installing };
      if (progress) {
        installing[version] = progress;
      } else {
        delete installing[version];
      }
      return { installing };
    }),
  markInstalled: (version) =>
    set((state) => ({
      versions: state.versions.map((v) =>
        v.version === version ? { ...v, isInstalled: true } : v
      ),
    })),
  switchVersion: (version) =>
    set((state) => ({
      currentVersion: version,
      versions: state.versions.map((v) => ({
        ...v,
        isActive: v.version === version,
      })),
    })),
}));
