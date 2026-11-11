import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import "./App.css";
import {
  convert,
  engineInfo,
  errorMessage,
  formatBytes,
  inspect,
  isDesktopApp,
  listenForDrops,
  pickDestination,
  pickSource,
  showInFolder,
} from "./lib/files";
import type { EngineInfo, FileInfo } from "./lib/files";

type Phase = "idle" | "selected" | "converting" | "done";
type Theme = "dark" | "light";
type IconName =
  | "file"
  | "upload"
  | "download"
  | "arrow"
  | "check"
  | "close"
  | "folder"
  | "shield"
  | "lock"
  | "spark"
  | "sun"
  | "moon"
  | "grid"
  | "list"
  | "info"
  | "alert"
  | "cpu"
  | "refresh"
  | "image"
  | "table";

const ICONS: Record<IconName, ReactNode> = {
  file: (
    <>
      <path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" />
      <path d="M14 3v5h5" />
    </>
  ),
  upload: (
    <>
      <path d="M12 16V4" />
      <path d="m7 9 5-5 5 5" />
      <path d="M5 20h14" />
    </>
  ),
  download: (
    <>
      <path d="M12 4v12" />
      <path d="m7 11 5 5 5-5" />
      <path d="M5 20h14" />
    </>
  ),
  arrow: (
    <>
      <path d="M5 12h14" />
      <path d="m13 6 6 6-6 6" />
    </>
  ),
  check: <path d="m5 13 4 4L19 7" />,
  close: (
    <>
      <path d="M18 6 6 18" />
      <path d="m6 6 12 12" />
    </>
  ),
  folder: <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />,
  shield: (
    <>
      <path d="M12 3 5 6v5c0 4.5 3 7.5 7 10 4-2.5 7-5.5 7-10V6z" />
      <path d="m9 12 2 2 4-4" />
    </>
  ),
  lock: (
    <>
      <rect x="4" y="10" width="16" height="10" rx="2" />
      <path d="M8 10V7a4 4 0 0 1 8 0v3" />
    </>
  ),
  spark: (
    <>
      <path d="M12 3v3" />
      <path d="M12 18v3" />
      <path d="M3 12h3" />
      <path d="M18 12h3" />
      <path d="m5.6 5.6 2.1 2.1" />
      <path d="m16.3 16.3 2.1 2.1" />
      <path d="m18.4 5.6-2.1 2.1" />
      <path d="m7.7 16.3-2.1 2.1" />
    </>
  ),
  sun: (
    <>
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2v2" />
      <path d="M12 20v2" />
      <path d="M4.9 4.9l1.4 1.4" />
      <path d="m17.7 17.7 1.4 1.4" />
      <path d="M2 12h2" />
      <path d="M20 12h2" />
      <path d="m4.9 19.1 1.4-1.4" />
      <path d="m17.7 6.3 1.4-1.4" />
    </>
  ),
  moon: <path d="M20 14.5A8.5 8.5 0 0 1 9.5 4 8.5 8.5 0 1 0 20 14.5z" />,
  grid: (
    <>
      <rect x="4" y="4" width="6" height="6" rx="1" />
      <rect x="14" y="4" width="6" height="6" rx="1" />
      <rect x="4" y="14" width="6" height="6" rx="1" />
      <rect x="14" y="14" width="6" height="6" rx="1" />
    </>
  ),
  list: (
    <>
      <path d="M9 6h11" />
      <path d="M9 12h11" />
      <path d="M9 18h11" />
      <path d="M4 6h.01" />
      <path d="M4 12h.01" />
      <path d="M4 18h.01" />
    </>
  ),
  info: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 11v5" />
      <path d="M12 8h.01" />
    </>
  ),
  alert: (
    <>
      <path d="M12 3 2.8 20h18.4z" />
      <path d="M12 9v5" />
      <path d="M12 17h.01" />
    </>
  ),
  cpu: (
    <>
      <rect x="6" y="6" width="12" height="12" rx="2" />
      <path d="M9 2v3" />
      <path d="M15 2v3" />
      <path d="M9 19v3" />
      <path d="M15 19v3" />
      <path d="M2 9h3" />
      <path d="M2 15h3" />
      <path d="M19 9h3" />
      <path d="M19 15h3" />
    </>
  ),
  refresh: (
    <>
      <path d="M20 11a8 8 0 0 0-14.6-4.2L4 8" />
      <path d="M4 4v4h4" />
      <path d="M4 13a8 8 0 0 0 14.6 4.2L20 16" />
      <path d="M20 20v-4h-4" />
    </>
  ),
  image: (
    <>
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <circle cx="9" cy="10" r="1.6" />
      <path d="m4 18 5-5 3 3 3-4 5 6" />
    </>
  ),
  table: (
    <>
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <path d="M3 10h18" />
      <path d="M9 10v10" />
    </>
  ),
};

