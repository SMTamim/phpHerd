import { create } from "zustand";

export interface Site {
  name: string;
  path: string;
  url: string;
  phpVersion: string | null;
  nodeVersion: string | null;
  secured: boolean;
  isParked: boolean;
}

interface SitesState {
  sites: Site[];
  parkedPaths: string[];
  setSites: (sites: Site[]) => void;
  setParkedPaths: (paths: string[]) => void;
  addSite: (site: Site) => void;
  removeSite: (name: string) => void;
  parkDirectory: (path: string) => void;
  unparkDirectory: (path: string) => void;
}

export const useSitesStore = create<SitesState>((set) => ({
  sites: [],
  parkedPaths: [],
  setSites: (sites) => set({ sites }),
  setParkedPaths: (parkedPaths) => set({ parkedPaths }),
  addSite: (site) => set((state) => ({ sites: [...state.sites, site] })),
  removeSite: (name) =>
    set((state) => ({ sites: state.sites.filter((s) => s.name !== name) })),
  parkDirectory: (path) =>
    set((state) => ({
      parkedPaths: [...state.parkedPaths, path],
    })),
  unparkDirectory: (path) =>
    set((state) => ({
      parkedPaths: state.parkedPaths.filter((p) => p !== path),
    })),
}));
