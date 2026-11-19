export interface NormalizedError {
  code: string;
  message: string;
  suggestion?: string;
}

const ERROR_MAP: Record<string, { message: string; suggestion?: string }> = {
  FILE_NOT_FOUND: {
    message: "No se encontró el archivo seleccionado.",
    suggestion: "Verifica que el archivo no haya sido movido o eliminado.",
  },
  EMPTY_FILE: {
    message: "El archivo está vacío y no contiene datos para convertir.",
    suggestion: "Selecciona un archivo con contenido válido.",
  },
  FILE_TOO_LARGE: {
    message: "El archivo supera el límite máximo permitido de 25 MB.",
    suggestion: "Utiliza un archivo más pequeño o comprímelo antes de convertir.",
  },
  UNSUPPORTED_FORMAT: {
    message: "El formato del archivo no es compatible.",
    suggestion: "Consulta la lista de formatos compatibles en la aplicación.",
  },
  TARGET_NOT_AVAILABLE: {
    message: "La conversión solicitada no está disponible para este formato.",
    suggestion: "Elige uno de los formatos de destino sugeridos.",
  },
  CORRUPT_FILE: {
    message: "El archivo está dañado o no coincide con su extensión.",
    suggestion: "Asegúrate de que el archivo se abra correctamente en su programa original.",
  },
  PERMISSION_DENIED: {
    message: "No se tienen permisos de lectura o escritura en la ruta indicada.",
    suggestion: "Comprueba los permisos del sistema o elige otra carpeta de destino.",
  },
  CONVERSION_FAILED: {
    message: "Ocurrió un error durante el procesamiento del archivo.",
    suggestion: "Intenta convertir con otro formato destino o verifica el contenido.",
  },
};

export function normalizeError(cause: unknown): NormalizedError {
  if (!cause) {
    return {
      code: "UNKNOWN",
      message: "Ocurrió un error inesperado al procesar el archivo.",
    };
  }

  const rawMessage =
    typeof cause === "string"
      ? cause
      : cause instanceof Error
        ? cause.message
        : String(cause);

  const lower = rawMessage.toLowerCase();

  if (lower.includes("no existe") || lower.includes("not found")) {
    return { code: "FILE_NOT_FOUND", ...ERROR_MAP.FILE_NOT_FOUND };
  }
  if (lower.includes("vacío") || lower.includes("empty") || lower.includes("0 bytes")) {
    return { code: "EMPTY_FILE", ...ERROR_MAP.EMPTY_FILE };
  }
  if (lower.includes("límite") || lower.includes("25 mb") || lower.includes("too large")) {
    return { code: "FILE_TOO_LARGE", ...ERROR_MAP.FILE_TOO_LARGE };
  }
  if (lower.includes("no está disponible") || lower.includes("unsupported")) {
    return { code: "TARGET_NOT_AVAILABLE", ...ERROR_MAP.TARGET_NOT_AVAILABLE };
  }
  if (lower.includes("dañado") || lower.includes("corrupt") || lower.includes("no coincide")) {
    return { code: "CORRUPT_FILE", ...ERROR_MAP.CORRUPT_FILE };
  }
  if (lower.includes("permiso") || lower.includes("permission") || lower.includes("access denied")) {
    return { code: "PERMISSION_DENIED", ...ERROR_MAP.PERMISSION_DENIED };
  }

  return {
    code: "CONVERSION_FAILED",
    message: rawMessage.trim() || ERROR_MAP.CONVERSION_FAILED.message,
    suggestion: ERROR_MAP.CONVERSION_FAILED.suggestion,
  };
}
