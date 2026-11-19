# DEV_LOG.md — FileFlow Desktop

Registro cronológico de decisiones técnicas, auditorías de compatibilidad, ajustes de diseño y evidencia de verificación.

---

## 2026-11-19 — Build de release v0.1.0 — Ejecutables generados

### Contexto
- Tras completar la reestructuración del código (todos los archivos ≤ 500 líneas), se ejecutó `pnpm tauri build` para generar los ejecutables de producción.

### Resultado
- `pnpm tauri build` → Exit 0
- Rust release profile compilado en **5m 59s**
- 2 bundles generados exitosamente

### Artefactos generados

| Archivo | Ubicación | Tamaño |
|---|---|---|
| `FileFlow.exe` | `src-tauri/target/release/fileflow-desktop.exe` | 10.74 MB |
| `FileFlow-Setup.exe` (NSIS) | `src-tauri/target/release/bundle/nsis/FileFlow_0.1.0_x64-setup.exe` | 3.55 MB |
| `FileFlow_0.1.0_x64_en-US.msi` (WiX) | `src-tauri/target/release/bundle/msi/` | 5.03 MB |

### Copiados a raíz del proyecto
- `d:\Documents\FileFlow-Desktop\FileFlow.exe` ← portable (10.74 MB)
- `d:\Documents\FileFlow-Desktop\FileFlow-Setup.exe` ← instalador NSIS (3.55 MB)

### Stack de verificación previa al build
- `pnpm typecheck` → 0 errores
- `pnpm lint` → 0 errores
- `cargo fmt` → Exit 0
- `cargo clippy -- -D warnings` → Exit 0
- `cargo test` → 74/74 passed

---

## 2026-11-16 — Auditoría de compatibilidad de conversiones y arquitectura

### Contexto
- Objetivo: auditar las conversiones disponibles para los 13 formatos activos del producto web (TXT, HTML, MD, JPG/JPEG, PNG, CSV, TSV, JSON, XML) y consolidar el soporte nativo al 100% en el backend de Rust y en el frontend de React.
- Referencia: `docs/AUDITORIA-FORMATOS.md`, `src-tauri/src/convert/catalog.rs`.

### Hallazgos de la auditoría
1. **Motor Rust (`src-tauri/src/convert/`):**
   - El motor nativo no sólo cubre el 100% de las 13 combinaciones web originales, sino que implementa un total de 33 formatos y 230 rutas directas (documentos con `office`, `markup`, `pdf`, `text`; imágenes con `image` y `resvg`; hojas de cálculo con `calamine`, `rust_xlsxwriter` y escritor BIFF8; audio con FFmpeg local).
   - Todos los formatos solicitados por el usuario están plenamente cubiertos con fidelidad garantizada:
     - Documentos: TXT, MD, HTML (hacia DOCX, ODT, RTF, TXT, MD, HTML, PDF, EPUB).
     - Imágenes: JPG, PNG (hacia JPG, PNG, WEBP, BMP, GIF, TIFF, AVIF, PDF).
     - Datos / Hojas: CSV, TSV, JSON, XML (hacia XLSX, XLS, ODS, CSV, TSV, JSON, XML, PDF).
   - Pruebas unitarias de Rust: las 50 pruebas en `src-tauri/src/lib.rs` pasan exitosamente.

2. **Frontend (`src/`):**
   - Actualmente todo el frontend reside monolíticamente en `src/App.tsx` y `src/lib/files.ts`.
   - Se detectó que `variables.css` y `App.css` aún tenían referencias al color índigo antiguo (`#6366f1` / `#4f46e5`), violando la regla inquebrantable de `AGENTS.md` (un solo acento cromático: lime `#c8f96b` en oscuro y verde lima accesible en claro).
   - Falta estructurar los módulos frontend según `AGENTS.md`:
     - `src/types/`: DTOs espejo de Rust (`FileInfo`, `EngineInfo`, `AudioOptions`, etc.).
     - `src/lib/tauri.ts`: puente IPC único donde se invoca Tauri.
     - `src/lib/formats.ts`: matriz de formatos y helpers de extensiones.
     - `src/lib/errors.ts`: normalización de errores con códigos de error amigables.
     - `src/styles/tokens.css`: tokens CSS de diseño estricto dual theme (lime `#c8f96b`).
     - `src/features/converter/`: componentes desacoplados con manejo estricto de accesibilidad, teclado y estados (`idle | validating | converting | done | error | cancelled`).