function Icon({ name, size = 16, className }: { name: IconName; size?: number; className?: string }) {
  return (
    <svg
      className={className}
      viewBox="0 0 24 24"
      width={size}
      height={size}
      fill="none"
      stroke="currentColor"
      strokeWidth={1.7}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {ICONS[name]}
    </svg>
  );
}

const CATEGORY: Record<string, string> = {
  TXT: "neutral",
  HTML: "indigo",
  MD: "neutral",
  JPG: "amber",
  JPEG: "amber",
  PNG: "teal",
  CSV: "lime",
  TSV: "lime",
  JSON: "yellow",
  XML: "orange",
  PDF: "red",
};

const CATEGORY_OF_FORMAT: Record<string, string> = {
  TXT: "Documentos",
  HTML: "Documentos",
  MD: "Documentos",
  JPG: "Imágenes",
  JPEG: "Imágenes",
  PNG: "Imágenes",
  CSV: "Hojas de cálculo",
  TSV: "Hojas de cálculo",
  JSON: "Hojas de cálculo",
  XML: "Hojas de cálculo",
};

const POPULAR: { from: string; to: string }[] = [
  { from: "TXT", to: "PDF" },
  { from: "TXT", to: "HTML" },
  { from: "TXT", to: "MD" },
  { from: "HTML", to: "MD" },
  { from: "MD", to: "HTML" },
  { from: "JPG", to: "PDF" },
  { from: "JPEG", to: "PDF" },
  { from: "PNG", to: "PDF" },
  { from: "CSV", to: "JSON" },
  { from: "CSV", to: "XML" },
  { from: "TSV", to: "CSV" },
  { from: "JSON", to: "CSV" },
  { from: "XML", to: "CSV" },
];

const FILTERS = ["Todos", "Documentos", "Imágenes", "Hojas de cálculo"];

const FAQ = [
  {
    question: "¿Se guardan mis archivos en un servidor?",
    answer:
      "No. Todo el proceso ocurre dentro de tu dispositivo con el motor local de FileFlow. Los archivos nunca se suben a internet, no se almacenan copias y no se comparten con terceros.",
  },
  {
    question: "¿Qué formatos puedo convertir?",
    answer:
      "Documentos de texto (TXT, HTML, MD), imágenes (JPG, JPEG, PNG) y datos (CSV, TSV, JSON, XML). También puedes generar PDF a partir de TXT e imágenes. Son 10 formatos y 13 combinaciones disponibles.",
  },
  {
    question: "¿Es necesario registrarme o pagar?",
    answer:
      "No. FileFlow es gratuito, funciona sin cuenta y sin conexión a internet. El único límite es de 25 MB por archivo para mantener la conversión rápida y estable.",
  },
  {
    question: "¿Dónde se guarda el archivo convertido?",
    answer:
      "Tú eliges la carpeta y el nombre antes de convertir. Al terminar, el archivo se escribe directamente en tu disco y puedes abrir su ubicación desde la misma ventana.",
  },
];

const SECTIONS = [
  { id: "converter", label: "Conversor", icon: "refresh" as IconName },
  { id: "converters", label: "Conversiones", icon: "grid" as IconName },
  { id: "how-it-works", label: "Cómo funciona", icon: "list" as IconName },
  { id: "privacy", label: "Privacidad", icon: "shield" as IconName },
  { id: "faq", label: "FAQ", icon: "info" as IconName },
];

