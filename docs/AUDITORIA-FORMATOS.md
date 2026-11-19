# Auditoría y contrato de formatos

Documento de control del motor de conversión de **FileFlow Desktop**. Registra el catálogo
declarado, las rutas implementadas, las dependencias de herramientas locales y las limitaciones
conocidas.

- Última revisión: 18 de noviembre de 2026
- Motor: `src-tauri/src/convert/` (Rust)
- Fuente de verdad del catálogo: `src-tauri/src/convert/catalog.rs`
- Fuente de verdad de disponibilidad: `src-tauri/src/convert/mod.rs` (`route_available`)
- Categorías: documentos, imágenes, hojas de cálculo, presentaciones, audio, vídeo y comprimidos
- Límite por archivo: 25 MB

## 1. Metodología

1. Se auditó el catálogo del producto web (`server/src/utils/formatRules.ts` y los conversores de
   `server/src/services/converters/`) y se contrastó con el catálogo Rust.
2. Se añadieron únicamente formatos y rutas que el motor local puede completar de verdad. No se
   declaró ninguna combinación que dependa de una herramienta ausente sin ocultarla en la
   inspección.
3. Cada ruta se clasifica como:
   - **R**: ruta obligatoria del contrato, implementada y cubierta por pruebas.
   - **O**: ampliación implementada.
   - **E**: implementada pero requiere una herramienta externa instalada.
   - **—**: no soportada.
4. La disponibilidad se calcula en tiempo de inspección: `inspect` devuelve sólo los destinos que
   pueden completarse en el equipo actual. `convert` vuelve a comprobar la ruta antes de ejecutar.
5. `PDF/A` **no está declarado**. El backend web lo anuncia pero no lo implementa; producir PDF/A
   sin perfil ICC y `OutputIntent` sería una promesa de fidelidad falsa.

## 2. Dependencias locales

| Herramienta | Rutas que habilita | Detección |
| ----------- | ------------------ | --------- |
| FFmpeg + FFprobe | Audio, vídeo, HEIC/AVIF, AVIF de salida, GIF→MP4 | `PATH`, junto al ejecutable, `FILEFLOW_FFMPEG` |
| LibreOffice (`soffice`) | DOC, PPT, PPTX↔ODP, diapositivas→JPG/PNG | `PATH`, rutas habituales, `FILEFLOW_SOFFICE` |
| Ghostscript (`gs`) | PDF→JPG/PNG (primera página) | `PATH`, rutas habituales, `FILEFLOW_GS` |
| WinRAR (`Rar.exe`/`UnRAR.exe`) | Crear/extraer RAR | `PATH`, `%ProgramFiles%\WinRAR`, `FILEFLOW_RAR` |
| 7-Zip (`7z`) | Crear/extraer 7Z | `PATH`, `%ProgramFiles%\7-Zip`, `FILEFLOW_7Z` |
| PowerShell + voz SAPI | TXT→MP3 | Windows + `System.Speech` + FFmpeg con `libmp3lame` |

Si falta la herramienta, la ruta no se muestra y `convert` responde con un error explícito que
indica qué instalar. No hay descargas automáticas ni llamadas de red.

## 3. Catálogo declarado

Resultado: **50 formatos** de entrada y **476 rutas** declaradas. En el equipo de auditoría
(Windows, FFmpeg y WinRAR instalados; sin LibreOffice, 7-Zip ni Ghostscript) están disponibles
**444 rutas**. El contador que muestra la interfaz usa el valor real devuelto por `engine_info` y
`conversion_matrix`, que además publica cada ruta con su disponibilidad y la herramienta que falta.