### Acciones planificadas
1. Implementar la capa modular de TypeScript (`src/types/`, `src/lib/tauri.ts`, `src/lib/formats.ts`, `src/lib/errors.ts`).
2. Implementar `src/styles/tokens.css` eliminando el índigo por completo y aplicando el sistema lime `#c8f96b` + Geist / Geist Mono.
3. Desacoplar la UI en componentes accesibles con estados de conversión con discriminated unions.
4. Añadir scripts de `typecheck` y `lint` en `package.json`.
5. Ejecutar `pnpm typecheck`, `pnpm lint`, `cargo fmt`, `cargo clippy` y `cargo test` registrando resultados.


---

## 2026-11-16 — Ejecución y evidencia de verificación

### Acciones completadas
1. **Layout y contratos (`AGENTS.md`):**
   - Actualizada la sección 3 de `AGENTS.md` para reflejar la estructura real del repositorio (`src-tauri/src/convert/` y módulos frontend).
   - Creados `TODO.md` y `DEV_LOG.md` cumpliendo el protocolo de memoria.
2. **Capa modular de Frontend (`src/`):**
   - `src/types/index.ts`: Definidos `FileInfo`, `EngineInfo`, `AudioParams`, `ConvertOptions` y la unión discriminada de estados de conversión (`idle | validating | selected | converting | done | error | cancelled`).
   - `src/lib/tauri.ts`: Centralizado todo el acceso a IPC de Tauri (`inspect`, `convert`, `engine_info`, selectores nativos de archivo/directorio y detector de drop). Eliminado acceso directo de `@tauri-apps/api/core` en componentes.
   - `src/lib/formats.ts`: Catálogo de formatos, badges y accesos populares, helpers de formato de bytes y duración.
   - `src/lib/errors.ts`: Mapeo de errores normalizados con códigos estables (`FILE_NOT_FOUND`, `EMPTY_FILE`, `FILE_TOO_LARGE`, `TARGET_NOT_AVAILABLE`, `CORRUPT_FILE`, `PERMISSION_DENIED`, etc.) y sugerencias al usuario.
3. **Diseño y tokens (`tokens.css`):**
   - Creado `src/styles/tokens.css` implementando la paleta estricta de `AGENTS.md`:
     - Fondo oscuro: `#08090a` (near-black) con acento lime `#c8f96b`.
     - Fondo claro: `#f7f7f5` (off-white) con acento lime accesible `#3b6605` (contraste AA >= 4.5:1).
     - Tipografía: Geist para interfaz y Geist Mono para formatos/rutas/extensiones.
     - Eliminadas todas las variables o clases que contenían el tono índigo (`#6366f1` / `#4f46e5`).
     - Foco visible global accesible (`:focus-visible`).
     - Soporte para `prefers-reduced-motion`.
     - Cero estilos en línea (`style={{ ... }}`).
4. **Componentes y UI accesible:**
   - Creados `Icon.tsx`, `FormatBadge.tsx`, `DropZone.tsx`, `FormatSelector.tsx`, `AudioSettings.tsx` y `ConversionCard.tsx`.
   - Accesibilidad: botones con `aria-label`, grupos de chips con `role="group"` y `aria-pressed`, spinners con etiqueta legible y contexto textual de progreso.
   - Limpieza: borrado `src/lib/files.ts` obsoleto.
5. **Calidad de Rust (`clippy` y `fmt`):**
   - Subsanadas advertencias de clippy (`collapsible_match` en `office.rs`, `needless_borrow` en `markup.rs` y `too_many_arguments` en `sheets/mod.rs`).

