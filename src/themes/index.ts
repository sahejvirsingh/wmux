import { daylight } from "./daylight";
import { dracula } from "./dracula";
import { midnight } from "./midnight";
import { nord } from "./nord";
import type { Theme } from "./types";

export const builtinThemes: Theme[] = [midnight, daylight, nord, dracula];

export function findTheme(id: string, custom: Theme[] = []): Theme {
  return [...builtinThemes, ...custom].find((t) => t.id === id) ?? midnight;
}

export function allThemes(custom: Theme[] = []): Theme[] {
  return [...builtinThemes, ...custom];
}

export * from "./types";