| Formato | Categoría | Extensiones | MIME principal | Dirección | Motor |
| ------- | --------- | ----------- | -------------- | --------- | ----- |
| DOCX | Documentos | `docx` | `application/vnd.openxmlformats-officedocument.wordprocessingml.document` | Ambas | Rust |
| DOC | Documentos | `doc` | `application/msword` | Entrada | LibreOffice |
| ODT | Documentos | `odt` | `application/vnd.oasis.opendocument.text` | Ambas | Rust |
| RTF | Documentos | `rtf` | `application/rtf` | Ambas | Rust |
| TXT | Documentos | `txt` | `text/plain` | Ambas | Rust |
| MD | Documentos | `md`, `markdown` | `text/markdown` | Ambas | Rust |
| HTML | Documentos | `html`, `htm` | `text/html` | Ambas | Rust |
| PDF | Documentos | `pdf` | `application/pdf` | Ambas | Rust / Ghostscript |
| EPUB | Documentos | `epub` | `application/epub+zip` | Ambas | Rust |
| JPG | Imágenes | `jpg`, `jpeg` | `image/jpeg` | Ambas | Rust |
| PNG | Imágenes | `png` | `image/png` | Ambas | Rust |
| WEBP | Imágenes | `webp` | `image/webp` | Ambas | Rust |
| BMP | Imágenes | `bmp` | `image/bmp` | Ambas | Rust |
| GIF | Imágenes | `gif` | `image/gif` | Ambas | Rust / FFmpeg |
| TIFF | Imágenes | `tif`, `tiff` | `image/tiff` | Ambas | Rust |
| HEIC | Imágenes | `heic`, `heif` | `image/heic` | Entrada | FFmpeg |
| SVG | Imágenes | `svg` | `image/svg+xml` | Entrada | Rust (`resvg`) |
| AVIF | Imágenes | `avif` | `image/avif` | Ambas | Rust / FFmpeg |
| ICO | Imágenes | `ico` | `image/x-icon` | Ambas | Rust (`ico`) |
| XLSX | Hojas de cálculo | `xlsx` | `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` | Ambas | Rust |
| XLS | Hojas de cálculo | `xls` | `application/vnd.ms-excel` | Ambas | Rust |
| ODS | Hojas de cálculo | `ods` | `application/vnd.oasis.opendocument.spreadsheet` | Ambas | Rust |
| CSV | Hojas de cálculo | `csv` | `text/csv` | Ambas | Rust |
| TSV | Hojas de cálculo | `tsv` | `text/tab-separated-values` | Ambas | Rust |
| JSON | Hojas de cálculo | `json` | `application/json` | Ambas | Rust |
| XML | Hojas de cálculo | `xml` | `application/xml` | Ambas | Rust |
| YAML | Hojas de cálculo | `yaml`, `yml` | `application/x-yaml` | Ambas | Rust |
| SQL | Hojas de cálculo | `sql` | `application/sql` | Ambas | Rust |
| PPTX | Presentaciones | `pptx` | `application/vnd.openxmlformats-officedocument.presentationml.presentation` | Ambas | Rust / LibreOffice |
| PPT | Presentaciones | `ppt` | `application/vnd.ms-powerpoint` | Entrada | LibreOffice |
| ODP | Presentaciones | `odp` | `application/vnd.oasis.opendocument.presentation` | Ambas | Rust / LibreOffice |
| MP3 | Audio | `mp3` | `audio/mpeg` | Ambas | FFmpeg |
| WAV | Audio | `wav` | `audio/wav` | Ambas | FFmpeg |
| FLAC | Audio | `flac` | `audio/flac` | Ambas | FFmpeg |
| AAC | Audio | `aac` | `audio/aac` | Ambas | FFmpeg |
| OGG | Audio | `ogg`, `oga` | `audio/ogg` | Ambas | FFmpeg |
| M4A | Audio | `m4a` | `audio/mp4` | Ambas | FFmpeg |
| WMA | Audio | `wma` | `audio/x-ms-wma` | Ambas | FFmpeg |
| OPUS | Audio | `opus` | `audio/opus` | Ambas | FFmpeg |
| AIFF | Audio | `aiff`, `aif` | `audio/aiff` | Ambas | FFmpeg |
| MP4 | Vídeo | `mp4` | `video/mp4` | Ambas | FFmpeg |
| MOV | Vídeo | `mov` | `video/quicktime` | Ambas | FFmpeg |
| AVI | Vídeo | `avi` | `video/x-msvideo` | Ambas | FFmpeg |
| MKV | Vídeo | `mkv` | `video/x-matroska` | Ambas | FFmpeg |
| WEBM | Vídeo | `webm` | `video/webm` | Ambas | FFmpeg |
| WMV | Vídeo | `wmv` | `video/x-ms-wmv` | Ambas | FFmpeg |
| FLV | Vídeo | `flv` | `video/x-flv` | Ambas | FFmpeg |
| ZIP | Comprimidos | `zip` | `application/zip` | Ambas | Rust |
| RAR | Comprimidos | `rar` | `application/vnd.rar` | Ambas | WinRAR |
| 7Z | Comprimidos | `7z` | `application/x-7z-compressed` | Ambas | 7-Zip |