function FormatBadge({ format }: { format: string }) {
  return (
    <span className="format-badge" data-cat={CATEGORY[format] ?? "neutral"}>
      {format}
    </span>
  );
}

function App() {
  const desktop = isDesktopApp();
  const [theme, setTheme] = useState<Theme>(() => {
    try {
      return localStorage.getItem("fileflow-theme") === "light" ? "light" : "dark";
    } catch {
      return "dark";
    }
  });
  const [engine, setEngine] = useState<EngineInfo | null>(null);
  const [engineError, setEngineError] = useState(false);
  const [phase, setPhase] = useState<Phase>("idle");
  const [file, setFile] = useState<FileInfo | null>(null);
  const [sourcePath, setSourcePath] = useState("");
  const [target, setTarget] = useState("");
  const [outputPath, setOutputPath] = useState("");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [dragOver, setDragOver] = useState(false);
  const [filter, setFilter] = useState("Todos");
  const [openFaq, setOpenFaq] = useState<number | null>(0);
  const [activeSection, setActiveSection] = useState("converter");
  const dropHandler = useRef<(path: string) => void>(() => {});

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem("fileflow-theme", theme);
    } catch {
      /* almacenamiento no disponible */
    }
  }, [theme]);

  useEffect(() => {
    if (!desktop) return;
    let cancelled = false;
    engineInfo()
      .then((info) => {
        if (!cancelled) setEngine(info);
      })
      .catch(() => {
        if (!cancelled) setEngineError(true);
      });
    return () => {
      cancelled = true;
    };
  }, [desktop]);

  const loadPath = useCallback(async (path: string, expectedFrom?: string, expectedTo?: string) => {
    setError("");
    setNotice("");
    setOutputPath("");
    try {
      const info = await inspect(path);
      const requested = expectedTo && info.targets.includes(expectedTo) ? expectedTo : info.targets[0];
      setFile(info);
      setSourcePath(path);
      setTarget(requested ?? "");
      setPhase("selected");
      if (expectedFrom && info.format !== expectedFrom) {
        setNotice(
          `El archivo es ${info.format}. Se ajustaron las opciones disponibles para ese formato.`,
        );
      } else if (expectedTo && requested !== expectedTo) {
        setNotice(`La conversión ${info.format} → ${expectedTo} no está disponible.`);
      }
    } catch (cause) {
      setFile(null);
      setPhase("idle");
      setError(errorMessage(cause));
    }
  }, []);

  dropHandler.current = (path: string) => {
    void loadPath(path);
  };

  useEffect(() => {
    if (!desktop) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    listenForDrops(
      (path) => dropHandler.current(path),
      (active) => setDragOver(active),
    )
      .then((stop) => {
        if (cancelled) stop();
        else unlisten = stop;
      })
      .catch(() => {
        if (!cancelled) setError("No se pudo activar la carga de archivos por arrastre.");
      });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [desktop]);

  useEffect(() => {
    const elements = SECTIONS.map((section) => document.getElementById(section.id)).filter(
      (element): element is HTMLElement => element !== null,
    );
    if (elements.length === 0) return;
    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((entry) => entry.isIntersecting)
          .sort((left, right) => right.intersectionRatio - left.intersectionRatio)[0];
        if (visible) setActiveSection(visible.target.id);
      },
      { rootMargin: "-20% 0px -60% 0px", threshold: [0.1, 0.5] },
    );
    elements.forEach((element) => observer.observe(element));
    return () => observer.disconnect();
  }, []);

  function scrollTo(id: string) {
    document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
  }

  function reset() {
    setPhase("idle");
    setFile(null);
    setSourcePath("");
    setTarget("");
    setOutputPath("");
    setError("");
    setNotice("");
  }

  async function chooseFile(from?: string, to?: string) {
    if (!desktop) {
      setNotice("La selección de archivos está disponible dentro de la app de escritorio.");
      return;
    }
    setError("");
    try {
      const path = await pickSource(from);
      if (path) await loadPath(path, from, to);
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }

  async function convertCurrent() {
    if (!file || !sourcePath || !target || phase === "converting") return;
    setError("");
    setNotice("");
    try {
      const destination = await pickDestination(sourcePath, target);
      if (!destination) return;
      setPhase("converting");
      await convert(sourcePath, target, destination);
      setOutputPath(destination);
      setPhase("done");
    } catch (cause) {
      setError(errorMessage(cause));
      setPhase("selected");
    }
  }

  async function startPopular(from: string, to: string) {
    scrollTo("converter");
    await chooseFile(from, to);
  }

  async function openOutputFolder() {
    if (!outputPath) return;
    try {
      await showInFolder(outputPath);
    } catch (cause) {
      setError(errorMessage(cause));
    }
  }

  const visiblePairs =
    filter === "Todos" ? POPULAR : POPULAR.filter((pair) => CATEGORY_OF_FORMAT[pair.from] === filter);
  const formatTotal = engine?.formats ?? 10;
  const conversionTotal = engine?.conversions ?? 13;
  const maxMb = engine ? Math.round(engine.maxBytes / 1024 / 1024) : 25;
  const outputName = outputPath.split(/[/\\]/).pop() ?? "archivo convertido";

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">
            <Icon name="file" size={18} />
          </span>
          <span className="brand-text">
            <strong>FileFlow</strong>
            <small>Conversor</small>
          </span>
        </div>

        <button className="new-file" type="button" onClick={() => void chooseFile()}>
          <Icon name="upload" size={16} />
          Nuevo archivo
        </button>

        <p className="sidebar-label">Secciones</p>
        <nav className="nav" aria-label="Secciones de la aplicación">
          {SECTIONS.map((section) => (
            <button
              key={section.id}
              type="button"
              className={activeSection === section.id ? "nav-item active" : "nav-item"}
              onClick={() => {
                setActiveSection(section.id);
                scrollTo(section.id);
              }}
            >
              <Icon name={section.icon} size={16} />
              {section.label}
            </button>
          ))}
        </nav>

        <div className="sidebar-status">
          <span className="status-dot" data-state={engineError ? "error" : desktop ? "ok" : "idle"} />
          <div>
            <strong>{engineError ? "Motor no disponible" : desktop ? "Motor local activo" : "Vista previa web"}</strong>
            <small>{desktop ? "Sin conexión · sin cuenta" : "Abre la app de escritorio"}</small>
          </div>
          <span className="version">v{engine?.version ?? "0.1.0"}</span>
        </div>
      </aside>

      <div className="workspace">
        <header className="topbar">
          <div className="breadcrumb">
            <span>FILEFLOW</span>
            <span className="sep">/</span>
            <strong>CONVERSOR</strong>
          </div>
          <div className="topbar-actions">
            <span className="engine-pill" data-state={engineError ? "error" : desktop ? "ok" : "idle"}>
              <span className="status-dot" data-state={engineError ? "error" : desktop ? "ok" : "idle"} />
              {engineError ? "Motor no disponible" : desktop ? "En línea · local" : "Vista previa web"}
            </span>
            <button
              className="icon-button"
              type="button"
              onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
              aria-label={theme === "dark" ? "Activar tema claro" : "Activar tema oscuro"}
              title={theme === "dark" ? "Tema claro" : "Tema oscuro"}
            >
              <Icon name={theme === "dark" ? "sun" : "moon"} size={16} />
            </button>
          </div>
        </header>

        <main className="content">
          <div className="backdrop" aria-hidden="true" />
          <div className="page">
            <section className="hero" id="converter">
              <div className="hero-copy">
                <p className="eyebrow">
                  <Icon name="spark" size={14} />
                  CONVERSOR LOCAL DE ALTA FIDELIDAD
                </p>
                <h1>
                  Tus archivos,
                  <br />
                  en el formato
                  <br />
                  correcto.
                </h1>
                <p className="lede">
                  Documentos de texto, imágenes y datos: convierte entre TXT, HTML, MD, JPG, PNG, CSV, TSV, JSON y
                  XML con un procesador local, rápido y sin instalar nada más.
                </p>
                <div className="stats">
                  <div>
                    <strong>{formatTotal}</strong>
                    <span>FORMATOS</span>
                  </div>
                  <div>
                    <strong>{conversionTotal}</strong>
                    <span>CONVERSIONES</span>
                  </div>
                  <div>
                    <strong>{maxMb} MB</strong>
                    <span>POR ARCHIVO</span>
                  </div>
                </div>
              </div>

              <div className="panel">
                <div className="panel-head">
                  <span className="panel-label">ARCHIVO</span>
                  <span className="panel-state" data-busy={phase === "converting"}>
                    {phase === "converting" ? "PROCESANDO" : phase === "done" ? "COMPLETADO" : file ? "LISTO" : "ESPERANDO"}
                  </span>
                </div>
                <div className="panel-title">
                  <h2>{phase === "done" ? "Conversión terminada" : "Convertir un archivo"}</h2>
                </div>

                {!desktop && (
                  <div className="browser-note" role="status">
                    <Icon name="info" size={16} />
                    <span>
                      Esta es una vista previa web. Ejecuta la app de escritorio con <code>pnpm tauri dev</code> para
                      convertir archivos con el motor local.
                    </span>
                  </div>
                )}

                {error && (
                  <div className="alert" role="alert">
                    <Icon name="alert" size={16} />
                    <span>{error}</span>
                    <button type="button" onClick={() => setError("")} aria-label="Cerrar error">
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

                {phase === "done" && file ? (
                  <div className="success">
                    <span className="success-icon">
                      <Icon name="check" size={22} />
                    </span>
                    <div className="success-copy">
                      <strong>{outputName}</strong>
                      <span>
                        {file.name} → {target} · guardado en tu dispositivo
                      </span>
                    </div>
                    <div className="success-actions">
                      <button className="primary" type="button" onClick={() => void openOutputFolder()}>
                        <Icon name="folder" size={15} />
                        Ver en carpeta
                      </button>
                      <button className="ghost" type="button" onClick={reset}>
                        <Icon name="refresh" size={15} />
                        Convertir otro
                      </button>
                    </div>
                  </div>
                ) : file && (phase === "selected" || phase === "converting") ? (
                  <div className="selected">
                    <div className="file-row">
                      <FormatBadge format={file.format} />
                      <div className="file-meta">
                        <strong title={file.name}>{file.name}</strong>
                        <span>
                          {formatBytes(file.size)}
                          {file.width && file.height ? ` · ${file.width} × ${file.height} px` : ""}
                        </span>
                      </div>
                      <button
                        className="icon-button small"
                        type="button"
                        onClick={reset}
                        aria-label={`Quitar ${file.name}`}
                        disabled={phase === "converting"}
                      >
                        <Icon name="close" size={14} />
                      </button>
                    </div>

                    <div className="targets">
                      <span className="field-label">Convertir a</span>
                      <div className="chips" role="group" aria-label="Formato de destino">
                        {file.targets.map((option) => (
                          <button
                            key={option}
                            type="button"
                            className={option === target ? "chip active" : "chip"}
                            onClick={() => setTarget(option)}
                            aria-pressed={option === target}
                            disabled={phase === "converting"}
                          >
                            {option}
                          </button>
                        ))}
                      </div>
                    </div>

                    <button
                      className="primary wide"
                      type="button"
                      onClick={() => void convertCurrent()}
                      disabled={!target || phase === "converting"}
                    >
                      {phase === "converting" ? (
                        <>
                          <span className="spinner" />
                          Procesando localmente…
                        </>
                      ) : (
                        <>
                          <Icon name="download" size={15} />
                          Convertir a {target || "…"}
                        </>
                      )}
                    </button>
                    <p className="privacy-line">
                      <Icon name="lock" size={13} />
                      El archivo se procesa en tu dispositivo y no se sube a ningún servidor.
                    </p>
                  </div>
                ) : (
                  <>
                    <button
                      type="button"
                      className={dragOver ? "dropzone dragging" : "dropzone"}
                      onClick={() => void chooseFile()}
                      disabled={!desktop}
                    >
                      <span className="dropzone-icon">
                        <Icon name="upload" size={20} />
                      </span>
                      <strong>Arrastra un archivo aquí</strong>
                      <span>o elige uno desde tu dispositivo — soporta documentos, imágenes y hojas de cálculo.</span>
                      <span className="fake-button">Seleccionar archivo</span>
                    </button>
                    <div className="formats-block">
                      <div className="formats-head">
                        <span>FORMATOS</span>
                        <span>HASTA {maxMb} MB</span>
                      </div>
                      <div className="format-chips">
                        {["TXT", "HTML", "MD", "JPG", "JPEG", "PNG", "CSV", "TSV", "JSON", "XML"].map((format) => (
                          <FormatBadge key={format} format={format} />
                        ))}
                      </div>
                    </div>
                  </>
                )}
              </div>
            </section>

            <section className="section" id="converters">
              <div className="section-head">
                <div>
                  <p className="eyebrow">ACCESOS RÁPIDOS</p>
                  <h2>Conversiones populares</h2>
                  <p className="section-sub">Haz clic en una opción para configurarla directamente en el conversor.</p>
                </div>
                <div className="filters" role="group" aria-label="Filtrar conversiones">
                  {FILTERS.map((option) => (
                    <button
                      key={option}
                      type="button"
                      className={filter === option ? "filter active" : "filter"}
                      onClick={() => setFilter(option)}
                      aria-pressed={filter === option}
                    >
                      {option}
                    </button>
                  ))}
                </div>
              </div>

              <div className="pairs">
                {visiblePairs.map((pair) => (
                  <button
                    key={`${pair.from}-${pair.to}`}
                    type="button"
                    className="pair"
                    onClick={() => void startPopular(pair.from, pair.to)}
                  >
                    <FormatBadge format={pair.from} />
                    <Icon name="arrow" size={13} className="pair-arrow" />
                    <FormatBadge format={pair.to} />
                    <span className="pair-action">
                      Convertir
                      <Icon name="arrow" size={13} />
                    </span>
                  </button>
                ))}
              </div>
              <p className="micro-caption">
                GENERADO DINÁMICAMENTE A PARTIR DE LAS {conversionTotal} CONVERSIONES COMPATIBLES
              </p>
            </section>

            <section className="section" id="how-it-works">
              <div className="section-head">
                <div>
                  <p className="eyebrow">FLUJO DE TRABAJO</p>
                  <h2>Cómo funciona</h2>
                </div>
              </div>
              <div className="steps">
                <article className="step">
                  <span className="step-number">01</span>
                  <span className="step-icon">
                    <Icon name="upload" size={18} />
                  </span>
                  <h3>Arrastra tu archivo</h3>
                  <p>
                    Sube documentos, imágenes o tablas de hasta {maxMb} MB. El motor local detecta el formato al
                    instante.
                  </p>
                </article>
                <article className="step">
                  <span className="step-number">02</span>
                  <span className="step-icon">
                    <Icon name="list" size={18} />
                  </span>
                  <h3>Elige el formato destino</h3>
                  <p>
                    Solo se muestran las opciones compatibles con tu archivo. Nunca verás una combinación que no
                    funcione.
                  </p>
                </article>
                <article className="step">
                  <span className="step-number">03</span>
                  <span className="step-icon">
                    <Icon name="download" size={18} />
                  </span>
                  <h3>Convierte y guarda</h3>
                  <p>
                    El procesador local convierte el archivo en segundos y lo escribe en la carpeta que elijas, sin
                    salir de tu equipo.
                  </p>
                </article>
              </div>
            </section>

            <section className="section" id="privacy">
              <div className="privacy-card">
                <div className="privacy-head">
                  <p className="eyebrow">
                    <Icon name="shield" size={14} />
                    PRIVACIDAD
                  </p>
                  <span className="privacy-link">
                    <Icon name="cpu" size={14} />
                    MOTOR LOCAL
                  </span>
                </div>
                <h2>Privacidad y seguridad de tus archivos</h2>
                <p className="section-sub">
                  La validación y la conversión se realizan directamente en tu dispositivo. Los archivos no viajan a
                  ningún servidor, no se almacenan copias y no se comparten con terceros.
                </p>

                <div className="privacy-rows">
                  <div className="privacy-row">
                    <span className="privacy-label">DOCUMENTOS · TXT × 3</span>
                    <FormatBadge format="TXT" />
                    <Icon name="arrow" size={12} className="row-arrow" />
                    <span className="row-targets">
                      <FormatBadge format="PDF" />
                      <FormatBadge format="HTML" />
                      <FormatBadge format="MD" />
                    </span>
                  </div>
                  <div className="privacy-row">
                    <span className="privacy-label">IMÁGENES · JPG × 1</span>
                    <FormatBadge format="JPG" />
                    <Icon name="arrow" size={12} className="row-arrow" />
                    <span className="row-targets">
                      <FormatBadge format="PDF" />
                    </span>
                  </div>
                  <div className="privacy-row">
                    <span className="privacy-label">HOJAS DE CÁLCULO Y DATOS · CSV × 2</span>
                    <FormatBadge format="CSV" />
                    <Icon name="arrow" size={12} className="row-arrow" />
                    <span className="row-targets">
                      <FormatBadge format="JSON" />
                      <FormatBadge format="XML" />
                    </span>
                  </div>
                </div>

                <div className="privacy-foot">
                  <span className="status-dot" data-state={engineError ? "error" : desktop ? "ok" : "idle"} />
                  {engineError
                    ? "No se pudo consultar el motor local."
                    : desktop
                      ? `Procesamiento 100% local · ${engine?.name ?? "FileFlow Engine"} v${engine?.version ?? "0.1.0"} · ${engine?.platform ?? "escritorio"}`
                      : "Vista previa web: sin acceso al motor local."}
                </div>
              </div>
            </section>

            <section className="section" id="faq">
              <div className="section-head">
                <div>
                  <p className="eyebrow">DUDAS FRECUENTES</p>
                  <h2>Preguntas frecuentes</h2>
                </div>
              </div>
              <div className="faq">
                {FAQ.map((item, index) => {
                  const open = openFaq === index;
                  return (
                    <div key={item.question} className={open ? "faq-item open" : "faq-item"}>
                      <button
                        type="button"
                        onClick={() => setOpenFaq(open ? null : index)}
                        aria-expanded={open}
                      >
                        <span>{item.question}</span>
                        <Icon name={open ? "close" : "arrow"} size={15} className={open ? "faq-close" : "faq-open"} />
                      </button>
                      {open && <p>{item.answer}</p>}
                    </div>
                  );
                })}
              </div>
            </section>

            <footer className="footer">
              <div className="footer-brand">
                <span className="brand-mark small">
                  <Icon name="file" size={15} />
                </span>
                <div>
                  <strong>FileFlow</strong>
                  <p>Tus archivos, en el formato correcto.</p>
                </div>
              </div>
              <div className="footer-links">
                <div>
                  <span>PRODUCTO</span>
                  <button type="button" onClick={() => scrollTo("converter")}>Convertir un archivo</button>
                  <button type="button" onClick={() => scrollTo("converters")}>Conversiones populares</button>
                  <button type="button" onClick={() => scrollTo("how-it-works")}>Cómo funciona</button>
                </div>
                <div>
                  <span>LEGAL Y CONTACTO</span>
                  <button type="button" onClick={() => scrollTo("privacy")}>Privacidad</button>
                  <button type="button" onClick={() => scrollTo("faq")}>Preguntas frecuentes</button>
                  <span className="footer-email">hola@fileflow.app</span>
                </div>
              </div>
              <div className="footer-status">
                <span className="engine-pill" data-state={engineError ? "error" : desktop ? "ok" : "idle"}>
                  <span className="status-dot" data-state={engineError ? "error" : desktop ? "ok" : "idle"} />
                  {engineError ? "MOTOR NO DISPONIBLE" : desktop ? "MOTOR LOCAL ACTIVO" : "VISTA WEB"}
                </span>
                <span className="version">v{engine?.version ?? "0.1.0"}</span>
              </div>
              <div className="footer-bottom">
                <span>© 2026 FileFlow · Conversor de archivos local</span>
                <span>HECHO PARA SER RÁPIDO</span>
              </div>
            </footer>
          </div>
        </main>
      </div>
    </div>
  );
}

export default App;