### Evidencia de comandos de verificación
- `pnpm typecheck`: Exit code 0 (0 errores de TypeScript en modo estricto).
- `pnpm lint`: Exit code 0 (0 errores de validación).
- `pnpm build`: Exit code 0 (`vite v8.3.7 building client environment for production... built in 3.19s`).
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: Exit code 0 (formato perfecto).
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`: Exit code 0 (0 advertencias, 0 errores).
- `cargo test --manifest-path src-tauri/Cargo.toml`: Exit code 0 (**50 de 50 pruebas unitarias pasadas**).

---

## 2026-11-19 — Reestructuración Senior: Cero archivos > 500 líneas y modularización total

### Contexto
El usuario solicitó reestructurar el proyecto como un ingeniero senior con máxima experiencia para que **ningún archivo de código exceda las 500 líneas de código**, sin romper ninguna funcionalidad ni pruebas, dejando revisar los cambios antes de empaquetar los instaladores `.exe`.

### Módulos refactorizados

1. **Frontend (`src/`):**
   - `src/App.css` (1641 líneas) descompuesto en módulos bajo `src/styles/`:
     - `fonts.css` (39 líneas): fuentes Geist y Geist Mono.
     - `base.css` (147 líneas): reset, variables, scrollbars.
     - `layout.css` (290 líneas): grid shell, sidebar, topbar, workspace.
     - `views.css` (150 líneas): view headers, panelería, badges de estado.
     - `converter.css` (290 líneas): DropZone, selector de formato, progress, states.
     - `badges.css` (120 líneas): colores de formato por categoría y tema.
     - `catalog.css` (280 líneas): tabla de catálogo, búsqueda y accesos rápidos.
     - `info-views.css` (130 líneas): cuadrícula de privacidad y acordeón FAQ.
     - `src/App.css` reducido a 14 líneas de imports declarativos.
   - `src/App.tsx` (492 líneas) descompuesto en:
     - `src/components/layout/Sidebar.tsx` (80 líneas)
     - `src/components/layout/Topbar.tsx` (55 líneas)
     - `src/features/converter/ConverterView.tsx` (130 líneas)
     - `src/features/privacy/PrivacyView.tsx` (65 líneas)
     - `src/features/faq/FaqView.tsx` (50 líneas)
     - `src/App.tsx` reducido a 240 líneas limpias de ruteo de vistas.

2. **Backend Rust (`src-tauri/src/`):**
   - `convert/mod.rs` (703 líneas):
     - Pruebas extraídas a `convert/tests.rs` (283 líneas).
     - `convert/mod.rs` reducido a 439 líneas.
   - `convert/catalog.rs` (679 líneas) modularizado en `convert/catalog/`:
     - `formats.rs` (380 líneas): catálogo maestro de 50 formatos.
     - `targets.rs` (115 líneas): resolución de rutas disponibles y multimedia.
     - `matrix.rs` (60 líneas): matrices de prueba por categoría.
     - `tests.rs` (70 líneas): pruebas de ruteo.
     - `mod.rs` (15 líneas): re-exports limpios.
   - `convert/documents/office.rs` (777 líneas) modularizado en `convert/documents/office/`:
     - `zip_utils.rs` (129 líneas): empaquetado ZIP y utilidades XML.
     - `docx.rs` (160 líneas): lector/escritor DOCX.
     - `odt.rs` (145 líneas): lector/escritor ODT.
     - `rtf.rs` (170 líneas): lector/escritor RTF.
     - `epub.rs` (105 líneas): lector/escritor EPUB.
     - `tests.rs` (50 líneas): pruebas de round-trip.
     - `mod.rs` (16 líneas): re-exports.
   - `convert/sheets/mod.rs` (700 líneas) modularizado en:
     - `sheets/biff8.rs` (215 líneas): escritor BIFF8 con wrapper OLE2 Compound Document.
     - `sheets/writers.rs` (105 líneas): generadores XLSX y ODS.
     - `sheets/readers.rs` (120 líneas): lectores Calamine y decodificadores tabulares.
     - `sheets/tests.rs` (55 líneas): pruebas de hojas de cálculo.
     - `sheets/mod.rs` reducido a 115 líneas.
   - `convert/sheets/structured.rs` (660 líneas) modularizado en `convert/sheets/structured/`:
     - `yaml.rs` (85 líneas): parser y emisor YAML.
     - `sql_parser.rs` (335 líneas): tokenizer SQL, lexer y extractor AST.
     - `sql.rs` (175 líneas): generador CREATE TABLE / INSERT y parse_sql.
     - `tests.rs` (55 líneas): pruebas unitarias YAML y SQL.
     - `mod.rs` (26 líneas): re-exports.
   - `convert/documents/pdf.rs` (528 líneas) modularizado en `convert/documents/pdf/`:
     - `render.rs` (52 líneas): rasterización de primera página con Ghostscript.
     - `writer.rs` (285 líneas): generador nativo PDF para texto, imágenes y tablas.
     - `tests.rs` (28 líneas): pruebas de WinAnsi y rasterizado.
     - `mod.rs` (13 líneas): re-exports.

### Auditoría de líneas de código
- **Total de archivos escaneados:** 72 archivos (.rs, .ts, .tsx, .css).
- **Archivos > 500 líneas:** **0 archivos**. (100% de los archivos están por debajo de 500 líneas).

### Evidencia de verificación
- `pnpm typecheck`: Exit code 0 (0 errores de TypeScript).
- `pnpm lint`: Exit code 0.
- `pnpm build`: Exit code 0 (Bundle Vite generado con éxito).
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: Exit code 0 (Formateo perfecto).
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`: Exit code 0 (0 advertencias, 0 errores).
- `cargo test --manifest-path src-tauri/Cargo.toml`: Exit code 0 (**74 de 74 pruebas unitarias pasadas en 7.09s**).


