import { Icon } from "../../components/Icon";
import type { EngineInfo } from "../../types";

interface PrivacyViewProps {
  engine: EngineInfo | null;
  engineError: boolean;
  desktop: boolean;
  maxMb: number;
}

export function PrivacyView({
  engine,
  engineError,
  desktop,
  maxMb,
}: PrivacyViewProps) {
  return (
    <div className="view-container">
      <div className="view-header">
        <div className="view-header-copy">
          <h1>Privacidad y Arquitectura Local</h1>
          <p>
            FileFlow Desktop está diseñado bajo una premisa innegociable: tus archivos nunca salen de tu ordenador.
          </p>
        </div>
      </div>

      <div className="privacy-panel">
        <div className="privacy-grid">
          <article className="privacy-feature">
            <div className="privacy-feature-icon" aria-hidden="true">
              <Icon name="cpu" size={18} />
            </div>
            <h3>Procesamiento 100% nativo</h3>
            <p>
              Todo el trabajo de lectura, transformación y escritura se ejecuta en la CPU de tu máquina
              usando motores compilados en Rust.
            </p>
          </article>

          <article className="privacy-feature">
            <div className="privacy-feature-icon" aria-hidden="true">
              <Icon name="shield" size={18} />
            </div>
            <h3>Cero telemetría ni red</h3>
            <p>
              La aplicación funciona completamente desconectada de internet. No hay analíticas, rastreo de uso
              ni registros remotos de tus archivos.
            </p>
          </article>

          <article className="privacy-feature">
            <div className="privacy-feature-icon" aria-hidden="true">
              <Icon name="download" size={18} />
            </div>
            <h3>Archivos en tu control</h3>
            <p>
              Los archivos temporales se limpian en cuanto finaliza el proceso o se cierra la aplicación. Los
              archivos finales se depositan donde tú indiques.
            </p>
          </article>
        </div>

        <div className="engine-specs-box">
          <span>MOTOR: {engine?.name ?? "FileFlow Engine"} v{engine?.version ?? "0.1.0"}</span>
          <span>PLATAFORMA: {engine?.platform ?? "desktop"}</span>
          <span>LÍMITE POR ARCHIVO: {maxMb} MB</span>
          <span>ESTADO: {engineError ? "ERROR" : desktop ? "ACTIVO Y PROTEGIDO" : "VISTA WEB"}</span>
        </div>
      </div>
    </div>
  );
}
