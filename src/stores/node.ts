import { create } from "zustand";

export interface NodeVersion {
  version: string;
  isActive: boolean;
  isInstalled: boolean;
  path: string;
}

interface NodeState {
  versions: NodeVersion[];
  currentVersion: string | null;
  setVersions: (versions: NodeVersion[]) => void;
  setCurrentVersion: (version: string) => void;
  switchVersion: (version: string) => void;
}

export const useNodeStore = create<NodeState>((set) => ({
  versions: [],
  currentVersion: null,
  setVersions: (versions) => set({ versions }),
  setCurrentVersion: (currentVersion) => set({ currentVersion }),
  switchVersion: (version) =>
    set((state) => ({
      currentVersion: version,
      versions: state.versions.map((v) => ({
        ...v,
        isActive: v.version === version,
      })),
    })),
}));
