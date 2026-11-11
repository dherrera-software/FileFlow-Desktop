import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open, save } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";

export type FileInfo = {
  name: string;
  format: string;
  size: number;
  targets: string[];
  width: number | null;
  height: number | null;
};

export type EngineInfo = {
  name: string;
  version: string;
  platform: string;
  maxBytes: number;
  formats: number;
  conversions: number;
};

const EXTENSIONS: Record<string, string[]> = {
  TXT: ["txt"],
  HTML: ["html", "htm"],
  MD: ["md", "markdown"],
  JPG: ["jpg"],
  JPEG: ["jpeg"],
  PNG: ["png"],
  CSV: ["csv"],
  TSV: ["tsv"],
  JSON: ["json"],
  XML: ["xml"],
};

const ALL_EXTENSIONS = Object.values(EXTENSIONS).flat();

export function isDesktopApp(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / 1024 ** index;
  const digits = index === 0 || value >= 10 ? 0 : 1;
  return `${value.toFixed(digits)} ${units[index]}`;
}

export function errorMessage(error: unknown): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message.trim()) return error.message;
  return "No se pudo completar la operación.";
}

export function inspect(path: string): Promise<FileInfo> {
  return invoke<FileInfo>("inspect", { path });
}

export function convert(path: string, target: string, outputPath: string): Promise<void> {
  return invoke<void>("convert", { path, target, outputPath });
}

export function engineInfo(): Promise<EngineInfo> {
  return invoke<EngineInfo>("engine_info");
}

export async function pickSource(source?: string): Promise<string | null> {
  const extensions = source ? EXTENSIONS[source] ?? ALL_EXTENSIONS : ALL_EXTENSIONS;
  const selected = await open({
    multiple: false,
    directory: false,
    title: source ? `Abrir archivo ${source}` : "Abrir archivo",
    filters: [{ name: source ? `Archivos ${source}` : "Archivos compatibles", extensions }],
  });
  return typeof selected === "string" ? selected : null;
}

export async function pickDestination(sourcePath: string, target: string): Promise<string | null> {
  const selected = await save({
    title: "Guardar conversión",
    defaultPath: suggestedPath(sourcePath, target),
    filters: [{ name: target, extensions: [target.toLowerCase()] }],
  });
  return selected;
}

export async function showInFolder(path: string): Promise<void> {
  await revealItemInDir(path);
}

export async function listenForDrops(onDrop: (path: string) => void, onHover: (active: boolean) => void): Promise<() => void> {
  const webview = getCurrentWebview();
  return webview.onDragDropEvent((event) => {
    const payload = event.payload;
    if (payload.type === "leave") {
      onHover(false);
      return;
    }
    if (payload.type === "enter" || payload.type === "over") {
      onHover(true);
      return;
    }
    if (payload.type === "drop") {
      onHover(false);
      const path = "paths" in payload ? payload.paths[0] : undefined;
      if (path) onDrop(path);
    }
  });
}

function suggestedPath(sourcePath: string, target: string): string {
  const file = sourcePath.split(/[/\\]/).pop() ?? "archivo";
  const stem = file.replace(/\.[^/.]+$/, "") || "archivo";
  const folder = sourcePath.slice(0, Math.max(0, sourcePath.length - file.length));
  return `${folder}${stem}.${target.toLowerCase()}`;
}
