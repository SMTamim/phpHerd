import { useEffect, useState } from "react";
import { Save, CheckCircle, Terminal } from "lucide-react";
import { useSettingsStore } from "../stores/settings";
import { addBinToPath, checkBinOnPath } from "../lib/tauri";
import toast from "react-hot-toast";

export default function Settings() {
  const settings = useSettingsStore((s) => s.settings);
  const updateSettings = useSettingsStore((s) => s.updateSettings);
  const [onPath, setOnPath] = useState<boolean | null>(null);

  const [form, setForm] = useState({
    tld: settings.tld,
    editor: settings.editor,
    autoStart: settings.autoStart,
    smtpPort: settings.smtpPort,
    dumpPort: settings.dumpPort,
  });

  useEffect(() => {
    checkBinOnPath()
      .then(setOnPath)
      .catch(() => setOnPath(null));
  }, []);

  const handleSave = () => {
    updateSettings({
      ...settings,
      tld: form.tld,
      editor: form.editor,
      autoStart: form.autoStart,
      smtpPort: form.smtpPort,
      dumpPort: form.dumpPort,
    });
    toast.success("Settings saved");
  };

  const handleAddToPath = async () => {
    try {
      const added = await addBinToPath();
      if (added) {
        toast.success("Added to PATH. Restart your terminal to use php and composer globally.");
        setOnPath(true);
      } else {
        toast("Already on PATH");
        setOnPath(true);
      }
    } catch (err) {
      toast.error(String(err));
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-text-primary">Settings</h1>
          <p className="text-text-secondary mt-1">
            Configure phpHerd preferences
          </p>
        </div>
        <button
          onClick={handleSave}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover transition-colors"
        >
          <Save className="w-4 h-4" />
          Save Settings
        </button>
      </div>

      <div className="max-w-2xl space-y-6">
        {/* Environment PATH */}
        <div className="bg-surface rounded-xl border border-border p-6">
          <h2 className="text-lg font-semibold text-text-primary mb-2">
            Terminal Integration
          </h2>
          <p className="text-sm text-text-secondary mb-4">
            Add phpHerd's bin directory to your system PATH so you can use{" "}
            <code className="px-1.5 py-0.5 rounded bg-gray-100 text-xs font-mono">php</code>,{" "}
            <code className="px-1.5 py-0.5 rounded bg-gray-100 text-xs font-mono">composer</code>,{" "}
            and{" "}
            <code className="px-1.5 py-0.5 rounded bg-gray-100 text-xs font-mono">node</code>{" "}
            from any terminal.
          </p>

          {onPath === true ? (
            <div className="flex items-center gap-2 px-4 py-3 rounded-lg bg-green-50 border border-green-200">
              <CheckCircle className="w-5 h-5 text-success" />
              <div>
                <p className="text-sm font-medium text-green-800">
                  phpHerd bin is on your PATH
                </p>
                <p className="text-xs text-green-600 font-mono mt-0.5">
                  php, composer, and node are available in all terminals
                </p>
              </div>
            </div>
          ) : (
            <button
              onClick={handleAddToPath}
              className="flex items-center gap-2 px-4 py-3 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover transition-colors"
            >
              <Terminal className="w-4 h-4" />
              Add to System PATH
            </button>
          )}

          <p className="text-xs text-text-muted mt-3">
            After adding, restart VS Code or open a new terminal for changes to take effect.
          </p>
        </div>

        {/* General */}
        <div className="bg-surface rounded-xl border border-border p-6">
          <h2 className="text-lg font-semibold text-text-primary mb-4">
            General
          </h2>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-text-primary mb-1">
                Top-Level Domain
              </label>
              <input
                type="text"
                value={form.tld}
                onChange={(e) => setForm({ ...form, tld: e.target.value })}
                className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-sm focus:outline-none focus:border-primary"
                placeholder="test"
              />
              <p className="text-xs text-text-muted mt-1">
                Sites will be available at *.{form.tld}
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-text-primary mb-1">
                Editor Command
              </label>
              <input
                type="text"
                value={form.editor}
                onChange={(e) => setForm({ ...form, editor: e.target.value })}
                className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-sm focus:outline-none focus:border-primary"
                placeholder="code"
              />
              <p className="text-xs text-text-muted mt-1">
                Command to open projects in your editor (e.g., code, phpstorm,
                subl)
              </p>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <label className="block text-sm font-medium text-text-primary">
                  Auto-start on login
                </label>
                <p className="text-xs text-text-muted mt-0.5">
                  Start phpHerd automatically when you log in
                </p>
              </div>
              <button
                onClick={() =>
                  setForm({ ...form, autoStart: !form.autoStart })
                }
                className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                  form.autoStart ? "bg-primary" : "bg-gray-300"
                }`}
              >
                <span
                  className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                    form.autoStart ? "translate-x-6" : "translate-x-1"
                  }`}
                />
              </button>
            </div>
          </div>
        </div>

        {/* Ports */}
        <div className="bg-surface rounded-xl border border-border p-6">
          <h2 className="text-lg font-semibold text-text-primary mb-4">
            Ports
          </h2>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-text-primary mb-1">
                SMTP Port (Mail Catcher)
              </label>
              <input
                type="number"
                value={form.smtpPort}
                onChange={(e) =>
                  setForm({ ...form, smtpPort: parseInt(e.target.value) || 2525 })
                }
                className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-sm focus:outline-none focus:border-primary"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-text-primary mb-1">
                Dump Server Port
              </label>
              <input
                type="number"
                value={form.dumpPort}
                onChange={(e) =>
                  setForm({ ...form, dumpPort: parseInt(e.target.value) || 9912 })
                }
                className="w-full px-3 py-2 rounded-lg border border-border bg-surface text-sm focus:outline-none focus:border-primary"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
