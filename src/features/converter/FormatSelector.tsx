interface FormatSelectorProps {
  targets: string[];
  selectedTarget: string;
  onSelectTarget: (target: string) => void;
  disabled?: boolean;
}

export function FormatSelector({
  targets,
  selectedTarget,
  onSelectTarget,
  disabled = false,
}: FormatSelectorProps) {
  return (
    <div className="targets">
      <span className="field-label" id="target-format-label">
        Convertir a
      </span>
      <div
        className="chips"
        role="group"
        aria-labelledby="target-format-label"
      >
        {targets.map((option) => {
          const isSelected = option === selectedTarget;
          return (
            <button
              key={option}
              type="button"
              className={isSelected ? "chip active" : "chip"}
              onClick={() => onSelectTarget(option)}
              aria-pressed={isSelected}
              disabled={disabled}
            >
              {option}
            </button>
          );
        })}
      </div>
    </div>
  );
}
