//! Audio. La decodificación y codificación se delega en ffmpeg para cubrir
//! MP3, WAV, FLAC, AAC, OGG, M4A, WMA, Opus y AIFF con una sola ruta estable.
//! Se validan parámetros, duración y archivos corruptos.

use std::ffi::OsStr;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::convert::external;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioOptions {
    pub bitrate_kbps: u32,
    pub sample_rate_hz: u32,
    pub channels: u16,
}

impl Default for AudioOptions {
    fn default() -> Self {
        AudioOptions {
            bitrate_kbps: 192,
            sample_rate_hz: 44_100,
            channels: 2,
        }
    }
}

impl AudioOptions {
    pub fn validate(&self) -> Result<(), String> {
        if !(64..=320).contains(&self.bitrate_kbps) {
            return Err(
                "El bitrate debe estar entre 64 y 320 kbps. Ajusta el valor e inténtalo de nuevo."
                    .to_string(),
            );
        }
        if !(22_050..=96_000).contains(&self.sample_rate_hz) {
            return Err(
                "La frecuencia de muestreo debe estar entre 22.05 kHz y 96 kHz.".to_string(),
            );
        }
        if !matches!(self.channels, 1 | 2) {
            return Err("Los canales deben ser mono (1) o estéreo (2).".to_string());
        }
        Ok(())
    }
}

pub fn convert(
    bytes: &[u8],
    source: &str,
    target: &str,
    options: AudioOptions,
) -> Result<Vec<u8>, String> {
    options.validate()?;
    let workspace = external::Workspace::new()?;
    let input = workspace.write(&format!("entrada.{}", extension(source)), bytes)?;
    let output = workspace.path(&format!("salida.{}", extension(target)));

    if external::ffprobe_duration_seconds(&input).is_some_and(|duration| duration <= 0.05) {
        return Err(
            "El audio tiene duración cero o está vacío; no hay nada que convertir.".to_string(),
        );
    }

    let mut args: Vec<&OsStr> = vec![
        OsStr::new("-y"),
        OsStr::new("-hide_banner"),
        OsStr::new("-loglevel"),
        OsStr::new("error"),
        OsStr::new("-i"),
        input.as_os_str(),
        OsStr::new("-map"),
        OsStr::new("0:a:0"),
        OsStr::new("-vn"),
        OsStr::new("-ar"),
    ];
    let sample_rate = options.sample_rate_hz.to_string();
    let channels = options.channels.to_string();
    args.push(OsStr::new(&sample_rate));
    args.push(OsStr::new("-ac"));
    args.push(OsStr::new(&channels));

    let bitrate = format!("{}k", options.bitrate_kbps);
    let codec = codec_for(target)?;
    args.push(OsStr::new("-c:a"));
    args.push(OsStr::new(codec));
    if uses_bitrate(target) {
        args.push(OsStr::new("-b:a"));
        args.push(OsStr::new(&bitrate));
    }
    args.push(output.as_os_str());

    external::run_ffmpeg(&args)?;
    let converted = fs::read(&output)
        .map_err(|_| "ffmpeg no generó el archivo de audio de salida.".to_string())?;
    if converted.is_empty() {
        return Err("La conversión produjo un archivo de audio vacío.".to_string());
    }
    Ok(converted)
}

pub fn duration_ms(bytes: &[u8], source: &str) -> Option<u64> {
    let workspace = external::Workspace::new().ok()?;
    let input = workspace
        .write(&format!("entrada.{}", extension(source)), bytes)
        .ok()?;
    external::ffprobe_duration_seconds(&input).map(|seconds| (seconds * 1000.0).round() as u64)
}

pub fn is_audio_target(id: &str) -> bool {
    crate::convert::catalog::is_audio_target(id)
}

/// Comprueba que FFmpeg exista y que el codificador requerido esté compilado.
pub fn target_available(target: &str) -> bool {
    match codec_for(target) {
        Ok(codec) => external::ffmpeg_has_encoder(codec),
        Err(_) => false,
    }
}

