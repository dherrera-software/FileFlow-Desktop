import { Icon } from "../../components/Icon";
import { FormatBadge } from "../../components/FormatBadge";
import type { ConversionMatrix } from "../../types";

interface ConversionCatalogProps {
  matrix: ConversionMatrix | null;
  loading: boolean;
  error: boolean;
  category: string;
  query: string;
  onQueryChange: (value: string) => void;
  showUnavailable: boolean;
  onToggleUnavailable: () => void;
  onSelectRoute: (from: string, to: string) => void;
}

interface VisibleCategory {
  key: string;
  label: string;
  formats: {
    format: string;
    extensions: string[];
    targets: { format: string; available: boolean; requires: string | null }[];
  }[];
}

function filterMatrix(
  matrix: ConversionMatrix,
  category: string,
  query: string,
  showUnavailable: boolean,
): VisibleCategory[] {
  const needle = query.trim().toUpperCase();
  return matrix.categories
    .filter((entry) => category === "Todos" || entry.label === category)
    .map((entry) => {
      const formats = entry.formats
        .map((format) => ({
          ...format,
          targets: format.targets.filter((target) => showUnavailable || target.available),
        }))
        .filter((format) => {
          if (format.targets.length === 0) return false;
          if (!needle) return true;
          if (format.format.includes(needle)) return true;
          if (format.extensions.some((extension) => extension.toUpperCase().includes(needle))) {
            return true;
          }
          return format.targets.some((target) => target.format.includes(needle));
        });
      return { key: entry.key, label: entry.label, formats };
    })
    .filter((entry) => entry.formats.length > 0);
}

export function ConversionCatalog({
  matrix,
  loading,
  error,
  category,
  query,
  onQueryChange,
  showUnavailable,
  onToggleUnavailable,
  onSelectRoute,
}: ConversionCatalogProps) {
  if (error) {
    return (
      <p className="catalog-empty" role="status">
        No se pudo leer el catálogo del motor local. Reinicia la app para volver a intentarlo.
      </p>
    );
  }

  if (!matrix) {
    return (
      <p className="catalog-empty" role="status">
        {loading
          ? "Leyendo la matriz de conversiones del motor local…"
          : "La matriz completa necesita el motor local. Ejecuta la app de escritorio con pnpm tauri dev para ver todas las rutas."}
      </p>
    );
  }

  const categories = filterMatrix(matrix, category, query, showUnavailable);
  const visibleRoutes = categories.reduce(
    (total, entry) =>
      total + entry.formats.reduce((sum, format) => sum + format.targets.length, 0),
    0,
  );
  const hiddenRoutes = matrix.declared - matrix.available;

  return (
    <div className="catalog">
      <div className="catalog-toolbar">
        <label className="catalog-search">
          <Icon name="search" size={14} />
          <input
            type="search"
            value={query}
            onChange={(event) => onQueryChange(event.target.value)}
            placeholder="Filtrar por formato: PDF, MP4, XLSX…"
            aria-label="Filtrar conversiones por formato"
          />
        </label>

        {hiddenRoutes > 0 && (
          <label className="catalog-toggle">
            <input
              type="checkbox"
              checked={showUnavailable}
              onChange={onToggleUnavailable}
            />
            <span>
              Mostrar {hiddenRoutes} rutas que requieren una herramienta externa
            </span>
          </label>
        )}

        <span className="catalog-count" aria-live="polite">
          {visibleRoutes} rutas visibles
        </span>
      </div>

      {categories.length === 0 ? (
        <div className="catalog-empty" role="status">
          <p>Ningún formato coincide con «{query.trim()}» en esta categoría.</p>
          <button type="button" className="ghost" onClick={() => onQueryChange("")}>
            Limpiar búsqueda
          </button>
        </div>
      ) : (
        <div className="catalog-groups">
          {categories.map((entry) => {
            const formatCount = entry.formats.length;
            const routeCount = entry.formats.reduce(
              (sum, format) => sum + format.targets.length,
              0,
            );
            return (
              <section
                key={entry.key}
                className="catalog-group"
                aria-labelledby={`catalog-${entry.key}`}
              >
                <div className="catalog-group-head">
                  <h3 id={`catalog-${entry.key}`}>{entry.label}</h3>
                  <span>
                    {formatCount} formatos · {routeCount} rutas
                  </span>
                </div>

                <div className="catalog-rows">
                  {entry.formats.map((format) => (
                    <div className="catalog-row" key={format.format}>
                      <div className="catalog-source">
                        <FormatBadge format={format.format} />
                        <span className="catalog-ext">
                          .{format.extensions.join(" · .")}
                        </span>
                      </div>

                      <div className="catalog-targets">
                        {format.targets.map((target) => (
                          <button
                            key={`${format.format}-${target.format}`}
                            type="button"
                            className="catalog-target"
                            data-available={target.available}
                            disabled={!target.available}
                            onClick={() => onSelectRoute(format.format, target.format)}
                            title={
                              target.available
                                ? `Convertir ${format.format} a ${target.format}`
                                : `Requiere ${target.requires ?? "una herramienta local"}`
                            }
                            aria-label={
                              target.available
                                ? `Convertir ${format.format} a ${target.format}`
                                : `${format.format} a ${target.format}: no disponible, requiere ${target.requires ?? "una herramienta local"}`
                            }
                          >
                            <FormatBadge format={target.format} />
                            {!target.available && (
                              <span className="catalog-requirement">
                                {target.requires ?? "Herramienta local"}
                              </span>
                            )}
                          </button>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </section>
            );
          })}
        </div>
      )}

      <p className="micro-caption">
        {matrix.available} RUTAS DISPONIBLES EN ESTE EQUIPO · {matrix.declared} DECLARADAS
        {showUnavailable && hiddenRoutes > 0
          ? " · LAS RUTAS CON CANDADO REQUIEREN UNA HERRAMIENTA LOCAL"
          : ""}
      </p>
    </div>
  );
}
