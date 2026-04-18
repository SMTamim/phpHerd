import { create } from "zustand";

export interface ServiceInstance {
  id: string;
  serviceType: string;
  version: string;
  port: number;
  status: "Running" | "Stopped" | "Error";
  dataDir: string;
}

interface ServicesState {
  services: ServiceInstance[];
  setServices: (services: ServiceInstance[]) => void;
  addService: (service: ServiceInstance) => void;
  removeService: (id: string) => void;
  startService: (id: string) => void;
  stopService: (id: string) => void;
}

export const useServicesStore = create<ServicesState>((set) => ({
  services: [],
  setServices: (services) => set({ services }),
  addService: (service) =>
    set((state) => ({ services: [...state.services, service] })),
  removeService: (id) =>
    set((state) => ({ services: state.services.filter((s) => s.id !== id) })),
  startService: (id) =>
    set((state) => ({
      services: state.services.map((s) =>
        s.id === id ? { ...s, status: "Running" as const } : s
      ),
    })),
  stopService: (id) =>
    set((state) => ({
      services: state.services.map((s) =>
        s.id === id ? { ...s, status: "Stopped" as const } : s
      ),
    })),
}));