`TXT` declara además la ruta **E** TXT→MP3 (narración local). `ZIP` como destino significa
«extraer y volver a comprimir» siguiendo la semántica del producto web.

## 4. Matrices de conversión

### 4.1 Documentos

| Origen \ Destino | DOCX | ODT | RTF | TXT | MD | HTML | PDF | EPUB | XLSX | CSV | JPG | PNG | MP3 |
| ---------------- | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: |
| DOC  | R | O | O | O | O | O | E | O | — | — | — | — | — |
| DOCX | — | R | R | R | R | R | R | O | — | — | — | — | — |
| ODT  | R | — | O | O | O | O | R | O | — | — | — | — | — |
| RTF  | R | O | — | O | O | O | R | O | — | — | — | — | — |
| TXT  | R | O | O | — | O | O | R | O | — | — | — | — | E |
| MD   | R | O | O | O | — | R | R | O | — | — | — | — | — |
| HTML | O | O | O | O | R | — | R | O | — | — | — | — | — |
| PDF  | R | O | O | O | O | O | — | O | R | R | E | E | — |
| EPUB | — | R | O | O | O | O | R | — | — | — | — | — | — |

DOC, PPT y las rutas `E` de PDF sólo aparecen si LibreOffice/Ghostscript están instalados.
PDF→XLSX/CSV usa una heurística de líneas y tabulaciones; no reconstruye tablas gráficas.

### 4.2 Imágenes

| Origen \ Destino | JPG | PNG | WEBP | BMP | GIF | TIFF | AVIF | ICO | PDF | MP4 |
| ---------------- | :-: | :-: | :--: | :-: | :-: | :--: | :--: | :-: | :-: | :-: |
| JPG  | — | R | R | R | R | R | E | R | R | — |
| PNG  | R | — | R | R | R | R | E | R | R | — |
| WEBP | R | R | — | R | R | R | E | R | R | — |
| BMP  | R | R | R | — | R | R | E | R | R | — |
| GIF  | R | R | R | R | — | R | E | R | R | E |
| TIFF | R | R | R | R | R | — | E | R | R | — |
| HEIC | E | E | E | E | E | E | E | E | E | — |
| SVG  | O | R | O | — | — | — | — | R | R | — |
| AVIF | E | E | E | E | E | E | — | E | E | — |
| ICO  | R | R | R | R | R | R | E | — | R | — |

HEIC/AVIF y la salida AVIF usan FFmpeg. La animación de GIF y WEBP se reduce a un fotograma.
GIF→MP4 también usa FFmpeg.

### 4.3 Hojas de cálculo y datos

| Origen \ Destino | XLSX | XLS | ODS | CSV | TSV | JSON | XML | YAML | SQL | PDF |
| ---------------- | :--: | :-: | :-: | :-: | :-: | :--: | :-: | :--: | :-: | :-: |
| XLSX | — | R | R | R | R | R | O | R | R | R |
| XLS  | R | — | R | R | R | O | O | R | R | R |
| ODS  | R | R | — | R | R | O | O | R | R | R |
| CSV  | R | O | O | — | R | R | O | R | R | R |
| TSV  | R | O | O | R | — | R | O | R | R | R |
| JSON | R | O | O | R | R | — | O | R | R | R |
| XML  | O | O | O | O | O | O | — | R | R | R |
| YAML | R | O | O | R | R | R | O | — | R | R |
| SQL  | R | O | O | R | R | R | O | R | — | R |

