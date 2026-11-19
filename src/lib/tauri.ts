import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open, save } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import type { ConversionMatrix, ConvertOptions, EngineInfo, FileInfo } from "../types";
import { EXTENSIONS, OUTPUT_EXTENSION } from "./formats";

export function isDesktopApp(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function inspectFile(path: string): Promise<FileInfo> {
  return invoke<FileInfo>("inspect", { path });
}

export function convertFile(
  path: string,
  target: string,
  outputPath: string,
  options?: ConvertOptions,
): Promise<void> {
  return invoke<void>("convert", {
    path,
    target,
    outputPath,
    options: options ?? null,
  });
}

export function getEngineInfo(): Promise<EngineInfo> {
  return invoke<EngineInfo>("engine_info");
}

export function getConversionMatrix(): Promise<ConversionMatrix> {
  return invoke<ConversionMatrix>("conversion_matrix");
}

export async function pickSourceFile(sourceFormat?: string): Promise<string | null> {
  const allExtensions = Object.values(EXTENSIONS).flat();
  const extensions = sourceFormat ? EXTENSIONS[sourceFormat] ?? allExtensions : allExtensions;
  const selected = await open({
    multiple: false,
    directory: false,
    title: sourceFormat ? `Abrir archivo ${sourceFormat}` : "Abrir archivo",
    filters: [{ name: sourceFormat ? `Archivos ${sourceFormat}` : "Archivos compatibles", extensions }],
  });
  return typeof selected === "string" ? selected : null;
}

export async function pickDestinationPath(sourcePath: string, targetFormat: string): Promise<string | null> {
  const extension = OUTPUT_EXTENSION[targetFormat] ?? targetFormat.toLowerCase();
  const selected = await save({
    title: "Guardar conversión",
    defaultPath: suggestOutputPath(sourcePath, extension),
    filters: [{ name: targetFormat, extensions: [extension] }],
  });
  return selected;
}

export async function showItemInFolder(path: string): Promise<void> {
  await revealItemInDir(path);
}

export async function listenFileDrops(
  onDrop: (path: string) => void,
  onHover: (active: boolean) => void,
): Promise<() => void> {
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
      const path = "paths" in payload && Array.isArray(payload.paths) ? payload.paths[0] : undefined;
      if (path) onDrop(path);
    }
  });
}

export function suggestOutputPath(sourcePath: string, extension: string): string {
  const file = sourcePath.split(/[/\\]/).pop() ?? "archivo";
  const stem = file.replace(/\.[^/.]+$/, "") || "archivo";
  const folder = sourcePath.slice(0, Math.max(0, sourcePath.length - file.length));
  return `${folder}${stem}.${extension}`;
}
