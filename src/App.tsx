import { useEffect, useRef, useState } from "react";
import "./App.css";
import { Sidebar, type NavSection } from "./components/layout/Sidebar";
import { Topbar } from "./components/layout/Topbar";
import { ConverterView } from "./features/converter/ConverterView";
import { ConversionCatalog } from "./features/converter/ConversionCatalog";
import { PrivacyView } from "./features/privacy/PrivacyView";
import { FaqView, type FaqItem } from "./features/faq/FaqView";
import {
  getConversionMatrix,
  getEngineInfo,
  isDesktopApp,
  listenFileDrops,
} from "./lib/tauri";
import type { ConversionMatrix, EngineInfo, RouteRequest, Theme } from "./types";

const FILTERS = [
  "Todos",
  "Documentos",
  "Imágenes",
  "Hojas de cálculo",
  "Presentaciones",
  "Audio",
  "Vídeo",
  "Comprimidos",
];

const FAQ_ITEMS: FaqItem[] = [
  {
    question: "¿Se guardan mis archivos en algún servidor?",
    answer:
      "No. Todo el proceso ocurre íntegramente dentro de tu dispositivo con el motor local de FileFlow. Los archivos nunca se suben a internet, no se almacenan copias externas y no se comparte información con terceros.",
  },
  {
    question: "¿Qué formatos puedo convertir?",
    answer:
      "Soporta 50 formatos: documentos (DOCX, DOC, ODT, RTF, TXT, MD, HTML, PDF y EPUB), imágenes (JPG, PNG, WEBP, BMP, GIF, TIFF, HEIC, SVG, AVIF e ICO), hojas de cálculo y datos (XLSX, XLS, ODS, CSV, TSV, JSON, XML, YAML y SQL), presentaciones (PPTX, PPT y ODP), audio (MP3, WAV, FLAC, AAC, OGG, M4A, WMA, OPUS y AIFF), vídeo (MP4, MOV, AVI, MKV, WEBM, WMV y FLV) y comprimidos (ZIP, RAR y 7Z).",
  },
  {
    question: "¿Necesito instalar algo para audio, vídeo o formatos de Office antiguos?",
    answer:
      "La mayoría de conversiones funcionan con el motor nativo de Rust. Para códecs multimedia (HEIC, AVIF, audio y vídeo) se apoya en un FFmpeg local; DOC, PPT y rasterización de PDF requieren LibreOffice o Ghostscript; RAR y 7Z requieren WinRAR o 7-Zip. Si falta una herramienta, FileFlow oculta esa ruta y lo explica claramente en lugar de fallar a mitad del proceso.",
  },
  {
    question: "¿Es necesario registrarse o pagar?",
    answer:
      "No. FileFlow es completamente gratuito, sin cuentas, sin telemetría y sin requerir conexión a internet.",
  },
  {
    question: "¿Dónde se guarda el archivo convertido?",
    answer:
      "Tú eliges la carpeta y el nombre antes de convertir mediante el selector nativo de tu sistema operativo. Al terminar, puedes pulsar «Ver en carpeta» para abrir directamente la ubicación con el archivo seleccionado.",
  },
];

const SECTIONS: NavSection[] = [
  { id: "converter", label: "Conversor", icon: "refresh" },
  { id: "converters", label: "Catálogo", icon: "grid" },
  { id: "privacy", label: "Privacidad", icon: "shield" },
  { id: "faq", label: "Ayuda y FAQ", icon: "info" },
];

