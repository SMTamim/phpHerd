import { create } from "zustand";

export interface DumpEntry {
  id: string;
  type: "dump" | "query" | "job" | "view" | "http" | "log";
  content: string;
  file: string | null;
  line: number | null;
  site: string | null;
  timestamp: string;
}

interface DumpsState {
  dumps: DumpEntry[];
  setDumps: (dumps: DumpEntry[]) => void;
  addDump: (dump: DumpEntry) => void;
  clearDumps: () => void;
}

export const useDumpsStore = create<DumpsState>((set) => ({
  dumps: [],
  setDumps: (dumps) => set({ dumps }),
  addDump: (dump) => set((state) => ({ dumps: [dump, ...state.dumps] })),
  clearDumps: () => set({ dumps: [] }),
}));