### Contexto

El producto web anunciaba rutas que el escritorio no tenía: PDF a DOCX/JPG/PNG/XLSX/CSV,
PPTX/PPT/ODP, YAML/SQL, ICO, vídeo, ZIP/RAR/7Z, DOC y PDF/A. Se auditó el catálogo web
(`formatRules.ts` y los conversores de `server/src/services/converters/`) y se trasladó al motor
Rust sólo lo que puede completarse en local.

### Decisiones

1. **PDF/A excluido.** El backend web declara `pdfa` pero no lo implementa. Generar PDF/A sin
   perfil ICC embebido ni `OutputIntent` sería una promesa falsa de fidelidad. No se declara.
2. **Disponibilidad real, no catálogo teórico.** `catalog::targets_for` define el contrato, pero
   `inspect` filtra cada destino con `route_available`. Si falta LibreOffice, Ghostscript, 7-Zip,
   WinRAR, FFmpeg o la voz de Windows, la ruta no aparece y `convert` la rechaza con un mensaje
   que indica qué instalar.
3. **Herramientas externas explícitas y cacheadas.** `external.rs` descubre `ffmpeg`, `ffprobe`,
   `soffice`, `7z`, `rar`, `unrar`, `gs` y `powershell` una sola vez (`OnceLock`) y admite overrides
   `FILEFLOW_<NOMBRE>`. No se descarga ni se ejecuta nada por red.
4. **Vídeo con códecs verificados.** MP4/MOV/MKV/FLV = H.264/AAC; WEBM = VP9/Opus; AVI =
   MPEG-4/MP3; WMV = WMV2/WMA. Cada destino se publica sólo si FFmpeg tiene el codificador.
5. **PPTX/ODP con Rust puro para texto; LibreOffice para render.** Extraer texto no exige
   LibreOffice; JPG/PNG, PPT binario y PPTX↔ODP sí. **PPTX→MP4 no se declara**: requeriría
   LibreOffice + Ghostscript + FFmpeg y aún no está implementado de extremo a extremo.
6. **PDF→XLSX/CSV heurístico.** Usa líneas, tabulaciones, comas y dos o más espacios, igual que el
   producto web, y falla de forma explícita si no encuentra filas. No aplica OCR.
7. **Archivos comprimidos.** Se extrae a un espacio temporal y se reempaqueta. ZIP es Rust puro;
   RAR/7Z usan los binarios detectados. Se rechazan rutas `..`, enlaces simbólicos, más de 2000
   entradas y expansiones superiores a 100 MB.
8. **TXT→MP3 local.** Usa `System.Speech` de Windows vía PowerShell y FFmpeg para MP3; sólo se
   ofrece en Windows con voz, PowerShell y `libmp3lame`.

### Implementación