CSV, TSV, JSON, XML, YAML y SQL son representaciones planas de una sola hoja. SQL cubre
`CREATE TABLE` e `INSERT INTO … VALUES`; procedimientos y dialectos de proveedor quedan fuera.

### 4.4 Presentaciones

| Origen \ Destino | PPTX | ODP | PDF | DOCX | ODT | RTF | TXT | MD | HTML | EPUB | JPG | PNG |
| ---------------- | :--: | :-: | :-: | :--: | :-: | :-: | :-: | :-: | :--: | :--: | :-: | :-: |
| PPTX | — | E | R | R | R | R | R | R | R | R | E | E |
| PPT  | E | E | E | E | E | E | E | E | E | E | E | E |
| ODP  | E | — | R | R | R | R | R | R | R | R | E | E |

PPTX y ODP se leen con Rust puro (texto por diapositiva). Todo lo que requiere renderizar
diapositivas, el formato binario PPT o cambiar de contenedor usa LibreOffice. PPT se convierte
primero a PPTX con LibreOffice y después se aplica el mismo motor de texto, porque los filtros de
Writer (DOCX, ODT, EPUB…) no son fiables desde Impress.
**PPTX→MP4 no está declarado**: exigiría LibreOffice + Ghostscript + FFmpeg y aún no está
implementado de extremo a extremo.

### 4.5 Audio y vídeo

Audio: todos los formatos de audio convierten entre sí mediante FFmpeg. La UI expone bitrate,
frecuencia y canales; los formatos sin pérdida ignoran el bitrate.

Vídeo: todos los formatos de vídeo comparten la misma matriz de destinos.

| Destino | Códec de vídeo | Códec de audio | Motor |
| ------- | -------------- | -------------- | ----- |
| MP4, MOV, MKV, FLV | `libx264` | `aac` | FFmpeg |
| WEBM | `libvpx-vp9` | `libopus` | FFmpeg |
| AVI | `mpeg4` | `libmp3lame` | FFmpeg |
| WMV | `wmv2` | `wmav2` | FFmpeg |
| GIF | Paleta de 256 colores | — | FFmpeg |
| JPG / PNG | Primer fotograma | — | FFmpeg |
| Cualquier formato de audio | — | Según destino | FFmpeg |

Cada destino se publica sólo si FFmpeg tiene el codificador correspondiente. La duración y las
dimensiones se leen con FFprobe durante la inspección.

### 4.6 Comprimidos

| Origen \ Destino | ZIP | RAR | 7Z |
| ---------------- | :-: | :-: | :-: |
| ZIP | R (recomprimir) | E | E |
| RAR | E | E | E |
| 7Z  | E | E | E |

La conversión extrae a una carpeta temporal y vuelve a comprimir, igual que el producto web. Se
rechazan rutas con `..`, enlaces simbólicos, más de 2000 entradas y una expansión superior a
100 MB.

## 5. Validación de archivos

Cada archivo pasa por estas comprobaciones antes de convertirse:

1. **Extensión**: debe pertenecer al catálogo (`catalog::detect`), sin distinguir mayúsculas.
2. **Tamaño**: mayor que cero y menor o igual a 25 MB.
3. **Firma o estructura**:
   - imágenes raster: cabeceras JPEG, PNG, WEBP, BMP, GIF, TIFF, ICO y HEIC/AVIF (caja `ftyp`);
   - SVG: contiene `<svg>` o una declaración XML;
   - RTF: cabecera `{\rtf`;
   - TXT, MD, HTML, CSV, TSV, JSON, XML, YAML y SQL: se validan al analizarlos;
   - PDF: cabecera `%PDF` y texto extraíble;
   - DOCX, ODT, EPUB, XLSX, XLSM, XLSB, ODS, PPTX y ODP: archivo ZIP con las partes obligatorias;
   - DOC, XLS y PPT: contenedor OLE2/CFB válido;
   - ZIP, RAR y 7Z: firma del contenedor;
   - vídeo: caja ISO-BMFF, RIFF/AVI, Matroska/WEBM, ASF/WMV o FLV;
   - HEIC/AVIF: caja ISO-BMFF con marca compatible.