export function App() {
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
  const [matrix, setMatrix] = useState<ConversionMatrix | null>(null);
  const [matrixLoading, setMatrixLoading] = useState(desktop);
  const [matrixError, setMatrixError] = useState(false);
  const [catalogQuery, setCatalogQuery] = useState("");
  const [showUnavailable, setShowUnavailable] = useState(false);
  const [routeRequest, setRouteRequest] = useState<RouteRequest | null>(null);
  const [droppedPath, setDroppedPath] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const [filter, setFilter] = useState("Todos");
  const [openFaq, setOpenFaq] = useState<number | null>(0);
  const [activeSection, setActiveSection] = useState("converter");
  const routeNonce = useRef(0);

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
    getEngineInfo()
      .then((info) => {
        if (!cancelled) setEngine(info);
      })
      .catch(() => {
        if (!cancelled) setEngineError(true);
      });
    getConversionMatrix()
      .then((data) => {
        if (!cancelled) setMatrix(data);
      })
      .catch(() => {
        if (!cancelled) setMatrixError(true);
      })
      .finally(() => {
        if (!cancelled) setMatrixLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [desktop]);

  useEffect(() => {
    if (!desktop) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    listenFileDrops(
      (path) => {
        setDroppedPath(path);
        setActiveSection("converter");
      },
      (active) => setDragOver(active),
    )
      .then((stop) => {
        if (cancelled) stop();
        else unlisten = stop;
      })
      .catch(() => {});

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [desktop]);

  function handleSelectRoute(from: string, to: string) {
    routeNonce.current += 1;
    setRouteRequest({ from, to, nonce: routeNonce.current });
    setActiveSection("converter");
  }

  const formatTotal = engine?.formats ?? 50;
  const conversionTotal = engine?.conversions ?? 476;
  const maxMb = engine ? Math.round(engine.maxBytes / 1024 / 1024) : 25;

  return (
    <div className="app">
      <Sidebar
        sections={SECTIONS}
        activeSection={activeSection}
        onSelectSection={setActiveSection}
        engineError={engineError}
        desktop={desktop}
        engineVersion={engine?.version}
      />

      <div className="workspace">
        <Topbar
          activeSection={activeSection}
          engineError={engineError}
          desktop={desktop}
          theme={theme}
          onToggleTheme={() => setTheme(theme === "dark" ? "light" : "dark")}
        />

        <main className="content">
          <div className="page">
            {activeSection === "converter" && (
              <ConverterView
                desktop={desktop}
                maxMb={maxMb}
                formatTotal={formatTotal}
                conversionTotal={conversionTotal}
                dragOver={dragOver}
                routeRequest={routeRequest}
                onRouteRequestHandled={() => setRouteRequest(null)}
                droppedPath={droppedPath}
                onClearExternalRequest={() => setDroppedPath(null)}
                filter={filter}
                onSelectFilter={setFilter}
                filters={FILTERS}
                onSelectRoute={handleSelectRoute}
                onNavigateToCatalog={() => setActiveSection("converters")}
              />
            )}

            {activeSection === "converters" && (
              <div className="view-container">
                <div className="view-header">
                  <div className="view-header-copy">
                    <h1>Catálogo completo de conversiones</h1>
                    <p>
                      Explora los 50 formatos y las 476 combinaciones verificadas por el motor local.
                      Haz clic en cualquier destino para iniciar la conversión inmediatamente.
                    </p>
                  </div>
                </div>

                <div className="filters-bar" role="group" aria-label="Filtrar conversiones por categoría">
                  {FILTERS.map((opt) => (
                    <button
                      key={opt}
                      type="button"
                      className={filter === opt ? "filter active" : "filter"}
                      onClick={() => setFilter(opt)}
                      aria-pressed={filter === opt}
                    >
                      {opt}
                    </button>
                  ))}
                </div>

                <ConversionCatalog
                  matrix={matrix}
                  loading={matrixLoading}
                  error={matrixError}
                  category={filter}
                  query={catalogQuery}
                  onQueryChange={setCatalogQuery}
                  showUnavailable={showUnavailable}
                  onToggleUnavailable={() => setShowUnavailable((value) => !value)}
                  onSelectRoute={handleSelectRoute}
                />
              </div>
            )}

            {activeSection === "privacy" && (
              <PrivacyView
                engine={engine}
                engineError={engineError}
                desktop={desktop}
                maxMb={maxMb}
              />
            )}

            {activeSection === "faq" && (
              <FaqView
                faqItems={FAQ_ITEMS}
                openFaq={openFaq}
                onToggleFaq={(index) => setOpenFaq(openFaq === index ? null : index)}
              />
            )}
          </div>
        </main>
      </div>
    </div>
  );
}

export default App;