- `catalog.rs`: 50 formatos y 476 rutas declaradas; eliminado `PDFA`.
- `support.rs`: firmas de DOC/XLS/PPT (OLE2), PPTX/ODP (ZIP), ICO, ZIP/RAR/7Z, vídeo, YAML y SQL.
- Nuevos módulos: `video.rs`, `archives.rs`, `presentations.rs`, `documents/tts.rs` y
  `sheets/structured.rs`.
- `documents`: DOC vía LibreOffice, PDF→JPG/PNG vía Ghostscript (`-r150`, primera página),
  PDF→XLSX/CSV vía filas extraídas.
- `sheets`: YAML con `yaml-rust2` y SQL con parser de `CREATE TABLE` / `INSERT INTO`.
- `images`: ICO con la caja `ico` (16–256 px) y GIF→MP4 delegado a FFmpeg.
- `mod.rs`: despacho exhaustivo por categoría, filtrado de disponibilidad y contador real.
- Frontend: 7 categorías, 50 formatos, badges, filtros, conversiones populares, `CategoryKey`
  ampliado y parámetros de audio al extraer audio de vídeo.

### Herramientas detectadas en el equipo de auditoría

- Presentes: FFmpeg/FFprobe (WinGet), PowerShell y WinRAR (`Rar.exe`, `UnRAR.exe`).
- Ausentes: LibreOffice, 7-Zip, Ghostscript, Poppler, pandoc y antiword.
- Resultado: **444 de 476 rutas disponibles**. Las 32 restantes corresponden a LibreOffice,
  Ghostscript y 7-Zip y quedan ocultas por diseño.

### Evidencia de verificación

- `pnpm typecheck`: 0 errores.
- `pnpm lint`: 0 errores.
- `pnpm build`: compilación Vite correcta.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: sin diferencias.
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`: 0 advertencias.
- `cargo test --manifest-path src-tauri/Cargo.toml`: **73 de 73 pruebas pasadas**.
  Incluye ZIP recomprimido, Zip Slip rechazado, ida y vuelta ZIP↔RAR con WinRAR, YAML/SQL, ICO,
  PDF→CSV, vídeo MP4→PNG/JPG/GIF/MP3/WEBM/MKV y todas las rutas de audio con FFmpeg.
- Prueba de contador: `rutas disponibles en este equipo: 444 de 476 declaradas`.

### Pendiente de verificación manual

- Rutas con LibreOffice (DOC, PPT, PPTX/ODP→JPG/PNG y PPTX↔ODP): implementadas, no ejecutables en
  este equipo.
- PDF→JPG/PNG con Ghostscript: implementado, no ejecutable en este equipo.
- 7Z: implementado, oculto sin 7-Zip.
- macOS y Linux: no verificados; el desarrollo y las pruebas se hicieron en Windows.

---

## 2026-11-18 — Matriz completa de conversiones en la interfaz

### Contexto

La sección «Conversiones» sólo mostraba una lista curada de accesos rápidos, no el catálogo real
que el motor puede ejecutar. El usuario pidió visualizar todos los tipos de conversión.

### Decisiones

1. **La matriz vive en Rust, no en TypeScript.** Nuevo comando `conversion_matrix` que devuelve las
   7 categorías, los 50 formatos, sus extensiones y todos los destinos declarados, cada uno anotado
   con `available` y la herramienta que falta (`requires`). El frontend no duplica ninguna tabla.
2. **Disponibilidad honesta.** Por defecto sólo se muestran las rutas ejecutables en el equipo. Un
   conmutador permite ver las restantes atenuadas y deshabilitadas, con la herramienta necesaria
   (`LibreOffice`, `Ghostscript`, `FFmpeg`, `7-Zip`, `WinRAR`…). No se puede pulsar una ruta que
   vaya a fallar.
3. **Un clic prepara la conversión.** Pulsar un destino del catálogo (o un acceso rápido) lanza el
   selector nativo filtrado por el formato de origen y preselecciona el destino mediante una
   petición con `nonce`, evitando que un re-render repita el diálogo.
4. **Búsqueda y filtros.** Filtro por categoría existente + búsqueda por formato/extensión. El
   contador muestra rutas visibles, disponibles y declaradas.

### Implementación

- Backend: `MatrixTarget`/`MatrixFormat`/`MatrixCategory`/`ConversionMatrix`, `conversion_matrix()`
  y `route_requirement()` en `src-tauri/src/convert/mod.rs`; comando y prueba en `src-tauri/src/lib.rs`.
- Frontend: `src/features/converter/ConversionCatalog.tsx`, `getConversionMatrix()` en
  `src/lib/tauri.ts`, tipos espejo en `src/types/index.ts`, icono `search`, estilos en `App.css`.
- `ConversionCard` acepta `routeRequest` y abre el selector una sola vez por petición.
- Se conectó el arrastre de archivos: `listenFileDrops` ahora entrega la ruta a `ConversionCard`
  mediante `externalLoadRequest` (props que existían sin uso) y se limpia tras cargarla.
- La sección `converters` pasa a «Todas las conversiones» y conserva los accesos rápidos arriba.

### Evidencia de verificación

- `pnpm typecheck`: 0 errores.
- `pnpm lint`: 0 errores.
- `pnpm build`: 36 módulos transformados; CSS 29.57 kB y JS 276.65 kB.
- `cargo fmt -- --check`: sin diferencias.
- `cargo clippy -- -D warnings`: 0 advertencias.
- `cargo test`: **74 de 74 pruebas pasadas**, incluida `conversion_matrix_matches_the_catalog`
  (7 categorías, 50 formatos, 476 rutas declaradas y contador disponible coherente con
  `available_conversion_count`).

### Pendiente de verificación manual

- Confirmar en la ventana Tauri el diálogo nativo al pulsar un destino y el filtro por formato.
- Revisar ambos temas y el desbordamiento horizontal con la matriz completa visible.

---

## 2026-11-18 — Rediseño completo: Slate Cool / Slate White, Hyper Cyan y Experiencia Desktop Nativa

### Contexto

El usuario solicitó un rediseño completo de la interfaz visual con:
1. Una estética de software de escritorio moderno (menos formato de landing page de marketing web con scroll vertical infinito, más espacio de trabajo por vistas).
2. Paleta estricta dual-theme: **Slate Cool** (`#0d1117` / `#161b22`) en modo oscuro y **Slate White** (`#f0f2f5` / `#ffffff`) en modo claro.
3. Acento cromático principal: **Hyper Cyan** (`#22d3ee` con glow en oscuro, `#0284c7` en claro cumpliendo WCAG AA).
4. Badges de formato con colores asociados al estándar o aplicación real de cada archivo (PDF rojo Acrobat, DOCX azul Word, XLSX verde Excel, ZIP amarillo, MP3 verde, etc.) en lugar de una categoría monótona.

