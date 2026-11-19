import { Icon } from "../../components/Icon";
import { FormatBadge } from "../../components/FormatBadge";
import { ConversionCard } from "./ConversionCard";
import { POPULAR_CONVERSIONS, CATEGORY_OF_FORMAT } from "../../lib/formats";
import type { RouteRequest } from "../../types";

interface ConverterViewProps {
  desktop: boolean;
  maxMb: number;
  formatTotal: number;
  conversionTotal: number;
  dragOver: boolean;
  routeRequest: RouteRequest | null;
  onRouteRequestHandled: () => void;
  droppedPath: string | null;
  onClearExternalRequest: () => void;
  filter: string;
  onSelectFilter: (filter: string) => void;
  filters: string[];
  onSelectRoute: (from: string, to: string) => void;
  onNavigateToCatalog: () => void;
}

export function ConverterView({
  desktop,
  maxMb,
  formatTotal,
  conversionTotal,
  dragOver,
  routeRequest,
  onRouteRequestHandled,
  droppedPath,
  onClearExternalRequest,
  filter,
  onSelectFilter,
  filters,
  onSelectRoute,
  onNavigateToCatalog,
}: ConverterViewProps) {
  const visiblePairs =
    filter === "Todos"
      ? POPULAR_CONVERSIONS.slice(0, 12)
      : POPULAR_CONVERSIONS.filter((pair) => CATEGORY_OF_FORMAT[pair.from] === filter);

  return (
    <div className="view-container">
      <div className="view-header">
        <div className="view-header-copy">
          <h1>Convierte cualquier archivo. Sube nada.</h1>
          <p>
            Documentos, imágenes, tablas, presentaciones, audio, vídeo y comprimidos procesados
            localmente con la máxima velocidad de Rust.
          </p>
        </div>
        <div className="quick-stats-row">
          <div className="stat-chip">
            <strong>{formatTotal}</strong> Formatos
          </div>
          <div className="stat-chip">
            <strong>{conversionTotal}</strong> Rutas
          </div>
          <div className="stat-chip">
            <strong>Hasta {maxMb} MB</strong>
          </div>
        </div>
      </div>

      <ConversionCard
        desktop={desktop}
        maxMb={maxMb}
        isDragging={dragOver}
        routeRequest={routeRequest}
        onRouteRequestHandled={onRouteRequestHandled}
        externalLoadRequest={droppedPath ? { path: droppedPath } : null}
        onClearExternalRequest={onClearExternalRequest}
      />

      <div className="catalog-quick">
        <div className="catalog-quick-header">
          <span className="catalog-quick-label">ACCESOS RÁPIDOS MÁS USADOS</span>
          <div className="filters-bar-compact" role="group" aria-label="Filtrar accesos rápidos">
            {filters.map((opt) => (
              <button
                key={opt}
                type="button"
                className={filter === opt ? "filter-compact active" : "filter-compact"}
                onClick={() => onSelectFilter(opt)}
                aria-pressed={filter === opt}
              >
                {opt}
              </button>
            ))}
          </div>
        </div>

        {visiblePairs.length > 0 ? (
          <div className="pairs" role="list">
            {visiblePairs.map((pair) => (
              <button
                key={`${pair.from}-${pair.to}`}
                type="button"
                className="pair"
                onClick={() => onSelectRoute(pair.from, pair.to)}
                aria-label={`Preparar conversión de ${pair.from} a ${pair.to}`}
              >
                <FormatBadge format={pair.from} />
                <Icon name="arrow" size={13} className="pair-arrow" />
                <FormatBadge format={pair.to} />
                <span className="pair-action">
                  Convertir
                  <Icon name="arrow" size={12} />
                </span>
              </button>
            ))}
          </div>
        ) : (
          <div className="quick-empty">
            <p>
              No hay accesos rápidos predeterminados en la categoría «{filter}». Puedes explorar todas las combinaciones en el catálogo completo.
            </p>
            <button
              type="button"
              className="ghost"
              onClick={onNavigateToCatalog}
            >
              Ver catálogo completo
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
