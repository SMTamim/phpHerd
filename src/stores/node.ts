import { create } from "zustand";

export interface NodeVersion {
  version: string;
  fullVersion: string | null;
  isActive: boolean;
  isInstalled: boolean;
  path: string;
}

export interface InstallProgress {
  version: string;
  stage: "downloading" | "extracting" | "complete" | "error";
  progress: number;
  message: string;
}

interface NodeState {
  versions: NodeVersion[];
  currentVersion: string | null;
  installing: Record<string, InstallProgress>;
  setVersions: (versions: NodeVersion[]) => void;
  setCurrentVersion: (version: string | null) => void;
  setInstallProgress: (version: string, progress: InstallProgress | null) => void;
  switchVersion: (version: string) => void;
}

export const useNodeStore = create<NodeState>((set) => ({
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
  switchVersion: (version) =>
    set((state) => ({
      currentVersion: version,
      versions: state.versions.map((v) => ({
        ...v,
        isActive: v.version === version,
      })),
    })),
}));
