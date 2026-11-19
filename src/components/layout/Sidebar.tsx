import { Icon, type IconName } from "../Icon";

export interface NavSection {
  id: string;
  label: string;
  icon: IconName;
}

interface SidebarProps {
  sections: NavSection[];
  activeSection: string;
  onSelectSection: (id: string) => void;
  engineError: boolean;
  desktop: boolean;
  engineVersion?: string;
}

export function Sidebar({
  sections,
  activeSection,
  onSelectSection,
  engineError,
  desktop,
  engineVersion = "0.1.0",
}: SidebarProps) {
  return (
    <aside className="sidebar" aria-label="Barra lateral">
      <div className="brand">
        <span className="brand-mark" aria-hidden="true">
          <Icon name="file" size={19} />
        </span>
        <span className="brand-text">
          <strong>FileFlow</strong>
          <small>Desktop</small>
        </span>
      </div>

      <button
        className="new-file"
        type="button"
        onClick={() => onSelectSection("converter")}
      >
        <Icon name="upload" size={16} />
        Nuevo archivo
      </button>

      <p className="sidebar-label">Espacios</p>
      <nav className="nav" aria-label="Secciones de la aplicación">
        {sections.map((section) => (
          <button
            key={section.id}
            type="button"
            className={activeSection === section.id ? "nav-item active" : "nav-item"}
            onClick={() => onSelectSection(section.id)}
          >
            <Icon name={section.icon} size={17} />
            {section.label}
          </button>
        ))}
      </nav>

      <div className="sidebar-status">
        <span
          className="status-dot"
          data-state={engineError ? "error" : desktop ? "ok" : "idle"}
          aria-hidden="true"
        />
        <div>
          <strong>
            {engineError
              ? "No disponible"
              : desktop
                ? "Listo para convertir"
                : "Modo vista previa"}
          </strong>
          <small>{desktop ? "100% privado en tu equipo" : "Inicia con pnpm tauri dev"}</small>
        </div>
        <span className="version">v{engineVersion}</span>
      </div>
    </aside>
  );
}