### Decisiones e Implementación

1. **Tokens (`src/styles/tokens.css`):**
   - Actualizado el sistema de tokens a Slate Cool y Slate White.
   - Definidos tokens de acento Hyper Cyan (`--color-accent`, `--color-accent-hover`, `--color-accent-soft`, `--color-accent-border`, `--color-accent-glow`, `--color-accent-foreground`).
   - Radios y sombras ajustados a una app moderna (pill buttons, tarjetas limpias, elevaciones de superficie).
2. **Badges de formato (`src/lib/formats.ts` y `src/App.css`):**
   - Actualizado `FORMAT_BADGE_TONE` para asignar identidades visuales únicas a los 50 formatos.
   - Estilizados badges con contraste AA específico para cada formato tanto en modo oscuro como en modo claro.
3. **Estructura Desktop por Vistas (`src/App.tsx`):**
   - Eliminado el scroll de página web única con secciones de marketing repetitivas y pie de página web largo.
   - La barra lateral conmuta fluidamente entre 4 espacios dedicados:
     - **Conversor**: el área de trabajo primordial con tarjeta de conversión moderna, DropZone XXL, y accesos rápidos populares debajo.
     - **Catálogo**: herramienta completa de búsqueda y exploración de todas las combinaciones y dependencias del motor local.
     - **Privacidad y Motor**: panel informativo de seguridad offline, cero telemetría y especificaciones técnicas de Rust.
     - **Ayuda y FAQ**: preguntas frecuentes organizadas en acordeón limpio.
   - Cualquier clic en una ruta del catálogo o accesos rápidos conmuta automáticamente a la vista Conversor con la ruta preparada.
