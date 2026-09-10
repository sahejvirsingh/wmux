import { Plug, Trash2, X } from "lucide-react";
import { useEffect, useState } from "react";
import type { SavedSshConnection, SshTestResult } from "../../types";
import { sshConnect, sshDelete, sshList, sshSave, sshTest } from "../../lib/tauri-bridge";
import { useUiStore } from "../../stores/uiStore";
import "./SSHDialog.css";

const emptyForm = {
  name: "",
  host: "",
  port: 22,
  user: "",
  password: "",
  keyPath: "",
  passphrase: "",
};

export function SSHDialog() {
  const open = useUiStore((s) => s.sshOpen);
  const setOpen = useUiStore((s) => s.setSshOpen);
  const [form, setForm] = useState(emptyForm);
  const [authMode, setAuthMode] = useState<"password" | "key">("password");
  const [testing, setTesting] = useState(false);
  const [result, setResult] = useState<SshTestResult | null>(null);
  const [saved, setSaved] = useState<SavedSshConnection[]>([]);

  useEffect(() => {
    if (open) {
      sshList()
        .then(setSaved)
        .catch(() => setSaved([]));
      setResult(null);
    }
  }, [open]);

  if (!open) {
    return null;
  }

  const target = { host: form.host, port: form.port || 22, user: form.user };
  const auth =
    authMode === "password"
      ? { kind: "password" as const, password: form.password }
      : { kind: "key" as const, keyPath: form.keyPath, passphrase: form.passphrase || null };

  const onTest = async () => {
    if (!form.host || !form.user) {
      return;
    }
    setTesting(true);
    setResult(null);
    try {
      setResult(await sshTest(target, auth));
    } catch (e) {
      setResult({ ok: false, banner: null, error: String(e) });
    } finally {
      setTesting(false);
    }
  };

  const onConnect = async () => {
    try {
      await sshConnect(target, form.name || undefined);
      setOpen(false);
      setForm(emptyForm);
    } catch (e) {
      setResult({ ok: false, banner: null, error: String(e) });
    }
  };

  const onSave = async () => {
    if (!form.host || !form.user) {
      return;
    }
    const connection: SavedSshConnection = {
      id: `${form.user}@${form.host}:${form.port || 22}`,
      name: form.name || `${form.user}@${form.host}`,
      host: form.host,
      port: form.port || 22,
      user: form.user,
      auth,
    };
    try {
      await sshSave(connection);
      setSaved(await sshList());
    } catch (e) {
      setResult({ ok: false, banner: null, error: String(e) });
    }
  };

  const useSaved = (conn: SavedSshConnection) => {
    setForm({
      name: conn.name,
      host: conn.host,
      port: conn.port,
      user: conn.user,
      password: conn.auth.kind === "password" ? conn.auth.password : "",
      keyPath: conn.auth.kind === "key" ? conn.auth.keyPath : "",
      passphrase: conn.auth.kind === "key" ? conn.auth.passphrase ?? "" : "",
    });
    setAuthMode(conn.auth.kind);
  };

  return (
    <div className="panel-backdrop ssh-backdrop" onMouseDown={() => setOpen(false)}>
      <div className="ssh-dialog" onMouseDown={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <span>SSH</span>
          <button className="icon-button" onClick={() => setOpen(false)}>
            <X size={14} />
          </button>
        </div>
        <div className="ssh-body">
          <div className="ssh-form">
            <label className="settings-field">
              <span>Name (optional)</span>
              <input
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                placeholder="my server"
              />
            </label>
            <div className="ssh-form-row">
              <label className="settings-field">
                <span>Host</span>
                <input
                  value={form.host}
                  onChange={(e) => setForm({ ...form, host: e.target.value })}
                  placeholder="192.168.1.10"
                />
              </label>
              <label className="settings-field">
                <span>Port</span>
                <input
                  type="number"
                  value={form.port}
                  onChange={(e) => setForm({ ...form, port: Number(e.target.value) || 22 })}
                />
              </label>
              <label className="settings-field">
                <span>User</span>
                <input
                  value={form.user}
                  onChange={(e) => setForm({ ...form, user: e.target.value })}
                  placeholder="root"
                />
              </label>
            </div>
            <div className="ssh-auth-toggle">
              <button
                className={`settings-small-button ${authMode === "password" ? "active" : ""}`}
                onClick={() => setAuthMode("password")}
              >
                password
              </button>
              <button
                className={`settings-small-button ${authMode === "key" ? "active" : ""}`}
                onClick={() => setAuthMode("key")}
              >
                key file
              </button>
            </div>
            {authMode === "password" ? (
              <label className="settings-field">
                <span>Password</span>
                <input
                  type="password"
                  value={form.password}
                  onChange={(e) => setForm({ ...form, password: e.target.value })}
                />
              </label>
            ) : (
              <>
                <label className="settings-field">
                  <span>Key path</span>
                  <input
                    value={form.keyPath}
                    onChange={(e) => setForm({ ...form, keyPath: e.target.value })}
                    placeholder="C:\Users\me\.ssh\id_ed25519"
                  />
                </label>
                <label className="settings-field">
                  <span>Passphrase (optional)</span>
                  <input
                    type="password"
                    value={form.passphrase}
                    onChange={(e) => setForm({ ...form, passphrase: e.target.value })}
                  />
                </label>
              </>
            )}
            {result && (
              <div className={`ssh-result ${result.ok ? "ok" : "fail"}`}>
                {result.ok ? `connected${result.banner ? ` — ${result.banner}` : ""}` : result.error}
              </div>
            )}
            <div className="ssh-actions">
              <button className="settings-small-button" onClick={() => void onTest()} disabled={testing}>
                {testing ? "testing…" : "Test connection"}
              </button>
              <button className="settings-small-button" onClick={() => void onSave()}>
                Save
              </button>
              <button className="settings-small-button primary" onClick={() => void onConnect()}>
                <Plug size={12} /> Connect
              </button>
            </div>
          </div>
          <div className="ssh-saved">
            <div className="ssh-saved-header">Saved connections</div>
            {saved.length === 0 && <div className="ssh-saved-empty">nothing saved yet</div>}
            {saved.map((conn) => (
              <div key={conn.id} className="ssh-saved-row">
                <button className="ssh-saved-button" onClick={() => useSaved(conn)} title="Load">
                  <span className="ssh-saved-name">{conn.name}</span>
                  <span className="ssh-saved-target">
                    {conn.user}@{conn.host}:{conn.port}
                  </span>
                </button>
                <button
                  className="icon-button"
                  title="Delete"
                  onClick={async () => {
                    await sshDelete(conn.id);
                    setSaved(await sshList());
                  }}
                >
                  <Trash2 size={12} />
                </button>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
