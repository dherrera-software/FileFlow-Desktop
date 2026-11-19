import { Icon } from "../Icon";
import type { Theme } from "../../types";

interface TopbarProps {
  activeSection: string;
  engineError: boolean;
  desktop: boolean;
  theme: Theme;
  onToggleTheme: () => void;
}

const SECTION_TITLES: Record<string, string> = {
  converter: "CONVERSOR",
  converters: "CATÁLOGO DE FORMATOS",
  privacy: "PRIVACIDAD Y ARQUITECTURA",
  faq: "AYUDA Y PREGUNTAS",
};

export function Topbar({
  activeSection,
  engineError,
  desktop,
  theme,
  onToggleTheme,
}: TopbarProps) {
  return (
    <header className="topbar">
      <div className="breadcrumb" aria-label="Ruta de navegación">
        <span>FILEFLOW</span>
        <span className="sep" aria-hidden="true">/</span>
        <strong>{SECTION_TITLES[activeSection] ?? "CONVERSOR"}</strong>
      </div>

      <div className="topbar-actions">
        <span
          className="engine-pill"
          data-state={engineError ? "error" : desktop ? "ok" : "idle"}
        >
          <span
            className="status-dot"
            data-state={engineError ? "error" : desktop ? "ok" : "idle"}
            aria-hidden="true"
          />
          {engineError ? "Sin conexión" : desktop ? "Seguro y privado" : "Vista previa"}
        </span>

        <button
          className="icon-button"
          type="button"
          onClick={onToggleTheme}
          aria-label={theme === "dark" ? "Cambiar a tema claro" : "Cambiar a tema oscuro"}
          title={theme === "dark" ? "Cambiar a tema claro (Slate White)" : "Cambiar a tema oscuro (Slate Cool)"}
        >
          <Icon name={theme === "dark" ? "sun" : "moon"} size={16} />
        </button>
      </div>
    </header>
  );
}