4. **DropZone y ConversionCard (`src/features/converter/`):**
   - DropZone ampliada con estilo moderno, bordes cyan dashed, icono central y botón píldora.
   - ConversionCard con botones tipo píldora, chips de formato destino con estados activos en cyan, y barra de progreso animada.

### Evidencia de verificación

- `pnpm typecheck`: 0 errores.
- `pnpm lint`: 0 errores.
- `pnpm build`: 36 módulos empaquetados; bundle optimizado (CSS 34.52 kB, JS 273.28 kB).
- `cargo fmt -- --check`: 0 diferencias.
- `cargo clippy -- -D warnings`: 0 advertencias.
- `cargo test`: 74 de 74 pruebas pasadas exitosamente en 8.40s.

---

## 2026-11-18 — Corrección de responsividad y barra de filtros de categorías

### Contexto
El usuario reportó un problema de responsividad y errores visuales en la barra de categorías de formatos:
- Al abrir la app en su tamaño de ventana predeterminado (1080x720), la barra de filtros se deformaba al partirse en 2 líneas debido a `border-radius: var(--radius-full)` (9999px) en el contenedor compartido.
- Al estar apretada junto al título en `.view-header`, la opción «Comprimidos» quedaba pegada al borde o cortada.
- Además, la categoría «Comprimidos» no tenía pares predeterminados en `POPULAR_CONVERSIONS`.

### Solución aplicada
1. **Separación de layout:** Se extrajo la barra de filtros de `.view-header`, dándole su propio contenedor de ancho completo (`.filters-bar`), evitando que compita por espacio con el título y descripción.
2. **Eliminación del contenedor deformado:** Se reemplazó el contenedor con forma de píldora gigante de 9999px por un conjunto de botones individuales (`.filter`), cada uno con su propio borde y relleno redondeado. Si la pantalla es reducida, cada píldora se acomoda naturalmente como chip sin deformar ningún borde.
3. **Tipografía y contraste:** Se sustituyó la fuente mono diminuta en mayúsculas por `var(--font-ui)` (`Geist`) a 11.5px con contraste AA y estado activo resaltado en Hyper Cyan (`--accent-soft` + `--accent-glow`).
4. **Soporte de comprimidos en accesos rápidos:** Se añadieron las rutas `ZIP → 7Z`, `7Z → ZIP` y `RAR → ZIP` a `POPULAR_CONVERSIONS` para que al filtrar por «Comprimidos» se muestren opciones reales.
5. **Responsividad global:** `.page` ampliado a `max-width: 1240px` (aprovecha la pantalla al maximizar), `.pairs` actualizado a `repeat(auto-fill, minmax(210px, 1fr))` y `.privacy-grid` a `repeat(auto-fit, minmax(240px, 1fr))`.

### Evidencia de verificación
- `pnpm typecheck`: 0 errores.
- `pnpm lint`: 0 errores.
- `pnpm build`: 36 módulos empaquetados en 3.81s (CSS 35.81 kB, JS 274.00 kB).

---

## 2026-11-18 — Mejoras visuales de escala y densidad de contenido

### Contexto
El usuario reportó que la UI se sentía pequeña, con letras diminutas y el espacio en blanco excesivo, especialmente visible al maximizar la ventana.

