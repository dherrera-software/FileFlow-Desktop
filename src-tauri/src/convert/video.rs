//! Vídeo. Todas las rutas delegan en FFmpeg, que se detecta en tiempo de
//! inspección: si no está instalado, la UI no ofrece destinos de vídeo.
//!
//! Se usa H.264/AAC para MP4, MOV, MKV y FLV; VP9/Opus para WEBM; MPEG-4/MP3
//! para AVI y WMV2/WMA para WMV. Los fotogramas se extraen a JPG/PNG y el audio
//! se reutiliza del módulo `audio`.

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::convert::audio::{self, AudioOptions};
use crate::convert::external;

pub fn convert(
    bytes: &[u8],
    source: &str,
    target: &str,
    options: AudioOptions,
) -> Result<Vec<u8>, String> {
    match target {
        "JPG" | "PNG" => first_frame(bytes, source, target),
        "GIF" => to_gif(bytes, source),
        _ if audio::is_audio_target(target) => audio::convert(bytes, source, target, options),
        _ => transcode(bytes, source, target),
    }
}

pub fn dimensions(bytes: &[u8], source: &str) -> Option<(u32, u32)> {
    let workspace = external::Workspace::new().ok()?;
    let input = workspace.write(&format!("entrada.{source}"), bytes).ok()?;
    external::ffprobe_dimensions(&input)
}

pub fn duration_ms(path: &Path) -> Option<u64> {
    external::ffprobe_duration_seconds(path).map(|seconds| (seconds * 1000.0).round() as u64)
}

/// Indica si una ruta concreta puede completarse con los códecs instalados.
pub fn route_available(target: &str) -> bool {
    if external::ffmpeg().is_err() {
        return false;
    }
    if audio::is_audio_target(target) {
        return audio::target_available(target);
    }
    match codecs(target) {
        Some((video, audio)) => {
            external::ffmpeg_has_encoder(video) && audio.is_none_or(external::ffmpeg_has_encoder)
        }
        None => matches!(target, "GIF" | "JPG" | "PNG"),
    }
}

fn codecs(target: &str) -> Option<(&'static str, Option<&'static str>)> {
    match target {
        "MP4" | "MOV" | "MKV" | "FLV" => Some(("libx264", Some("aac"))),
        "WEBM" => Some(("libvpx-vp9", Some("libopus"))),
        "AVI" => Some(("mpeg4", Some("libmp3lame"))),
        "WMV" => Some(("wmv2", Some("wmav2"))),
        _ => None,
    }
}

fn transcode(bytes: &[u8], source: &str, target: &str) -> Result<Vec<u8>, String> {
    let (video_codec, audio_codec) = codecs(target)
        .ok_or_else(|| format!("La conversión de {source} a {target} no está disponible."))?;
    let workspace = external::Workspace::new()?;
    let input = workspace.write(&format!("entrada.{source}"), bytes)?;
    let output = workspace.path(&format!("salida.{}", extension(target)));
    if external::ffprobe_duration_seconds(&input).is_some_and(|duration| duration <= 0.05) {
        return Err("El vídeo tiene duración cero o está vacío.".to_string());
    }
    let has_audio = external::ffprobe_has_stream(&input, "a:0").unwrap_or(false);

    let scale = "scale=trunc(iw/2)*2:trunc(ih/2)*2,fps=30".to_string();
    let crf = if target == "WEBM" { "32" } else { "23" };
    let mut args: Vec<&OsStr> = vec![
        OsStr::new("-y"),
        OsStr::new("-hide_banner"),
        OsStr::new("-loglevel"),
        OsStr::new("error"),
        OsStr::new("-i"),
        input.as_os_str(),
        OsStr::new("-map"),
        OsStr::new("0:v:0"),
    ];
    if has_audio && audio_codec.is_some() {
        args.push(OsStr::new("-map"));
        args.push(OsStr::new("0:a:0?"));
    }
    args.extend([
        OsStr::new("-vf"),
        OsStr::new(&scale),
        OsStr::new("-c:v"),
        OsStr::new(video_codec),
    ]);
    if target == "WEBM" {
        args.extend([OsStr::new("-b:v"), OsStr::new("0")]);
    }
    args.extend([OsStr::new("-crf"), OsStr::new(crf)]);
    if target != "WEBM" {
        args.extend([OsStr::new("-preset"), OsStr::new("medium")]);
    }
    args.extend([OsStr::new("-pix_fmt"), OsStr::new("yuv420p")]);
    if has_audio {
        if let Some(codec) = audio_codec {
            args.extend([
                OsStr::new("-c:a"),
                OsStr::new(codec),
                OsStr::new("-b:a"),
                OsStr::new("192k"),
            ]);
        }
    } else {
        args.push(OsStr::new("-an"));
    }
    if matches!(target, "MP4" | "MOV") {
        args.extend([OsStr::new("-movflags"), OsStr::new("+faststart")]);
    }
    args.push(output.as_os_str());
    external::run_ffmpeg(&args)?;
    read_output(&output, "vídeo")
}

fn to_gif(bytes: &[u8], source: &str) -> Result<Vec<u8>, String> {
    let workspace = external::Workspace::new()?;
    let input = workspace.write(&format!("entrada.{source}"), bytes)?;
    let output = workspace.path("salida.gif");
    let filter =
        "fps=12,scale=480:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse";
    let args: Vec<&OsStr> = vec![
        OsStr::new("-y"),
        OsStr::new("-hide_banner"),
        OsStr::new("-loglevel"),
        OsStr::new("error"),
        OsStr::new("-i"),
        input.as_os_str(),
        OsStr::new("-an"),
        OsStr::new("-vf"),
        OsStr::new(filter),
        OsStr::new("-loop"),
        OsStr::new("0"),
        output.as_os_str(),
    ];
    external::run_ffmpeg(&args)?;
    read_output(&output, "GIF")
}

fn first_frame(bytes: &[u8], source: &str, target: &str) -> Result<Vec<u8>, String> {
    let workspace = external::Workspace::new()?;
    let input = workspace.write(&format!("entrada.{source}"), bytes)?;
    let output = workspace.path(&format!("salida.{}", extension(target)));
    let args: Vec<&OsStr> = vec![
        OsStr::new("-y"),
        OsStr::new("-hide_banner"),
        OsStr::new("-loglevel"),
        OsStr::new("error"),
        OsStr::new("-ss"),
        OsStr::new("0"),
        OsStr::new("-i"),
        input.as_os_str(),
        OsStr::new("-frames:v"),
        OsStr::new("1"),
        OsStr::new("-an"),
        OsStr::new("-q:v"),
        OsStr::new("2"),
        output.as_os_str(),
    ];
    external::run_ffmpeg(&args)?;
    read_output(&output, "fotograma")
}

fn read_output(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    let bytes =
        fs::read(path).map_err(|_| format!("ffmpeg no generó el archivo de {label} de salida."))?;
    if bytes.is_empty() {
        return Err(format!(
            "La conversión produjo un archivo de {label} vacío."
        ));
    }
    Ok(bytes)
}

fn extension(id: &str) -> &'static str {
    crate::convert::support::extension_for(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_routes_are_reported_as_available_only_with_codecs() {
        if external::ffmpeg().is_err() {
            assert!(!route_available("MP4"));
            return;
        }
        // H.264/AAC es la combinación base de la mayoría de instalaciones.
        assert!(route_available("MP4"));
    }
}