4. **Contenido**: un ZIP corrupto, un JSON/YAML/XML mal formado, un PDF sin texto, un archivo de
   audio sin duración o una presentación sin texto se rechazan con un mensaje claro.
5. **Destino**: la ruta de salida debe ser distinta de la de entrada.

Los archivos renombrados (por ejemplo, un GIF llamado `foto.png`) se rechazan en la validación de
firma.

## 6. Limitaciones conocidas

### Documentos

- PDF de entrada: extracción de texto y heurística de tabla. **No hay OCR**, no se conservan
  columnas, imágenes ni diseño, y un PDF escaneado puede quedar vacío.
- PDF→JPG/PNG rasteriza **sólo la primera página** con Ghostscript a 150 ppp.
- PDF de salida: documento nuevo con el texto interpretado. No es editable ni conserva el diseño.
- DOC requiere LibreOffice. DOCX, ODT, RTF, EPUB, MD y HTML usan un modelo de bloques común
  (títulos, párrafos, listas y tablas básicas); se pierden estilos avanzados, comentarios, cambios
  registrados, notas al pie, objetos incrustados y numeración compleja.
- EPUB se escribe con XHTML y CSS mínimos; no se valida con `epubcheck`.
- RTF implementa un subconjunto: texto, párrafos, negrita/cursiva, títulos y escapes Unicode.
- La codificación de texto se detecta por BOM, UTF-16 o Windows-1252.
- TXT→MP3 usa la voz instalada en Windows, no elige voces ni admite SSML; limita el texto a 20 000
  caracteres.

### Imágenes

- La salida HEIC y AVIF requiere FFmpeg con el codificador adecuado. Sin él, la ruta no se muestra.
- La entrada HEIC también requiere FFmpeg; no hay decodificador HEVC nativo.
- GIF y WEBP animados se convierten en un solo fotograma.
- SVG se rasteriza con `resvg`; fuentes externas, filtros avanzados o recursos remotos pueden no
  renderizarse.
- ICO se genera con tamaños 16, 32, 48, 64, 128 y 256 px; al leer se usa la imagen de mayor tamaño.
- No se conservan metadatos EXIF, ICC ni perfiles de color.
- La transparencia se aplana sobre fondo blanco en JPEG, BMP y PDF.

### Hojas de cálculo y datos

- CSV, TSV, JSON, XML, YAML y SQL son formatos planos: al exportar desde XLSX/XLS/ODS **sólo se usa
  la primera hoja**.
- Las fórmulas se exportan con su valor en caché, no como fórmula.
- JSON y XML aceptan arrays de objetos y arrays de arrays; estructuras muy anidadas se aplanan o se
  rechazan.
- YAML admite el primer documento del archivo. SQL sólo interpreta `CREATE TABLE` e
  `INSERT INTO … VALUES`; un volcado sin `INSERT` se rechaza.
- XLS se escribe en BIFF8; no se generan macros ni formatos condicionales.
- La detección de separador cubre coma, punto y coma y tabulador.

### Presentaciones

- PPTX/ODP→texto conserva el texto de cada diapositiva, no el orden visual, las notas del orador,
  las animaciones ni las imágenes.
- PPTX/ODP→PDF, DOCX, ODT, RTF, TXT, MD, HTML y EPUB genera un documento nuevo a partir de ese texto;
  **no conserva el diseño de las diapositivas**.
- PPTX→JPG/PNG y PPTX↔ODP requieren LibreOffice.
- PPT requiere LibreOffice para cualquier ruta.
- Una presentación sin texto editable se rechaza en lugar de producir un archivo vacío.

### Audio y vídeo