fn extension(id: &str) -> &'static str {
    crate::convert::support::extension_for(id)
}

fn codec_for(target: &str) -> Result<&'static str, String> {
    Ok(match target {
        "MP3" => "libmp3lame",
        "WAV" => "pcm_s16le",
        "FLAC" => "flac",
        "AAC" => "aac",
        "OGG" => "libvorbis",
        "M4A" => "aac",
        "WMA" => "wmav2",
        "OPUS" => "libopus",
        "AIFF" => "pcm_s16be",
        _ => {
            return Err(format!(
                "La conversión de audio a {target} no está disponible."
            ));
        }
    })
}

fn uses_bitrate(target: &str) -> bool {
    matches!(target, "MP3" | "AAC" | "OGG" | "M4A" | "WMA" | "OPUS")
}

/// WAV PCM 16 bits mono de 0.25 s, generado sin dependencias externas.
#[cfg(test)]
pub(crate) fn sample_wav() -> Vec<u8> {
    use std::f32::consts::TAU;
    let sample_rate = 44_100u32;
    let samples = 11_025usize;
    let mut data = Vec::with_capacity(44 + samples * 2);
    let byte_rate = sample_rate * 2;
    data.extend(b"RIFF");
    data.extend(((36 + samples * 2) as u32).to_le_bytes());
    data.extend(b"WAVEfmt ");
    data.extend(16u32.to_le_bytes());
    data.extend(1u16.to_le_bytes());
    data.extend(1u16.to_le_bytes());
    data.extend(sample_rate.to_le_bytes());
    data.extend(byte_rate.to_le_bytes());
    data.extend(2u16.to_le_bytes());
    data.extend(16u16.to_le_bytes());
    data.extend(b"data");
    data.extend((samples as u32 * 2).to_le_bytes());
    for index in 0..samples {
        let value = (TAU * 440.0 * index as f32 / sample_rate as f32).sin();
        data.extend(((value * 12_000.0) as i16).to_le_bytes());
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ffmpeg_available() -> bool {
        external::ffmpeg().is_ok()
    }

    #[test]
    fn rejects_out_of_range_parameters() {
        let invalid = AudioOptions {
            bitrate_kbps: 32,
            sample_rate_hz: 44_100,
            channels: 2,
        };
        assert!(invalid.validate().unwrap_err().contains("bitrate"));
        let invalid = AudioOptions {
            bitrate_kbps: 192,
            sample_rate_hz: 8_000,
            channels: 2,
        };
        assert!(invalid.validate().unwrap_err().contains("frecuencia"));
        let invalid = AudioOptions {
            bitrate_kbps: 192,
            sample_rate_hz: 44_100,
            channels: 3,
        };
        assert!(invalid.validate().unwrap_err().contains("canales"));
    }

    #[test]
    fn wav_converts_to_required_targets_with_parameters() {
        if !ffmpeg_available() {
            eprintln!("ffmpeg no disponible: se omite la prueba de audio");
            return;
        }
        let wav = sample_wav();
        let options = AudioOptions {
            bitrate_kbps: 128,
            sample_rate_hz: 44_100,
            channels: 2,
        };
        for target in ["MP3", "FLAC", "AAC", "OGG", "M4A"] {
            let bytes = convert(&wav, "WAV", target, options).unwrap_or_else(|error| {
                panic!("falló WAV -> {target}: {error}");
            });
            assert!(!bytes.is_empty(), "salida vacía WAV -> {target}");
        }
    }

    #[test]
    fn corrupt_audio_is_rejected() {
        if !ffmpeg_available() {
            return;
        }
        let error = convert(b"no soy audio", "MP3", "WAV", AudioOptions::default()).unwrap_err();
        assert!(error.contains("ffmpeg") || error.contains("dañado"));
    }
}
