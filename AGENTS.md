# AGENTS.md — FileFlow Desktop

## 1. Product Context

**FileFlow Desktop** is the native desktop version of FileFlow: a local-first,
privacy-first file converter. It converts documents, images, and data files
without uploading anything to a server, without accounts, and without requiring
an internet connection.

Core promise: **“Convert anything. Upload nothing.”**

The desktop app exists because native Rust conversion is faster, handles larger
files, and removes the web app’s 25 MB / browser-memory constraints. The web
version may keep a limited conversion matrix; the desktop app is the
high-fidelity, offline, full-quality product.

Current web conversion surface (for reference, not a hard desktop limit):
TXT, HTML, MD, JPG/JPEG, PNG, CSV, TSV, JSON, XML — 13 conversions. Desktop may
add more formats through native Rust engines, but the UI must never expose a
combination the Rust backend cannot actually complete.

### Non-negotiable product truths

- **Local by default.** Files never leave the machine. No uploads, no cloud
  conversion, no telemetry containing file contents or paths.
- **Privacy is the feature.** Do not add analytics, remote logging, crash
  reporting, or “phone home” behavior without explicit user approval.
- **Offline capable.** The app must work with the network disabled.
- **No accounts, no paywalls, no forced sign-in.**
- **Fidelity matters.** A conversion that silently corrupts a file is worse
  than a conversion that refuses to run. Prefer explicit errors over lossy
  guesses.

---

## 2. Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | **Tauri 2** |
| Backend / conversion engine | **Rust** |
| Frontend | **React 19 + TypeScript (strict)** |
| Bundler | **Vite** |
| Package manager | **pnpm** (workspace-aware) |
| Styling | **Tailwind v4 utility layer + CSS design tokens** |
| Fonts | **Geist** (UI) + **Geist Mono** (code, paths, technical values) |
| State | React local state + typed stores only where needed. Do not add a global state library without asking. |
| Persistence | Tauri app-data directory (settings) and OS temp directory (conversion scratch files). No database unless explicitly approved. |

### Hard constraints

- Do **not** import code from the web `server/` (Cloudflare Worker / Hono) into
  the desktop app. The desktop backend is Rust, not the Worker.
- Do **not** call the remote web API from the desktop app. Desktop conversions
  are local.
- Do **not** use `fs`, `path`, or Node APIs in the renderer. The renderer is a
  webview; all filesystem access goes through Tauri commands.
- TypeScript must remain `strict: true`. No `any` without a written reason in
  the same file. No non-null assertions unless the invariant is proven next to
  the assertion.

---

## 3. Repository Layout

Expected structure for the desktop app. If the actual repository differs,
**read the real tree first and update this section instead of inventing paths**.

```text
fileflow-desktop/
├── src/                         # React renderer
│   ├── components/              # Reusable UI (Icon, Badge, Spinner, etc.)
│   ├── features/                # Feature-scoped components + hooks
│   │   └── converter/           # DropZone, FormatSelector, ConversionCard, etc.
│   ├── hooks/                   # Shared React hooks (drops, theme, etc.)
│   ├── lib/
│   │   ├── tauri.ts             # Typed IPC bridge — the ONLY place invoke() is called
│   │   ├── formats.ts           # Supported format matrix + validation helpers
│   │   └── errors.ts            # Frontend error normalization
│   ├── styles/
│   │   ├── tokens.css           # Design tokens (dual theme: near-black & off-white + lime accent)
│   │   └── index.css            # Global styles and utilities
│   ├── types/                   # Shared TS types (mirrors Rust DTOs)
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs               # Tauri entry point, #[tauri::command] surface & DTOs
│   │   └── convert/             # Native conversion engine modules
│   │       ├── audio/           # Audio codecs & FFmpeg bridge
│   │       ├── documents/       # DOCX, ODT, RTF, TXT, MD, HTML, PDF, EPUB, DOC, TTS
│   │       ├── images/          # JPG, PNG, WEBP, BMP, GIF, TIFF, SVG, HEIC, AVIF, ICO
│   │       ├── sheets/          # XLSX, XLS, ODS, CSV, TSV, JSON, XML, YAML, SQL
│   │       ├── archives.rs      # ZIP, RAR and 7Z extraction/repacking
│   │       ├── catalog.rs       # Central format catalog and route matrix
│   │       ├── external.rs      # Subprocess execution (FFmpeg/LibreOffice/Ghostscript/7-Zip)
│   │       ├── presentations.rs # PPTX, PPT and ODP
│   │       ├── support.rs       # Encodings, BOM detection, separators, signatures
│   │       ├── video.rs         # Video transcode, frames, GIF, audio extraction
│   │       └── mod.rs           # Core convert_file, inspect_file APIs
│   ├── capabilities/            # Least-privilege permission files (default.json)
│   ├── icons/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/
│   └── AUDITORIA-FORMATOS.md    # Master format audit and contract
├── TODO.md
├── DEV_LOG.md
├── package.json
└── pnpm-lock.yaml
```

