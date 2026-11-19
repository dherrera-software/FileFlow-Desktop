//! Códecs y herramientas externas opcionales.
//!
//! FileFlow usa Rust puro para la mayoría de formatos. Las rutas que necesitan
//! un motor del sistema (FFmpeg, LibreOffice, 7-Zip/WinRAR, Ghostscript o la
//! voz de Windows) sólo se exponen en la UI cuando ese motor está disponible.
//! La detección se cachea: nunca se lanza un subproceso por cada destino.

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Workspace {
    pub dir: PathBuf,
}

impl Workspace {
    pub fn new() -> Result<Self, String> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let dir = std::env::temp_dir().join(format!("fileflow-{}-{unique}", std::process::id()));
        fs::create_dir_all(&dir)
            .map_err(|_| "No se pudo preparar el espacio temporal de conversión.".to_string())?;
        Ok(Workspace { dir })
    }

    pub fn write(&self, name: &str, bytes: &[u8]) -> Result<PathBuf, String> {
        let path = self.dir.join(name);
        fs::write(&path, bytes)
            .map_err(|_| "No se pudo preparar el archivo temporal.".to_string())?;
        Ok(path)
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

pub fn ffmpeg() -> Result<PathBuf, String> {
    cached("ffmpeg", locate_ffmpeg).ok_or_else(|| {
        "Este formato requiere el códec ffmpeg. Instala ffmpeg y asegúrate de que esté en el PATH, o define FILEFLOW_FFMPEG."
            .to_string()
    })
}

pub fn ffprobe() -> Option<PathBuf> {
    cached("ffprobe", locate_ffprobe)
}

/// LibreOffice en modo consola. Necesario para DOC/PPT y para renderizar
/// diapositivas a imagen.
pub fn soffice() -> Option<PathBuf> {
    cached("soffice", locate_soffice)
}

/// 7-Zip (7z/7za). Necesario para leer o crear contenedores 7z.
pub fn seven_zip() -> Option<PathBuf> {
    cached("7z", locate_seven_zip)
}

/// `Rar.exe` de WinRAR, necesario para crear contenedores RAR.
pub fn rar() -> Option<PathBuf> {
    cached("rar", locate_rar)
}

/// `UnRAR.exe` (o `Rar.exe`) para extraer contenedores RAR.
pub fn unrar() -> Option<PathBuf> {
    cached("unrar", locate_unrar)
}

/// Ghostscript para rasterizar la primera página de un PDF.
pub fn ghostscript() -> Option<PathBuf> {
    cached("gs", locate_ghostscript)
}

/// PowerShell permite usar la voz instalada de Windows para TXT -> MP3.
pub fn powershell() -> Option<PathBuf> {
    if !cfg!(windows) {
        return None;
    }
    cached("powershell", locate_powershell)
}

fn cached(name: &'static str, locate: fn() -> Option<PathBuf>) -> Option<PathBuf> {
    static CACHE: OnceLock<
        std::sync::Mutex<std::collections::HashMap<&'static str, Option<PathBuf>>>,
    > = OnceLock::new();
    let cache = CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    if let Ok(guard) = cache.lock()
        && let Some(found) = guard.get(name)
    {
        return found.clone();
    }
    let found = locate();
    if let Ok(mut guard) = cache.lock() {
        guard.insert(name, found.clone());
    }
    found
}

fn locate_ffmpeg() -> Option<PathBuf> {
    find_program("ffmpeg", &["-version"], &[])
}

fn locate_ffprobe() -> Option<PathBuf> {
    find_program("ffprobe", &["-version"], &[])
}

fn locate_soffice() -> Option<PathBuf> {
    find_program(
        "soffice",
        &["--version"],
        &[
            r"C:\Program Files\LibreOffice\program\soffice.exe",
            r"C:\Program Files (x86)\LibreOffice\program\soffice.exe",
            "/Applications/LibreOffice.app/Contents/MacOS/soffice",
            "/usr/bin/soffice",
            "/usr/bin/libreoffice",
            "/usr/local/bin/soffice",
            "/snap/bin/libreoffice",
        ],
    )
}

fn locate_seven_zip() -> Option<PathBuf> {
    find_program(
        "7z",
        &["i"],
        &[
            r"C:\Program Files\7-Zip\7z.exe",
            r"C:\Program Files (x86)\7-Zip\7z.exe",
            "/usr/bin/7z",
            "/usr/bin/7za",
            "/usr/local/bin/7z",
        ],
    )
}

fn locate_rar() -> Option<PathBuf> {
    find_program(
        "rar",
        &[],
        &[
            r"C:\Program Files\WinRAR\Rar.exe",
            r"C:\Program Files (x86)\WinRAR\Rar.exe",
            "/usr/bin/rar",
            "/usr/local/bin/rar",
        ],
    )
}

fn locate_unrar() -> Option<PathBuf> {
    find_program(
        "unrar",
        &[],
        &[
            r"C:\Program Files\WinRAR\UnRAR.exe",
            r"C:\Program Files (x86)\WinRAR\UnRAR.exe",
            r"C:\Program Files\WinRAR\Rar.exe",
            r"C:\Program Files (x86)\WinRAR\Rar.exe",
            "/usr/bin/unrar",
            "/usr/bin/rar",
            "/usr/local/bin/unrar",
        ],
    )
}

fn locate_ghostscript() -> Option<PathBuf> {
    let mut candidates = vec![
        r"C:\Program Files\gs".to_string(),
        r"C:\Program Files (x86)\gs".to_string(),
    ];
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        candidates.push(format!(r"{program_files}\gs"));
    }
    for root in candidates {
        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                for name in ["gswin64c.exe", "gswin32c.exe"] {
                    let candidate = entry.path().join("bin").join(name);
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }
    }
    find_program(
        "gswin64c",
        &["--version"],
        &[
            "/usr/bin/gs",
            "/usr/local/bin/gs",
            "/opt/homebrew/bin/gs",
            "/usr/local/bin/gswin64c",
        ],
    )
    .or_else(|| find_program("gs", &["--version"], &[]))
}

