import { useEffect } from "react";
import { Effect, EffectState, getCurrentWindow } from "@tauri-apps/api/window";
import { AppShell } from "./components/layout/AppShell";
import { useKeybindings } from "./hooks/useKeybindings";
import { useTheme } from "./hooks/useTheme";
import { isTauri } from "./lib/tauri-bridge";
import { useAppStore } from "./stores/appStore";
import { useSettingsStore } from "./stores/settingsStore";

export default function App() {
  const initApp = useAppStore((s) => s.init);
  const loadSettings = useSettingsStore((s) => s.load);
  useTheme();
  useKeybindings();

  useEffect(() => {
    if (isTauri()) {
      void initApp();
      void loadSettings();
      try {
        void getCurrentWindow().setEffects({
          effects: [Effect.Mica],
          state: EffectState.Active,
        });
      } catch {
        /* mica unsupported on this system */
      }
    }
  }, [initApp, loadSettings]);

  return <AppShell />;
}
