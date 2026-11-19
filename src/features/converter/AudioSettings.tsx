import type { AudioParams } from "../../types";

interface AudioSettingsProps {
  audio: AudioParams;
  onChange: (audio: AudioParams) => void;
  disabled?: boolean;
}

const BITRATES = [64, 96, 128, 192, 256, 320];
const SAMPLE_RATES = [22050, 32000, 44100, 48000, 96000];

export function AudioSettings({ audio, onChange, disabled = false }: AudioSettingsProps) {
  return (
    <div className="audio-params">
      <span className="field-label" id="audio-params-label">
        Parámetros de audio
      </span>
      <div className="audio-grid" role="group" aria-labelledby="audio-params-label">
        <label>
          <span>Bitrate</span>
          <select
            value={audio.bitrateKbps}
            onChange={(e) => onChange({ ...audio, bitrateKbps: Number(e.target.value) })}
            disabled={disabled}
            aria-label="Tasa de bits"
          >
            {BITRATES.map((val) => (
              <option key={val} value={val}>
                {val} kbps
              </option>
            ))}
          </select>
        </label>

        <label>
          <span>Frecuencia</span>
          <select
            value={audio.sampleRateHz}
            onChange={(e) => onChange({ ...audio, sampleRateHz: Number(e.target.value) })}
            disabled={disabled}
            aria-label="Frecuencia de muestreo"
          >
            {SAMPLE_RATES.map((val) => (
              <option key={val} value={val}>
                {(val / 1000).toLocaleString("es-ES")} kHz
              </option>
            ))}
          </select>
        </label>

        <label>
          <span>Canales</span>
          <select
            value={audio.channels}
            onChange={(e) => onChange({ ...audio, channels: Number(e.target.value) })}
            disabled={disabled}
            aria-label="Número de canales"
          >
            <option value={1}>Mono</option>
            <option value={2}>Estéreo</option>
          </select>
        </label>
      </div>
      <p className="audio-hint">
        Los formatos sin pérdida (WAV, FLAC, AIFF) preservan la calidad original sin compresión destructiva.
      </p>
    </div>
  );
}
