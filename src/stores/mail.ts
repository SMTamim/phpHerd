import { create } from "zustand";

export interface Email {
  id: string;
  from: string;
  to: string[];
  subject: string;
  htmlBody: string | null;
  textBody: string | null;
  timestamp: string;
  read: boolean;
  appName: string | null;
}

interface MailState {
  emails: Email[];
  setEmails: (emails: Email[]) => void;
  addEmail: (email: Email) => void;
  markRead: (id: string) => void;
  deleteEmail: (id: string) => void;
  clearAll: () => void;
}

export const useMailStore = create<MailState>((set) => ({
  emails: [],
  setEmails: (emails) => set({ emails }),
  addEmail: (email) => set((state) => ({ emails: [email, ...state.emails] })),
  markRead: (id) =>
    set((state) => ({
      emails: state.emails.map((e) =>
        e.id === id ? { ...e, read: true } : e
      ),
    })),
  deleteEmail: (id) =>
    set((state) => ({ emails: state.emails.filter((e) => e.id !== id) })),
  clearAll: () => set({ emails: [] }),
}));