### Rules

- Keep the existing folder architecture. Do not create parallel “v2”, “new”, or
  “temp” directories. If a new folder is needed, explain why first.
- Components in `src/components` are presentational and reusable. Feature
  behavior belongs in `src/features`.
- All Tauri `invoke()` calls live in `src/lib/tauri.ts`. Components must not
  import `@tauri-apps/api/core` directly.
- Rust commands are thin: validate input, call a converter/service, map errors.
  Conversion logic does not live inside `commands/`.

---

## 4. Architecture Rules

### 4.1 IPC contract

- Define request and response types in Rust with `serde` and mirror them in
  `src/types/`. Keep names identical.
- Every command returns a typed `Result<T, AppError>`. `AppError` serializes to
  a stable `{ code, message, details? }` shape.
- Frontend errors are normalized in `src/lib/errors.ts`; components render
  user-facing messages by error code, never by raw Rust strings.
- Long-running conversions must emit progress events. Support cancellation.
- Never block the Tauri async runtime with CPU-heavy conversion work. Use
  `tauri::async_runtime::spawn_blocking` or a dedicated worker for blocking
  codecs.

### 4.2 Filesystem boundaries

- The renderer may only receive safe display metadata (file name, size,
  detected format). It must not receive unrestricted absolute paths unless the
  user explicitly selected that file and the path is needed for display.
- Use Tauri’s dialog plugin for file selection. Do not build custom path input.
- Write conversion output to a temp location inside the app’s temp directory,
  then move/copy to the user-chosen destination. Clean up scratch files on
  success, failure, and app exit.
- Validate extension, MIME hint, and file size before invoking a converter.
- Handle symlinks conservatively. Do not follow symlinks outside the selected
  scope.
- Cross-platform paths: use `std::path::PathBuf`, never hardcode `/` or `\`.
  Test path behavior on Windows, macOS, and Linux.

### 4.3 Rust safety and quality

- No `unwrap()` or `expect()` in command paths, converters, or anything that can
  fail on user input. Use `?` with `AppError`.
- No `panic!` for recoverable errors. Panics are reserved for true invariants.
- Capability files must be least-privilege. Do not enable the shell plugin or
  broad filesystem scopes “just in case”.
- Run `cargo fmt` and `cargo clippy -- -D warnings` before considering Rust work
  done.
- New dependencies require a reason: binary size, license, maintenance status,
  and cross-platform support must be considered.

### 4.4 React quality

- Type all props and state. No implicit `any`. Prefer discriminated unions for
  conversion states: `idle | validating | converting | done | error | cancelled`.
- Do not duplicate the format matrix in multiple components. It lives in
  `src/lib/formats.ts` and is derived from backend-supported combinations.
- Avoid `useEffect` for derived state. Compute during render or use a selector.
- Components must be keyboard-operable and screen-reader friendly. Every icon
  button needs an accessible label.

---

## 5. Design System — Dual Theme, One Accent

FileFlow uses a dual theme (dark default, light equivalent) with a single
chromatic accent. Do not reintroduce the old indigo palette or mix additional
brand colors.

### Tokens

Source of truth: `src/styles/tokens.css`

| Token role | Dark | Light |
|---|---|---|
| Canvas / app background | near-black `#08090a` | off-white (light equivalent) |
| Surface ladder | layered elevated surfaces from canvas | layered elevated surfaces from canvas |
| Primary accent | lime `#c8f96b` | equivalent accessible lime |
| Text primary | high-contrast near-white | high-contrast near-black |
| Text muted | AA-compliant muted | AA-compliant muted |
| Border | subtle low-opacity line | subtle low-opacity line |

**Accent usage:** primary actions, focus rings, active states, progress.
Do not flood surfaces with lime. Lime is a signal, not a background.

- **Typography:** Geist for UI, Geist Mono for file names, extensions, paths,
  conversion codes, and technical metadata. Max display weight: 600.
- **Motion:** micro-interactions only. Hover lift of `-2px`, scale `1.02`,
  opacity/color transitions. No scroll-triggered animations. Respect
  `prefers-reduced-motion`.
- **Focus:** global `:focus-visible` ring using the accent token. Never remove
  focus outlines without an accessible replacement.
- **Contrast:** WCAG AA minimum. Verify both themes, including muted text and
  small labels.
- **No inline styles.** No `style={{ ... }}`. Use tokens and utility classes.
  Dynamic values that truly cannot be expressed in CSS must use a CSS custom
  property set through a class, not inline React styles.

### UI principles

- Calm, dense, professional. FileFlow is a tool, not a marketing page.
- The drop zone is the primary interaction. It must be obvious, keyboard
  accessible, and usable without drag-and-drop.
- Show only conversions that are valid for the detected input format.
- Never show a spinner without a label or progress context for conversions that
  take longer than a moment.
- Errors explain what failed and what the user can do next. No raw error dumps.

---

## 6. Agent Behavior Rules (Critical)