- Todas las rutas dependen de `ffmpeg`; FFprobe aporta duración y dimensiones.
- Los códecs disponibles dependen de la compilación instalada de FFmpeg.
- El vídeo se recodifica a 30 fps y dimensiones pares; se pierden subtítulos, capítulos, metadatos y
  pistas múltiples (sólo se conserva la primera de vídeo y la primera de audio).
- La extracción de audio usa los parámetros de la UI; la recodificación de vídeo usa 192 kbps.
- No se conservan carátulas, etiquetas ID3, capítulos ni metadatos.
- Un archivo sin pista de audio o con duración cero se rechaza.

### Comprimidos

- ZIP se crea con Rust puro. RAR y 7Z requieren WinRAR o 7-Zip; sin ellos, la ruta no se muestra.
- No se soportan contraseñas, volúmenes multidisco ni archivos cifrados.
- Los nombres con rutas absolutas o `..` se rechazan (protección Zip Slip).
- La expansión máxima es de 100 MB y 2000 entradas.

### Generales

- Límite de 25 MB por archivo, incluido vídeo.
- No hay conversión por lotes ni edición avanzada.
- No se realizan llamadas de red ni telemetría.

## 7. Pruebas

`cargo test --manifest-path src-tauri/Cargo.toml` cubre 74 pruebas:

- catálogo: 50 formatos, identificadores y extensiones únicos, destinos existentes y rutas
  obligatorias;
- documentos: lectura/escritura de DOCX, ODT, RTF, EPUB, Markdown/HTML y PDF;
- imágenes: rutas raster, SVG, ICO y PDF;
- hojas: XLSX, XLS, ODS, CSV, TSV, JSON, XML, YAML, SQL y PDF;
- presentaciones: extracción de texto PPTX/ODP y requisitos de LibreOffice;
- audio y vídeo: todas las rutas obligatorias cuando FFmpeg está disponible, incluido GIF→MP4;
- comprimidos: extracción/recompresión ZIP, ida y vuelta ZIP↔RAR con WinRAR y rechazo de Zip Slip;
- validación: archivos vacíos, desconocidos, renombrados, ZIP corruptos, JSON/YAML/SQL inválidos y
  firmas HEIC/AVIF/vídeo;
- capa de comandos: inspección, conversión, parámetros de audio, rechazo de destinos no disponibles
  y coherencia entre `conversion_matrix` y el catálogo (7 categorías, 50 formatos, 476 declaradas).

Las rutas que requieren LibreOffice, Ghostscript o 7-Zip se omiten en las pruebas de ejecución cuando
la herramienta no está instalada; en ese caso se comprueba que la ruta se reporte como no disponible.

## 8. Lista de aceptación

- [x] Catálogo web auditado y delta trasladado al motor local.
- [x] 50 formatos registrados con extensión, MIME, categoría y dirección.
- [x] 476 rutas declaradas; disponibilidad filtrada por herramienta instalada.
- [x] PDF→DOCX/JPG/PNG/XLSX/CSV implementado con motores explícitos.
- [x] PPTX/PPT/ODP y DOC implementados o bloqueados con mensaje claro.
- [x] YAML/SQL, ICO, vídeo, ZIP/RAR/7Z y TXT→MP3 integrados.
- [x] PDF/A excluido deliberadamente y documentado.
- [x] Validación de extensión, firma y archivos corruptos.
- [x] Límite de tamaño aplicado en inspección y conversión.
- [x] Parámetros de audio expuestos en la interfaz, también al extraer audio de vídeo.
- [x] La interfaz muestra las siete categorías y los destinos reales del equipo.
- [x] Catálogo completo en la UI vía `conversion_matrix`, con búsqueda, filtros y rutas bloqueadas marcadas con su herramienta.
- [x] `pnpm typecheck`, `pnpm lint`, `cargo fmt`, `cargo clippy -- -D warnings` y `cargo test`
      pasan en la fecha de revisión.
- [ ] Verificación manual en macOS/Linux (no realizada; el desarrollo actual es Windows).
- [ ] Verificación manual de rutas con LibreOffice, Ghostscript y 7-Zip en un equipo que los tenga.