fn locate_powershell() -> Option<PathBuf> {
    if let Ok(value) = std::env::var("FILEFLOW_POWERSHELL") {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Some(path);
        }
    }
    for candidate in [
        r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe",
        r"C:\Windows\SysWOW64\WindowsPowerShell\v1.0\powershell.exe",
    ] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }
    find_program("powershell", &["-NoProfile", "-Command", "exit 0"], &[])
}

/// Busca un ejecutable en `FILEFLOW_<NAME>`, rutas conocidas y el `PATH`.
/// `probe` se usa sólo como verificación cuando el binario se resuelve por
/// nombre; `known` contiene rutas absolutas específicas del sistema.
fn find_program(name: &str, probe: &[&str], known: &[&str]) -> Option<PathBuf> {
    let env_name = format!("FILEFLOW_{}", name.to_ascii_uppercase());
    if let Ok(value) = std::env::var(&env_name) {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Some(path);
        }
    }
    for candidate in known {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }
    let executable = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|current| current.parent().map(Path::to_path_buf))
    {
        for candidate in [
            dir.join(&executable),
            dir.join("binaries").join(&executable),
        ] {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let mut command = Command::new(name);
    command.args(probe);
    hide_console(&mut command);
    match command.output() {
        Ok(output) if output.status.success() => Some(PathBuf::from(name)),
        _ => None,
    }
}

/// Ejecuta una herramienta y devuelve una salida legible si falla.
pub fn run_tool(program: &Path, args: &[&OsStr], cwd: Option<&Path>) -> Result<Output, String> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    hide_console(&mut command);
    let output = command.output().map_err(|_| {
        format!(
            "No se pudo iniciar {}.",
            program
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "la herramienta externa".to_string())
        )
    })?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(tool_failure(program, &output))
    }
}

fn tool_failure(program: &Path, output: &Output) -> String {
    let name = program
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "la herramienta externa".to_string());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = stderr
        .lines()
        .chain(stdout.lines())
        .rfind(|line| !line.trim().is_empty())
        .unwrap_or("sin detalles")
        .trim();
    format!("{name} no pudo completar la conversión: {detail}.")
}

pub fn run_ffmpeg(args: &[&OsStr]) -> Result<(), String> {
    let program = ffmpeg()?;
    run_tool(&program, args, None).map(|_| ())
}

/// Convierte una imagen usando ffmpeg. `extra` se inserta antes de la salida.
pub fn ffmpeg_image(
    bytes: &[u8],
    input_extension: &str,
    output_extension: &str,
    extra: &[&str],
) -> Result<Vec<u8>, String> {
    let workspace = Workspace::new()?;
    let input = workspace.write(&format!("entrada.{input_extension}"), bytes)?;
    let output = workspace.path(&format!("salida.{output_extension}"));
    let mut args: Vec<&OsStr> = vec![
        OsStr::new("-y"),
        OsStr::new("-hide_banner"),
        OsStr::new("-loglevel"),
        OsStr::new("error"),
        OsStr::new("-i"),
        input.as_os_str(),
        OsStr::new("-frames:v"),
        OsStr::new("1"),
    ];
    for value in extra {
        args.push(OsStr::new(value));
    }
    args.push(output.as_os_str());
    run_ffmpeg(&args)?;
    fs::read(&output).map_err(|_| "ffmpeg no generó la imagen de salida.".to_string())
}

/// Comprueba si el FFmpeg instalado incluye un codificador concreto.
/// La lista de codificadores se consulta una sola vez por proceso.
pub fn ffmpeg_has_encoder(encoder: &str) -> bool {
    static ENCODERS: OnceLock<HashSet<String>> = OnceLock::new();
    let encoders = ENCODERS.get_or_init(|| {
        let Ok(program) = ffmpeg() else {
            return HashSet::new();
        };
        let mut command = Command::new(program);
        command.args(["-hide_banner", "-loglevel", "error", "-encoders"]);
        hide_console(&mut command);
        match command.output() {
            Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| line.split_whitespace().nth(1))
                .map(str::to_string)
                .collect(),
            _ => HashSet::new(),
        }
    });
    encoders.contains(encoder)
}

pub fn ffprobe_dimensions(path: &Path) -> Option<(u32, u32)> {
    let program = ffprobe()?;
    let mut command = Command::new(program);
    command.args([
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=width,height",
        "-of",
        "csv=s=x:p=0",
    ]);
    command.arg(path);
    hide_console(&mut command);
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let (width, height) = text.trim().split_once('x')?;
    Some((width.parse().ok()?, height.parse().ok()?))
}

pub fn ffprobe_duration_seconds(path: &Path) -> Option<f64> {
    let program = ffprobe()?;
    let mut command = Command::new(program);
    command.args([
        "-v",
        "error",
        "-show_entries",
        "format=duration",
        "-of",
        "default=noprint_wrappers=1:nokey=1",
    ]);
    command.arg(path);
    hide_console(&mut command);
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

pub fn ffprobe_has_stream(path: &Path, selector: &str) -> Option<bool> {
    let program = ffprobe()?;
    let mut command = Command::new(program);
    command.args([
        "-v",
        "error",
        "-select_streams",
        selector,
        "-show_entries",
        "stream=index",
        "-of",
        "csv=p=0",
    ]);
    command.arg(path);
    hide_console(&mut command);
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

#[cfg(windows)]
fn hide_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_console(_command: &mut Command) {}
