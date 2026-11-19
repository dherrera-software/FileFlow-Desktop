import { Icon } from "../../components/Icon";

interface DropZoneProps {
  isDragging: boolean;
  isDisabled: boolean;
  maxMb?: number;
  onChooseFile: () => void;
}

export function DropZone({
  isDragging,
  isDisabled,
  maxMb = 25,
  onChooseFile,
}: DropZoneProps) {
  return (
    <button
      type="button"
      className={isDragging ? "dropzone dragging" : "dropzone"}
      onClick={onChooseFile}
      disabled={isDisabled}
      aria-label="Seleccionar o soltar archivo para convertir"
    >
      <span className="dropzone-icon">
        <Icon name="upload" size={28} />
      </span>
      <strong>Arrastra tu archivo aquí para comenzar</strong>
      <span>
        o elige uno desde tu equipo — documentos, imágenes, tablas, presentaciones,
        audio, vídeo o comprimidos de hasta {maxMb} MB.
      </span>
      <span className="fake-button">Explorar archivos</span>
    </button>
  );
}