- **Zero invention — ask, don’t assume.** If a file path, API, Rust crate, Tauri
  capability, conversion behavior, or design token is unknown, ask or read the
  actual file. Do not hallucinate code, imports, schemas, or file contents. Do
  not claim a command works if you have not run it or read the implementation.
- **Read before writing.** Before modifying a file, read it. Before adding a
  dependency, check whether the project already has an equivalent. Before
  changing architecture, inspect the real tree.
- **Keep memory yourself.** Maintain `DEV_LOG.md` for decisions and verification
  evidence. Use `TODO.md` for pending work. Do not rely on conversation memory
  across sessions.
- **Clean, modular code.** No spaghetti. Extract shared logic into hooks,
  utilities, or Rust services. Keep components and commands small and testable.
- **Strict TypeScript.** Type every prop, state, event, and IPC payload. Prefer
  explicit types over inference at module boundaries.
- **No inline styles.** Prohibited: `style={{ ... }}`. Use CSS classes, Tailwind
  utilities, and design tokens.
- **Respect the structure.** Do not create new top-level folders, rename
  existing ones, or move files without explaining the reason and getting
  confirmation if the change is structural.
- **Do not break privacy.** Never add a network call, telemetry event, remote
  font, CDN dependency, or external API to the desktop runtime without explicit
  approval.
- **Verify, don’t assume.** A task is not done because the code “looks right”.
  It is done when the relevant checks pass and the behavior has been exercised.
- **Report uncertainty.** If a conversion cannot be implemented safely with the
  current stack, say so and propose alternatives. Do not ship a silent
  best-effort converter for a format that requires exact fidelity.

---

## 7. Commands and Verification

Use the project’s actual scripts. If a script name differs, read `package.json`
and update this section.

```bash
# Install
pnpm install

# Desktop dev (Vite + Tauri)
pnpm tauri dev

# Frontend only (for UI work that does not need native APIs)
pnpm dev

# Type checking
pnpm typecheck

# Lint
pnpm lint

# Rust formatting and linting
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Production build
pnpm tauri build
```

### Verification expectations

- **Frontend changes:** run `pnpm typecheck` and `pnpm lint`. For layout or
  responsive changes, run the app and verify at desktop, tablet, and narrow
  widths. Check `clientWidth` vs `scrollWidth` to catch horizontal overflow.
- **Rust changes:** run `cargo fmt`, `cargo clippy -- -D warnings`, and
  `cargo test`. Exercise the command through the actual UI, not only unit tests.
- **Conversion changes:** test with real files, including edge cases: empty
  file, wrong extension, oversized file, Unicode file name, path with spaces,
  and cancellation mid-conversion.
- **Theme changes:** verify both dark and light themes, focus states, and AA
  contrast.
- **Cross-platform claims:** do not claim Windows/macOS/Linux support unless the
  path handling and build have been checked on that platform or explicitly noted
  as untested.
- Record verification evidence in `DEV_LOG.md` with date, command, and result.

---

## 8. Definition of Done

A task is done only when all applicable items are true:

- [ ] The behavior matches the product promise and does not add network/cloud
      dependency.
- [ ] Types are strict; no unexplained `any`.
- [ ] No inline styles; tokens and classes are used.
- [ ] Renderer does not touch the filesystem directly.
- [ ] IPC payloads are typed on both sides and errors are normalized.
- [ ] Rust has no `unwrap()`/`expect()` in user-input paths.
- [ ] `pnpm typecheck`, `pnpm lint`, `cargo fmt`, and `cargo clippy` pass.
- [ ] Relevant manual verification is recorded in `DEV_LOG.md`.
- [ ] `TODO.md` is updated if anything remains unfinished.
- [ ] Accessibility basics are covered: keyboard access, labels, focus visible,
      AA contrast in both themes.

---

## 9. Memory Protocol

Maintain these files at the repository root:

- **`DEV_LOG.md`** — dated entries for decisions, design-system changes,
  verification runs, known issues, and trade-offs.
- **`TODO.md`** — short, actionable pending items. Check off completed items; do
  not let it become a second README.

When you make a non-obvious decision, write it down immediately. When you
discover a constraint, record it. Future agents will rely on these files more
than on chat history.

---

## 10. Global Skills

Global skills are available under `~/.agents/skills/`. Use them proactively when
the task matches:

- **`design-taste-frontend`** — for premium UI work and to avoid generic
  “AI-looking” layouts.
- **`awesome-design-md`** — to extract palettes and typography only if a design
  change is requested. Do not override FileFlow’s lime + Geist system without
  approval.
- **`web-design-guidelines`** — to audit accessibility, focus management, and UX
  of new components.
- **`playwright`** — for layout and responsive verification. For Tauri-specific
  behavior, use the actual desktop window or Tauri’s WebDriver support; do not
  assume Playwright can drive native dialogs or the Rust backend.

Skills are tools, not authority. If a skill suggests a pattern that conflicts
with this file, this file wins unless the user explicitly overrides it.