import type { AppNotification } from "../types";

export interface StoredNotification extends AppNotification {
  key: number;
  ptyId: string;
  workspaceId: string;
  paneId: string;
  at: number;
}

let nextKey = 1;

export function storeNotification(
  notification: AppNotification,
  workspaceId: string,
  paneId: string,
  ptyId: string,
): StoredNotification {
  return {
    ...notification,
    key: nextKey++,
    workspaceId,
    paneId,
    ptyId,
    at: Date.now(),
  };
}

export function timeAgo(at: number): string {
  const seconds = Math.max(0, Math.floor((Date.now() - at) / 1000));
  if (seconds < 10) return "now";
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}