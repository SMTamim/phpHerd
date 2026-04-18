import { create } from "zustand";

export interface LogFile {
  name: string;
  path: string;
  size: number;
  modified: string;
}

export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
  context: string | null;
}

interface LogsState {
  logFiles: LogFile[];
  entries: LogEntry[];
  selectedFile: string | null;
  setLogFiles: (files: LogFile[]) => void;
  setEntries: (entries: LogEntry[]) => void;
  setSelectedFile: (path: string) => void;
}

export const useLogsStore = create<LogsState>((set) => ({
  logFiles: [],
  entries: [],
  selectedFile: null,
  setLogFiles: (logFiles) => set({ logFiles }),
  setEntries: (entries) => set({ entries }),
  setSelectedFile: (selectedFile) => set({ selectedFile }),
}));
