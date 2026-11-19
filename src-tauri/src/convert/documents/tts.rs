//! Texto a voz local. Usa la voz instalada en Windows mediante System.Speech
//! (PowerShell) y codifica el WAV resultante a MP3 con FFmpeg. No hay ninguna
//! llamada de red: si la voz o FFmpeg no existen, la ruta no se ofrece.

use std::ffi::OsStr;
use std::fs;

use crate::convert::audio::{self, AudioOptions};
use crate::convert::external;

const MAX_CHARACTERS: usize = 20_000;

pub fn available() -> bool {
    cfg!(windows) && external::powershell().is_some() && external::ffmpeg_has_encoder("libmp3lame")
}

pub fn text_to_mp3(text: &str) -> Result<Vec<u8>, String> {
    if !available() {
        return Err(
            "Convertir texto a MP3 requiere Windows, PowerShell y ffmpeg con codificador MP3."
                .to_string(),
        );
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("El archivo de texto está vacío; no hay nada que narrar.".to_string());
    }
    if trimmed.chars().count() > MAX_CHARACTERS {
        return Err(format!(
            "El texto supera los {MAX_CHARACTERS} caracteres. Divídelo en archivos más pequeños para narrarlo."
        ));
    }
    let program = external::powershell()
        .ok_or_else(|| "No se encontró PowerShell para usar la voz de Windows.".to_string())?;
    let workspace = external::Workspace::new()?;
    let input = workspace.write("entrada.txt", trimmed.as_bytes())?;
    let wave = workspace.path("salida.wav");
    let script = workspace.write(
        "narrar.ps1",
        br#"param(
    [Parameter(Mandatory=$true)][string]$TextPath,
    [Parameter(Mandatory=$true)][string]$WavePath
)
Add-Type -AssemblyName System.Speech
$text = [System.IO.File]::ReadAllText($TextPath, [System.Text.Encoding]::UTF8)
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
try {
    $synth.SetOutputToWaveFile($WavePath)
    $synth.Speak($text)
} finally {
    $synth.Dispose()
}
"#,
    )?;
    let args: Vec<&OsStr> = vec![
        OsStr::new("-NoProfile"),
        OsStr::new("-NonInteractive"),
        OsStr::new("-ExecutionPolicy"),
        OsStr::new("Bypass"),
        OsStr::new("-File"),
        script.as_os_str(),
        OsStr::new("-TextPath"),
        input.as_os_str(),
        OsStr::new("-WavePath"),
        wave.as_os_str(),
    ];
    external::run_tool(&program, &args, None)?;
    let bytes = fs::read(&wave)
        .map_err(|_| "La voz de Windows no generó el audio esperado.".to_string())?;
    if bytes.is_empty() {
        return Err("La voz de Windows generó un audio vacío.".to_string());
    }
    audio::convert(
        &bytes,
        "WAV",
        "MP3",
        AudioOptions {
            bitrate_kbps: 128,
            sample_rate_hz: 44_100,
            channels: 1,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_to_speech_requires_a_local_engine() {
        // La prueba no narra texto real: sólo verifica que los límites se
        // aplican antes de invocar PowerShell.
        if !available() {
            let error = text_to_mp3("hola").unwrap_err();
            assert!(error.contains("PowerShell") || error.contains("Windows"));
        }
    }

    #[test]
    fn empty_text_is_rejected_before_calling_powershell() {
        let error = text_to_mp3("   \n").unwrap_err();
        assert!(error.contains("vacío") || error.contains("requiere"));
    }
}
