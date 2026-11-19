# TODO.md â€” FileFlow Desktop

Seguimiento de tareas y estado del proyecto.

## Completado

- [x] ReestructuraciÃ³n integral Senior del proyecto: modularizaciÃ³n de frontend y backend para que **el 100% de los archivos tenga â‰¤ 500 lÃ­neas de cÃ³digo**.
  - `src/App.css` (1641 lÃ­neas) desglosado en `src/styles/` (`fonts.css`, `base.css`, `layout.css`, `views.css`, `converter.css`, `badges.css`, `catalog.css`, `info-views.css`).
  - `src/App.tsx` (492 lÃ­neas) desglosado en `Sidebar.tsx`, `Topbar.tsx`, `ConverterView.tsx`, `PrivacyView.tsx`, `FaqView.tsx`.
  - `convert/mod.rs` (703 lÃ­neas) desglosado en `convert/tests.rs` y `convert/mod.rs` (439 lÃ­neas).
  - `convert/catalog.rs` (679 lÃ­neas) modularizado en `convert/catalog/` (`formats.rs`, `targets.rs`, `matrix.rs`, `tests.rs`, `mod.rs`).
  - `convert/documents/office.rs` (777 lÃ­neas) modularizado en `convert/documents/office/` (`docx.rs`, `odt.rs`, `rtf.rs`, `epub.rs`, `zip_utils.rs`, `tests.rs`, `mod.rs`).
  - `convert/sheets/mod.rs` (700 lÃ­neas) modularizado en `convert/sheets/` (`biff8.rs`, `writers.rs`, `readers.rs`, `tests.rs`, `mod.rs`).
  - `convert/sheets/structured.rs` (660 lÃ­neas) modularizado en `convert/sheets/structured/` (`yaml.rs`, `sql.rs`, `sql_parser.rs`, `tests.rs`, `mod.rs`).
  - `convert/documents/pdf.rs` (528 lÃ­neas) modularizado en `convert/documents/pdf/` (`render.rs`, `writer.rs`, `tests.rs`, `mod.rs`).
- [x] VerificaciÃ³n completa: `node line-count` (0 archivos > 500 lÃ­neas), `pnpm typecheck` (0 errores), `pnpm lint` (0 errores), `pnpm build` (ok), `cargo fmt` (ok), `cargo clippy -- -D warnings` (0 advertencias), `cargo test` (**74 de 74 tests pasados exitosamente**).
- [ ] Esperar revisiÃ³n del usuario de los cambios arquitectÃ³nicos antes de generar los `.exe` finales (`FileFlow.exe` y `FileFlow-Setup.exe`).

- [x] Auditar conversiones activas del producto web (TXT, HTML, MD, JPG, PNG, CSV, TSV, JSON, XML) y contrastar con catÃ¡logo Rust.
- [x] Crear `src/types/index.ts` con los tipos espejo de los DTOs de Rust (`FileInfo`, `EngineInfo`, `AudioParams`, `ConversionState`, etc.).
- [x] Crear `src/lib/tauri.ts` como Ãºnico punto de invocaciÃ³n IPC (`invoke`) desacoplando componentes.
- [x] Crear `src/lib/formats.ts` con la matriz de formatos, helpers de bytes/duraciÃ³n y accesos populares.
- [x] Crear `src/lib/errors.ts` para normalizar errores con cÃ³digos estables (`FILE_NOT_FOUND`, `CORRUPT_FILE`, `PERMISSION_DENIED`, etc.).
- [x] Crear `src/styles/tokens.css` eliminando el Ã­ndigo obsoleto e implementando el sistema estricto dual theme con acento lima (`#c8f96b` en oscuro, `#3b6605` en claro para AA contrast).
- [x] Refactorizar la UI en componentes modulares accesibles (`DropZone`, `FormatSelector`, `AudioSettings`, `ConversionCard`, `Icon`, `FormatBadge`) con estados tipados (`idle | validating | selected | converting | done | error | cancelled`).
- [x] Actualizar `package.json` con scripts `"typecheck": "tsc --noEmit"` y `"lint": "tsc --noEmit"`.
- [x] Resolver advertencias de `clippy` (`collapsible_match`, `needless_borrow`, `too_many_arguments`).
- [x] Verificar pipeline de calidad:
  - `pnpm typecheck` (0 errores)
  - `pnpm lint` (0 errores)
  - `pnpm build` (compilaciÃ³n y empaquetado Vite exitosos)
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` (0 diferencias)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` (0 advertencias)
  - `cargo test --manifest-path src-tauri/Cargo.toml` (74 de 74 pruebas unitarias pasadas)
- [x] Documentar evidencia de auditorÃ­a y pruebas en `DEV_LOG.md`.
- [x] Portar el delta del catÃ¡logo web: PDFâ†’DOCX/JPG/PNG/XLSX/CSV, DOC, PPTX/PPT/ODP, YAML/SQL, ICO, vÃ­deo, ZIP/RAR/7Z y TXTâ†’MP3.
- [x] Excluir PDF/A del catÃ¡logo por falta de implementaciÃ³n real y de perfil ICC.
- [x] Filtrar destinos por herramienta instalada (FFmpeg, LibreOffice, Ghostscript, WinRAR, 7-Zip, voz de Windows).
- [x] Ampliar pruebas Rust a 73 (nuevas rutas, seguridad de archivos comprimidos, ida y vuelta RAR y contador real).
- [x] Actualizar la UI a 7 categorÃ­as y 50 formatos, con el contador de rutas real del equipo.
- [x] Exponer la matriz completa desde Rust (`conversion_matrix`) y renderizarla en la UI con bÃºsqueda, filtros por categorÃ­a y rutas no disponibles marcadas con su herramienta.
- [x] Los accesos rÃ¡pidos y los destinos del catÃ¡logo abren el selector de archivos con la ruta ya preparada.
- [x] Verificar `pnpm typecheck`, `pnpm lint`, `pnpm build`, `cargo fmt`, `cargo clippy -- -D warnings` y `cargo test` (74/74).
- [ ] Verificar en un equipo con LibreOffice: DOCâ†’PDF/TXT, PPTâ†’PPTX, PPTXâ†”ODP y diapositivasâ†’JPG/PNG.
- [ ] Verificar PDFâ†’JPG/PNG con Ghostscript instalado.
- [ ] Verificar ZIP/RAR/7Z con 7-Zip instalado, incluidos archivos con contraseÃ±a (deben rechazarse con mensaje claro).
- [ ] Verificar la app empaquetada en macOS y Linux (rutas y detecciÃ³n de herramientas).
- [ ] Decidir si se implementa PPTXâ†’MP4 (requiere LibreOffice + Ghostscript + FFmpeg) antes de declararlo.

## PrÃ³ximos pasos (Backlog opcional)

- [x] Empaquetado de binarios e instalador con `pnpm tauri build` (.exe portable + instalador NSIS + MSI).
- [ ] Soporte para conversiÃ³n de archivos por lote (batch queue) si el usuario lo solicita.

