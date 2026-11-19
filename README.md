<div align="center">

<img src="src-tauri/icons/icon.png" width="96" height="96" alt="FileFlow icon" />

# FileFlow Desktop

**Convierte cualquier archivo. No subas nada.**

Conversor de archivos local, offline y sin cuentas — impulsado por Rust nativo dentro de una app de escritorio Tauri.

[![Versión](https://img.shields.io/badge/versión-0.1.0-00e5ff?style=flat-square)](https://github.com/tu-usuario/fileflow-desktop/releases)
[![Plataforma](https://img.shields.io/badge/plataforma-Windows-0078d4?style=flat-square&logo=windows)](https://github.com/tu-usuario/fileflow-desktop/releases)
[![Licencia](https://img.shields.io/badge/licencia-MIT-green?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2-24c8db?style=flat-square&logo=tauri)](https://tauri.app/)

</div>

---

## ¿Qué es FileFlow Desktop?

FileFlow Desktop es una aplicación nativa para Windows que convierte documentos, imágenes, hojas de cálculo, presentaciones, audio, vídeo y archivos comprimidos **sin enviar nada a ningún servidor**. Todo el procesamiento ocurre en tu propio equipo gracias a un motor escrito en Rust, empaquetado directamente dentro del ejecutable.

- 🔒 **Privacidad total** — Los archivos nunca abandonan tu máquina.
- ✈️ **100 % offline** — Funciona sin conexión a internet.
- ⚡ **Rápido** — Motor Rust nativo con perfiles de release optimizados (`lto = true`, `opt-level = 3`).
- 🆓 **Sin cuentas, sin paywalls, sin suscripciones.**

---

## Descarga

| Archivo | Descripción | Tamaño |
|---|---|---|
| `FileFlow-Setup.exe` | **Instalador recomendado** — instala la app con accesos directos | ~3.5 MB |
| `FileFlow.exe` | **Portable** — ejecuta sin instalación, llévalo en un USB | ~10.7 MB |

> Los instaladores están disponibles en [Releases](https://github.com/tu-usuario/fileflow-desktop/releases).

---

## Formatos soportados

**50 formatos de entrada · 476 rutas de conversión · 7 categorías**

| Categoría | Formatos de entrada | Rutas |
|---|---|---|
| 📄 Documentos | DOCX, DOC, ODT, RTF, TXT, MD, HTML, PDF, EPUB | 69 |
| 🖼️ Imágenes | JPG, PNG, WEBP, BMP, GIF, TIFF, HEIC, SVG, AVIF, ICO | 79 |
| 📊 Hojas de cálculo | XLSX, XLS, ODS, CSV, TSV, JSON, XML, YAML, SQL | 81 |
| 🎞️ Presentaciones | PPTX, PPT, ODP | 33 |
| 🎵 Audio | MP3, WAV, FLAC, AAC, OGG, M4A, WMA, OPUS, AIFF | 72 |
| 🎬 Vídeo | MP4, MOV, AVI, MKV, WEBM, WMV, FLV | 133 |
| 🗜️ Comprimidos | ZIP, RAR, 7Z | 9 |

La mayoría de conversiones se ejecutan en **Rust puro** sin dependencias externas. Algunas rutas avanzadas delegan en herramientas opcionales del sistema:

| Herramienta | Para qué se usa | Obligatoria |
|---|---|---|
| [FFmpeg](https://ffmpeg.org/) | Audio, vídeo, HEIC/AVIF | No |
| [LibreOffice](https://www.libreoffice.org/) | DOC, PPT, PPTX, ODP y diapositivas a imagen | No |
| [Ghostscript](https://www.ghostscript.com/) | PDF → JPG / PNG | No |
| [WinRAR](https://www.win-rar.com/) | Crear y extraer RAR | No |
| [7-Zip](https://www.7-zip.org/) | Crear y extraer 7Z | No |

Si una herramienta no está instalada, las rutas que dependen de ella no aparecen en la interfaz. El resto del catálogo sigue funcionando.

---

## Capturas de pantalla

> *(Agrega tus capturas de pantalla aquí una vez que subas las imágenes al repositorio.)*

---

## Desarrollo local

### Requisitos previos

- [Node.js 20+](https://nodejs.org/) y [pnpm](https://pnpm.io/)
- [Rust stable](https://rustup.rs/) + [dependencias de Tauri para Windows](https://tauri.app/start/prerequisites/)
- WebView2 (incluido en Windows 10/11 — no requiere instalación manual)

### Instalación

```bash
git clone https://github.com/tu-usuario/fileflow-desktop.git
cd fileflow-desktop
pnpm install
```

### Comandos

```bash
# Desarrollo con recarga en caliente (Vite + Tauri)
pnpm tauri dev

# Solo frontend (sin motor nativo — para trabajar en UI)
pnpm dev

# Verificar tipos TypeScript
pnpm typecheck

# Linting
pnpm lint

# Tests del motor Rust
cargo test --manifest-path src-tauri/Cargo.toml

# Build de producción (genera FileFlow.exe y FileFlow-Setup.exe)
pnpm tauri build
```

---

## Arquitectura

```
fileflow-desktop/
├── src/                         # Interfaz React 19 + TypeScript
│   ├── components/              # Componentes reutilizables (Icon, FormatBadge, layout/)
│   ├── features/                # Vistas por funcionalidad (converter/, privacy/, faq/)
│   ├── lib/
│   │   ├── tauri.ts             # Puente IPC tipado — único lugar con invoke()
│   │   ├── formats.ts           # Matriz de formatos y helpers de validación
│   │   └── errors.ts            # Normalización de errores frontend
│   ├── styles/                  # Tokens de diseño y estilos modulares
│   └── types/                   # Tipos TypeScript compartidos (mirror de DTOs Rust)
│
└── src-tauri/
    ├── src/
    │   ├── lib.rs               # Comandos Tauri: inspect, convert, engine_info, conversion_matrix
    │   └── convert/             # Motor de conversión en Rust
    │       ├── catalog/         # Catálogo central de formatos y matriz de rutas
    │       ├── documents/       # DOCX, ODT, RTF, TXT, MD, HTML, PDF, EPUB
    │       ├── images/          # JPG, PNG, WEBP, BMP, GIF, TIFF, SVG, HEIC, AVIF, ICO
    │       ├── sheets/          # XLSX, XLS, ODS, CSV, TSV, JSON, XML, YAML, SQL
    │       ├── presentations.rs # PPTX, PPT, ODP
    │       ├── audio/           # MP3, WAV, FLAC, AAC, OGG, M4A, WMA, OPUS, AIFF
    │       ├── video.rs         # MP4, MOV, AVI, MKV, WEBM, WMV, FLV
    │       └── archives.rs      # ZIP, RAR, 7Z
    └── capabilities/            # Permisos Tauri (principio de mínimo privilegio)
```

**Principios de diseño:**
- Todo `invoke()` pasa por `src/lib/tauri.ts` — los componentes nunca importan `@tauri-apps/api` directamente.
- Los comandos Rust son delgados: validan entrada, llaman al conversor, mapean errores.
- Ningún archivo de código fuente supera las 500 líneas.
- TypeScript en modo `strict: true`. Sin `any` sin justificación.

---

## Privacidad

FileFlow Desktop no incluye ningún tipo de telemetría, rastreo ni llamada de red. La única actividad de red que hace el ejecutable es la comunicación interna entre WebView2 y el runtime de Tauri (IPC local). No se envía ningún archivo, metadato ni dato de uso a terceros.

---

## Stack tecnológico

| Capa | Tecnología |
|---|---|
| Shell de escritorio | [Tauri 2](https://tauri.app/) |
| Motor / conversión | [Rust](https://www.rust-lang.org/) |
| Interfaz | [React 19](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) strict |
| Bundler | [Vite](https://vite.dev/) |
| Gestor de paquetes | [pnpm](https://pnpm.io/) |
| Estilos | Tailwind v4 + CSS design tokens |
| Tipografía | [Geist](https://vercel.com/font) + Geist Mono |

---

## Licencia

MIT © 2024 — FileFlow Desktop
