export type CategoryKey =
  | "document"
  | "image"
  | "sheet"
  | "presentation"
  | "audio"
  | "video"
  | "archive";

export interface FileInfo {
  name: string;
  format: string;
  category: CategoryKey;
  categoryLabel: string;
  mime: string;
  size: number;
  targets: string[];
  width: number | null;
  height: number | null;
  durationMs: number | null;
}

export interface EngineInfo {
  name: string;
  version: string;
  platform: string;
  maxBytes: number;
  formats: number;
  conversions: number;
}

export interface MatrixTarget {
  format: string;
  available: boolean;
  requires: string | null;
}

export interface MatrixFormat {
  format: string;
  extensions: string[];
  targets: MatrixTarget[];
}

export interface MatrixCategory {
  key: CategoryKey;
  label: string;
  formats: MatrixFormat[];
}

export interface ConversionMatrix {
  categories: MatrixCategory[];
  declared: number;
  available: number;
}

export interface AudioParams {
  bitrateKbps: number;
  sampleRateHz: number;
  channels: number;
}

export interface ConvertOptions {
  audio?: AudioParams;
}

export type Theme = "dark" | "light";

export interface RouteRequest {
  from: string;
  to: string;
  nonce: number;
}

export type ConversionState =
  | { status: "idle" }
  | { status: "validating"; path: string }
  | { status: "selected"; file: FileInfo; sourcePath: string; target: string }
  | { status: "converting"; file: FileInfo; sourcePath: string; target: string; progressLabel: string }
  | { status: "done"; file: FileInfo; sourcePath: string; target: string; outputPath: string }
  | { status: "error"; message: string; code?: string }
  | { status: "cancelled" };
