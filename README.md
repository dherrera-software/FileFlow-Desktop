# FileFlow Desktop

Conversor de archivos de escritorio construido con **Tauri 2 + React + TypeScript**. Todo el procesamiento
ocurre en el dispositivo: el frontend React se comunica con un motor en Rust mediante IPC, sin servidores,
sin cuentas y sin conexión a internet.

## Motor local

El "backend" de la aplicación es el binario Rust que Tauri empaqueta dentro del ejecutable:

- `inspect(path)`: detecta el formato, tamaño, dimensiones de imagen y conversiones disponibles.
- `convert(path, target, outputPath)`: convierte el archivo y lo escribe en la ruta elegida.
- `engine_info()`: informa versión, plataforma, número de formatos y límite de tamaño del motor.

Los comandos se ejecutan en el mismo equipo donde corre la app (`src-tauri/src/convert/`), por lo que:

- no hay peticiones HTTP ni dependencia de un servidor remoto;
- los archivos no salen del dispositivo;
- la app funciona sin conexión.

La ventana de Tauri carga el frontend desde el bundle local (`dist/` en producción y
`http://localhost:1420` sólo durante `tauri dev`). El backend Rust sigue siendo local en ambos casos.

## Conversiones compatibles

| Origen            | Destinos        |
| ----------------- | --------------- |
| TXT               | PDF, HTML, MD   |
| HTML / HTM        | MD              |
| MD / MARKDOWN     | HTML            |
| JPG / JPEG / PNG  | PDF             |
| CSV               | JSON, XML       |
| TSV               | CSV             |
| JSON              | CSV             |
| XML               | CSV             |

Límite: 25 MB por archivo. Los archivos de texto deben estar en UTF-8.

## Requisitos

- Node.js 20+ y pnpm
- Rust (stable) y las dependencias de Tauri para Windows
- WebView2 (incluido en Windows 10/11)

## Comandos

```bash
pnpm install          # instalar dependencias del frontend
pnpm tauri dev        # app de escritorio con recarga en caliente
pnpm tauri build      # instalador/ejecutable de producción
pnpm build            # comprobar TypeScript y compilar el frontend
cargo test --manifest-path src-tauri/Cargo.toml   # pruebas del motor local
```

> `pnpm dev` abre únicamente la vista previa web. En ese modo no hay acceso al motor local ni a los
> diálogos del sistema; usa `pnpm tauri dev` para probar la aplicación completa.

## Estructura

```
src/                     Interfaz React (App.tsx, estilos y puente IPC en lib/files.ts)
src-tauri/src/lib.rs     Comandos Tauri (inspect, convert, engine_info)
src-tauri/src/convert/   Motor de conversión en Rust (texto, markup, tablas, PDF e imágenes)
src-tauri/capabilities/  Permisos de la ventana (diálogos, drag & drop, abrir carpeta)
```

## Privacidad

No se envía telemetría ni archivos a ningún servicio. El único acceso al disco es el que autoriza el
usuario a través de los diálogos nativos de apertura y guardado.