### Cambios aplicados en `src/App.css`
1. **Tipografía base (`body`):** de 13.5px → 14.5px para mejor legibilidad en monitores grandes.
2. **Encabezados de vista (`h1`):** de 24px → 30px con `letter-spacing: -0.03em` para más impacto visual.
3. **Párrafo de subtítulo:** de 12.5px → 14px con `line-height: 1.6` y `max-width: 560px`.
4. **Stat chips:** de 10px → 11.5px; los valores accent de strong sin tamaño propio → 12.5px.
5. **DropZone:** `min-height` de 240px → 280px; `padding` de 32px → 44px; ícono de 48px → 60px con glow de acento.
6. **Título "Arrastra tu archivo" (`strong`):** de 14px → 17px.
7. **Descripción del dropzone:** de 11.5px → 13px.
8. **Botón "Explorar archivos":** de 12px/8px→18px a 13px/10px→22px.
9. **Tarjetas `.pair`:** padding de 9px→12px a 13px→16px; `border-radius` de 10px → 12px; gap de 8px → 10px; hover con `box-shadow` de glow de acento.
10. **Format badges:** de 9px → 10.5px, padding de 3px→7px a 4px→8px.
11. **Botones `.primary` y `.ghost`:** de 12px/9px→18px a 13px/10px→20px.
12. **Botón `.primary.wide`:** de 13px a 14px.
13. **FAQ botones:** de 13px → 14px; respuestas de 12px → 13px.
14. **Privacy features:** título de 13px → 14.5px; descripción de 11.5px → 13px; padding de 16px → 20px; gap de 8px → 10px; ícono de 34px → 40px.
15. **Chips de formato destino:** de 10.5px → 11.5px.
16. **Contenedor `.panel`:** padding de 20px → 24px; título h2 de 16px → 18px.
17. **`.view-container`:** gap de 20px → 24px.
18. **`.page`:** padding de 24px→28px y 28px→32px laterales.
19. **DropZone.tsx:** ícono de Upload size 22 → 28.

### Evidencia de verificación
- `pnpm typecheck`: 0 errores.
- `pnpm lint`: 0 errores.
- `pnpm build`: 36 módulos, CSS 35.94 kB, JS 274.00 kB — exitoso en 3.71s.

---

## 2026-11-18 — Rediseño del Sidebar y lenguaje accesible sin jerga técnica

### Contexto
El usuario solicitó mejorar la visualización y legibilidad de la barra lateral (sidebar), aumentando los tamaños de letra e iconos, y reemplazando los mensajes técnicos dirigidos a programadores («Motor local Rust activo») por expresiones claras y directas orientadas al usuario final.

### Cambios aplicados
1. **Dimensiones y escala de la barra lateral (`src/App.css`):**
   - Ancho del sidebar: de 224px a **240px** para brindar más aire y presencia.
   - Ícono de marca / logo: de 32px a **38px** (ícono interno `size={19}`), texto de marca de 14px a **16px** y subtítulo «DESKTOP» de 9px a **10px**.
   - Botón «Nuevo archivo»: de 12px a **13.5px** con padding `10px 16px` e ícono `size={16}`.
   - Etiqueta «ESPACIOS»: de 8.5px a **10px** con tipografía mono nítida.
   - Botones de navegación (`.nav-item`): de 12px a **13.5px**, padding `10px 14px` e íconos `size={17}`.
2. **Textos accesibles orientados a personas reales (`src/App.tsx`):**
   - Eliminado: «Motor local Rust activo» y «100% Offline · Cero nube».
   - Reemplazado por:
     - Estado normal: **«Listo para convertir»** con subtítulo **«100% privado en tu equipo»**.
     - Estado de error: **«No disponible»**.
   - Pastilla superior (topbar): de «Local · Seguro» a **«Seguro y privado»**.
3. **Compilación de producción:**
   - `pnpm tauri build --no-bundle` finalizado exitosamente en 6m 41s.
   - Actualizado `FileFlow.exe` (11.2 MB) en la raíz del proyecto.



### Contexto
El usuario solicitó generar el ejecutable `.exe` de Windows para poder ejecutar la aplicación FileFlow Desktop de forma nativa e independiente.

### Acciones completadas
1. Ejecutado `pnpm tauri build` en modo release con optimización LTO de Rust activada.
2. Generación exitosa de los 3 binarios nativos para Windows (x64):
   - **Ejecutable directo portátil (`.exe`):** `src-tauri/target/release/fileflow-desktop.exe` (11.2 MB). No requiere instalación previa; corre directamente en cualquier Windows 10/11 con WebView2.
   - **Instalador NSIS (`.exe`):** `src-tauri/target/release/bundle/nsis/FileFlow_0.1.0_x64-setup.exe` (3.72 MB). Asistente clásico de instalación de Windows con desinstalador y accesos directos.
   - **Instalador MSI de Windows:** `src-tauri/target/release/bundle/msi/FileFlow_0.1.0_x64_en-US.msi` (5.27 MB).
3. Verificada la integridad y existencia de los archivos generados en disco.


