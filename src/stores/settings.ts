import { create } from "zustand";

export interface AppSettings {
  tld: string;
  defaultPhp: string;
  editor: string;
  autoStart: boolean;
  smtpPort: number;
  dumpPort: number;
}

interface SettingsState {
  settings: AppSettings;
  updateSettings: (settings: AppSettings) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: {
    tld: "test",
    defaultPhp: "8.3",
    editor: "code",
    autoStart: true,
    smtpPort: 2525,
    dumpPort: 9912,
  },
  updateSettings: (settings) => set({ settings }),
}));
