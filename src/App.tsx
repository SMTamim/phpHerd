import { Routes, Route } from "react-router-dom";
import { Sidebar } from "./components/layout/Sidebar";
import Dashboard from "./pages/Dashboard";
import Sites from "./pages/Sites";
import PhpVersions from "./pages/PhpVersions";
import NodeVersions from "./pages/NodeVersions";
import Services from "./pages/Services";
import Dumps from "./pages/Dumps";
import Logs from "./pages/Logs";
import Mail from "./pages/Mail";
import Settings from "./pages/Settings";

function App() {
  return (
    <div className="flex h-screen overflow-hidden">
      <Sidebar />
      <main className="flex-1 overflow-y-auto bg-surface-secondary">
        <div className="p-8">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/sites" element={<Sites />} />
            <Route path="/php" element={<PhpVersions />} />
            <Route path="/node" element={<NodeVersions />} />
            <Route path="/services" element={<Services />} />
            <Route path="/dumps" element={<Dumps />} />
            <Route path="/logs" element={<Logs />} />
            <Route path="/mail" element={<Mail />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </div>
      </main>
    </div>
  );
}

export default App;
