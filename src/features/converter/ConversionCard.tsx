import { useEffect, useRef, useState } from "react";
import { Icon } from "../../components/Icon";
import { FormatBadge } from "../../components/FormatBadge";
import { DropZone } from "./DropZone";
import { FormatSelector } from "./FormatSelector";
import { AudioSettings } from "./AudioSettings";
import { formatBytes, formatDuration, isAudioTarget, DEFAULT_AUDIO_PARAMS } from "../../lib/formats";
import {
  pickSourceFile,
  pickDestinationPath,
  inspectFile,
  convertFile,
  showItemInFolder,
} from "../../lib/tauri";
import { normalizeError } from "../../lib/errors";
import type { AudioParams, ConversionState, FileInfo, RouteRequest } from "../../types";

interface ConversionCardProps {
  desktop: boolean;
  maxMb?: number;
  isDragging: boolean;
  onFileLoaded?: (file: FileInfo) => void;
  externalLoadRequest?: { path: string; expectedFrom?: string; expectedTo?: string } | null;
  onClearExternalRequest?: () => void;
  routeRequest?: RouteRequest | null;
  onRouteRequestHandled?: () => void;
}

export function ConversionCard({
  desktop,
  maxMb = 25,
  isDragging,
  routeRequest,
  onRouteRequestHandled,
  externalLoadRequest,
  onClearExternalRequest,
}: ConversionCardProps) {
  const [state, setState] = useState<ConversionState>({ status: "idle" });
  const [audio, setAudio] = useState<AudioParams>(DEFAULT_AUDIO_PARAMS);
  const [notice, setNotice] = useState<string>("");
  const handledRequest = useRef(0);
  const handledDrop = useRef<string | null>(null);

  async function handleSelectFile(fromFormat?: string, toFormat?: string) {
    if (!desktop) {
      setNotice("La selección de archivos requiere la app de escritorio.");
      return;
    }
    setNotice("");
    try {
      const path = await pickSourceFile(fromFormat);
      if (path) {
        await loadPath(path, fromFormat, toFormat);
      }
    } catch (err) {
      const normalized = normalizeError(err);
      setState({ status: "error", message: normalized.message, code: normalized.code });
    }
  }

  useEffect(() => {
    if (!routeRequest || routeRequest.nonce === handledRequest.current) return;
    handledRequest.current = routeRequest.nonce;
    void handleSelectFile(routeRequest.from, routeRequest.to);
    onRouteRequestHandled?.();
    // El nonce identifica cada clic del catálogo; handleSelectFile se recrea en
    // cada render y no debe reiniciar la petición.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [routeRequest]);

  async function loadPath(path: string, expectedFrom?: string, expectedTo?: string) {
    setNotice("");
    setState({ status: "validating", path });
    try {
      const info = await inspectFile(path);
      const chosenTarget =
        expectedTo && info.targets.includes(expectedTo)
          ? expectedTo
          : info.targets[0] ?? "";

      setState({
        status: "selected",
        file: info,
        sourcePath: path,
        target: chosenTarget,
      });

      if (expectedFrom && info.format !== expectedFrom) {
        setNotice(`El archivo es ${info.format}. Se adaptaron los formatos de salida disponibles.`);
      } else if (expectedTo && chosenTarget !== expectedTo) {
        setNotice(`La conversión de ${info.format} a ${expectedTo} no está soportada.`);
      }
    } catch (err) {
      const normalized = normalizeError(err);
      setState({ status: "error", message: normalized.message, code: normalized.code });
    }
  }

  useEffect(() => {
    if (!externalLoadRequest) {
      handledDrop.current = null;
      return;
    }
    if (externalLoadRequest.path === handledDrop.current) return;
    handledDrop.current = externalLoadRequest.path;
    void loadPath(
      externalLoadRequest.path,
      externalLoadRequest.expectedFrom,
      externalLoadRequest.expectedTo,
    );
    onClearExternalRequest?.();
    // La ruta identifica el archivo soltado; loadPath se recrea en cada render.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [externalLoadRequest]);

  async function handleConvert() {
    if (state.status !== "selected") return;
    const { file, sourcePath, target } = state;
    if (!target) return;

    setNotice("");
    try {
      const destination = await pickDestinationPath(sourcePath, target);
      if (!destination) return;

      setState({
        status: "converting",
        file,
        sourcePath,
        target,
        progressLabel: `Convirtiendo a ${target} de forma 100% local…`,
      });

      await convertFile(
        sourcePath,
        target,
        destination,
        file.category === "audio" ||
          (file.category === "video" && isAudioTarget(target))
          ? { audio }
          : undefined,
      );

      setState({
        status: "done",
        file,
        sourcePath,
        target,
        outputPath: destination,
      });
    } catch (err) {
      const normalized = normalizeError(err);
      setState({ status: "error", message: normalized.message, code: normalized.code });
    }
  }

  function handleReset() {
    setState({ status: "idle" });
    setNotice("");
    setAudio(DEFAULT_AUDIO_PARAMS);
  }

  async function handleOpenFolder(path: string) {
    try {
      await showItemInFolder(path);
    } catch (err) {
      const normalized = normalizeError(err);
      setNotice(normalized.message);
    }
  }

  const isConverting = state.status === "converting";
  const isValidating = state.status === "validating";

  return (
    <div className="panel" aria-live="polite">
      <div className="panel-head">
        <span className="panel-label">ARCHIVO</span>
        <span
          className="panel-state"
          data-busy={isConverting || isValidating}
        >
          {isConverting
            ? "PROCESANDO"
            : isValidating
              ? "VALIDANDO"
              : state.status === "done"
                ? "COMPLETADO"
                : state.status === "selected"
                  ? "LISTO"
                  : "ESPERANDO"}
        </span>
      </div>

      <div className="panel-title">
        <h2>
          {state.status === "done"
            ? "Conversión terminada"
            : isConverting
              ? "Procesando conversión"
              : isValidating
                ? "Inspeccionando archivo"
                : state.status === "selected"
                  ? "Configurar conversión"
                  : "Convertir un archivo"}
        </h2>
      </div>

      {!desktop && (
        <div className="browser-note" role="status">
          <Icon name="info" size={16} />
          <span>
            Modo vista previa web. Inicia la app de escritorio con <code>pnpm tauri dev</code> para convertir sin límites.
          </span>
        </div>
      )}

      {state.status === "error" && (
        <div className="alert" role="alert">
          <Icon name="alert" size={16} />
          <div className="alert-content">
            <strong>{state.message}</strong>
          </div>
          <button type="button" onClick={handleReset} aria-label="Cerrar error y volver">
            <Icon name="close" size={14} />
          </button>
        </div>
      )}

      {notice && (
        <div className="notice" role="status">
          <Icon name="info" size={16} />
          <span>{notice}</span>
          <button type="button" onClick={() => setNotice("")} aria-label="Cerrar aviso">
            <Icon name="close" size={14} />
          </button>
        </div>
      )}

      {isValidating && (
        <div className="validating-state">
          <span className="spinner" aria-hidden="true" />
          <p>Leyendo metadatos del archivo seleccionado…</p>
        </div>
      )}

      {state.status === "done" && (
        <div className="success">
          <span className="success-icon">
            <Icon name="check" size={22} />
          </span>
          <div className="success-copy">
            <strong>{state.outputPath.split(/[/\\]/).pop() ?? "archivo convertido"}</strong>
            <span>
              {state.file.name} → {state.target} · Guardado en tu almacenamiento local
            </span>
          </div>
          <div className="success-actions">
            <button
              className="primary"
              type="button"
              onClick={() => void handleOpenFolder(state.outputPath)}
            >
              <Icon name="folder" size={15} />
              Ver en carpeta
            </button>
            <button className="ghost" type="button" onClick={handleReset}>
              <Icon name="refresh" size={15} />
              Convertir otro
            </button>
          </div>
        </div>
      )}

      {(state.status === "selected" || isConverting) && (
        <div className="selected">
          <div className="file-row">
            <FormatBadge format={state.file.format} />
            <div className="file-meta">
              <strong title={state.file.name}>{state.file.name}</strong>
              <span>
                {formatBytes(state.file.size)}
                {state.file.width && state.file.height
                  ? ` · ${state.file.width} × ${state.file.height} px`
                  : ""}
                {state.file.durationMs
                  ? ` · ${formatDuration(state.file.durationMs)}`
                  : ""}
                {` · ${state.file.mime}`}
              </span>
            </div>
            <button
              className="icon-button small"
              type="button"
              onClick={handleReset}
              aria-label={`Quitar archivo ${state.file.name}`}
              disabled={isConverting}
            >
              <Icon name="close" size={14} />
            </button>
          </div>

          <FormatSelector
            targets={state.file.targets}
            selectedTarget={state.target}
            onSelectTarget={(t) => setState({ ...state, target: t })}
            disabled={isConverting}
          />

          {state.file.category === "audio" ||
          (state.file.category === "video" && isAudioTarget(state.target)) ? (
            <AudioSettings
              audio={audio}
              onChange={setAudio}
              disabled={isConverting}
            />
          ) : null}

          <button
            className="primary wide"
            type="button"
            onClick={() => void handleConvert()}
            disabled={!state.target || isConverting}
          >
            {isConverting ? (
              <>
                <span className="spinner" aria-hidden="true" />
                {state.progressLabel}
              </>
            ) : (
              <>
                <Icon name="download" size={15} />
                Convertir a {state.target || "…"}
              </>
            )}
          </button>

          <p className="privacy-line">
            <Icon name="lock" size={13} />
            El archivo se procesa íntegramente en tu máquina y nunca sale de tu equipo.
          </p>
        </div>
      )}

      {state.status === "idle" && (
        <DropZone
          isDragging={isDragging}
          isDisabled={!desktop}
          maxMb={maxMb}
          onChooseFile={() => void handleSelectFile()}
        />
      )}
    </div>
  );
}
