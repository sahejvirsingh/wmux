import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { WebglAddon } from "@xterm/addon-webgl";
import { Terminal } from "@xterm/xterm";
import { type RefObject, useEffect, useRef, useState } from "react";
import type { ITheme } from "@xterm/xterm";
import { attachPty, makeChannel, newPtyId, registerPty, resizePty, spawnPty, writePty } from "../lib/tauri-bridge";
import type { AppNotification, PtyEvent, TerminalPaneNode } from "../types";

export interface UseTerminalOptions {
  containerRef: RefObject<HTMLDivElement | null>;
  node: TerminalPaneNode;
  workspaceId: string;
  workspaceCwd: string;
  theme: ITheme;
  fontSize: number;
  fontFamily: string;
  onNotification: (notification: AppNotification) => void;
  onExit: (code: number) => void;
}

export interface TerminalHandle {
  terminal: Terminal | null;
  searchAddon: SearchAddon | null;
  ready: boolean;
  exited: boolean;
  exitCode: number | null;
  focus: () => void;
}

export function useTerminal(options: UseTerminalOptions): TerminalHandle {
  const {
    containerRef,
    node,
    workspaceId,
    workspaceCwd,
    theme,
    fontSize,
    fontFamily,
    onNotification,
    onExit,
  } = options;
  const terminalRef = useRef<Terminal | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const searchRef = useRef<SearchAddon | null>(null);
  const writeBufferRef = useRef("");
  const flushTimerRef = useRef<number | null>(null);
  const aliveRef = useRef(true);
  const ptyIdRef = useRef(node.ptyId);
  const [ready, setReady] = useState(false);
  const [exited, setExited] = useState(false);
  const [exitCode, setExitCode] = useState<number | null>(null);
  const startedRef = useRef(false);

  useEffect(() => {
    aliveRef.current = true;
    return () => {
      aliveRef.current = false;
    };
  }, []);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || terminalRef.current || startedRef.current) {
      return;
    }
    startedRef.current = true;

    const terminal = new Terminal({
      fontSize,
      fontFamily,
      theme,
      cursorBlink: true,
      allowProposedApi: true,
      scrollback: 10000,
      convertEol: false,
      windowsMode: true,
    });
    const fit = new FitAddon();
    const search = new SearchAddon();
    terminal.loadAddon(fit);
    terminal.loadAddon(search);
    terminal.loadAddon(new WebLinksAddon());
    try {
      terminal.loadAddon(new WebglAddon());
    } catch (e) {
      console.warn("webgl renderer unavailable, falling back to dom", e);
    }
    terminal.open(container);
    terminalRef.current = terminal;
    fitRef.current = fit;
    searchRef.current = search;

    const flushWrite = () => {
      flushTimerRef.current = null;
      const data = writeBufferRef.current;
      writeBufferRef.current = "";
      if (data.length > 0 && ptyIdRef.current) {
        writePty(ptyIdRef.current, data).catch(() => {});
      }
    };

    terminal.onData((data) => {
      writeBufferRef.current += data;
      if (flushTimerRef.current === null) {
        flushTimerRef.current = window.setTimeout(flushWrite, 8);
      }
    });

    try {
      fit.fit();
    } catch {
      /* container not yet sized */
    }
    const initialCols = Math.max(2, terminal.cols);
    const initialRows = Math.max(2, terminal.rows);
    setReady(true);

    const channel = makeChannel((event: PtyEvent) => {
      if (!aliveRef.current) {
        return;
      }
      switch (event.type) {
        case "output":
        case "scrollback":
          terminal.write(event.data);
          break;
        case "notification":
          onNotification(event.notification);
          break;
        case "exit":
          setExited(true);
          setExitCode(event.code);
          onExit(event.code);
          break;
        default:
          break;
      }
    });

    const boot = async () => {
      let ptyId = ptyIdRef.current;
      try {
        if (!ptyId) {
          ptyId = newPtyId();
          ptyIdRef.current = ptyId;
          await spawnPty(
            {
              ptyId,
              label: node.title,
              shell: node.shell || null,
              command: node.command || null,
              cwd: node.cwd || workspaceCwd || null,
              cols: initialCols,
              rows: initialRows,
            },
            channel,
          );
          await registerPty(workspaceId, node.id, ptyId);
        } else {
          await attachPty(workspaceId, node.id, ptyId, channel);
        }
      } catch (e) {
        console.error("failed to start pty", e);
        if (aliveRef.current) {
          terminal.writeln(`\r\n[wmux] failed to start terminal: ${String(e)}`);
        }
      }
    };
    void boot();

    const resizeObserver = new ResizeObserver(() => {
      if (!aliveRef.current) {
        return;
      }
      try {
        fit.fit();
      } catch {
        return;
      }
      const pty = ptyIdRef.current;
      if (pty && terminal.rows > 1 && terminal.cols > 1) {
        resizePty(pty, terminal.cols, terminal.rows).catch(() => {});
      }
    });
    resizeObserver.observe(container);

    return () => {
      resizeObserver.disconnect();
      if (flushTimerRef.current !== null) {
        window.clearTimeout(flushTimerRef.current);
        flushTimerRef.current = null;
      }
      const data = writeBufferRef.current;
      writeBufferRef.current = "";
      if (data.length > 0 && ptyIdRef.current) {
        writePty(ptyIdRef.current, data).catch(() => {});
      }
      terminalRef.current = null;
      fitRef.current = null;
      searchRef.current = null;
      try {
        terminal.dispose();
      } catch {
        /* already disposed */
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [node.id]);

  useEffect(() => {
    const terminal = terminalRef.current;
    if (terminal) {
      terminal.options.fontSize = fontSize;
      terminal.options.fontFamily = fontFamily;
      try {
        fitRef.current?.fit();
      } catch {
        /* not mounted */
      }
    }
  }, [fontSize, fontFamily]);

  useEffect(() => {
    const terminal = terminalRef.current;
    if (terminal) {
      terminal.options.theme = theme;
    }
  }, [theme]);

  return {
    terminal: terminalRef.current,
    searchAddon: searchRef.current,
    ready,
    exited,
    exitCode,
    focus: () => terminalRef.current?.focus(),
  };
}