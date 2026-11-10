/* FileFlow clone — standalone client-side app
   Reimplements the interactions and local conversions of the original site. */
(() => {
  'use strict';

  const THEME_KEY = 'fileflow-theme';
  const MAX_MB = 25;
  const MAX_BYTES = MAX_MB * 1024 * 1024;

  const FORMATS = {
    TXT: { category: 'Documentos', badge: 'bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300', description: 'Texto plano' },
    HTML: { category: 'Documentos', badge: 'bg-indigo-50 text-indigo-700 dark:bg-indigo-950/70 dark:text-indigo-300', description: 'Página web HTML' },
    MD: { category: 'Documentos', badge: 'bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-200', description: 'Documento Markdown' },
    JPG: { category: 'Imágenes', badge: 'bg-amber-50 text-amber-700 dark:bg-amber-950/70 dark:text-amber-300', description: 'Imagen JPG' },
    JPEG: { category: 'Imágenes', badge: 'bg-amber-50 text-amber-700 dark:bg-amber-950/70 dark:text-amber-300', description: 'Imagen JPEG' },
    PNG: { category: 'Imágenes', badge: 'bg-teal-50 text-teal-700 dark:bg-teal-950/70 dark:text-teal-300', description: 'Imagen PNG' },
    CSV: { category: 'Hojas de cálculo', badge: 'bg-lime-50 text-lime-700 dark:bg-lime-950/70 dark:text-lime-300', description: 'Hoja de cálculo CSV' },
    TSV: { category: 'Hojas de cálculo', badge: 'bg-lime-50 text-lime-700 dark:bg-lime-950/70 dark:text-lime-300', description: 'Datos tabulados TSV' },
    JSON: { category: 'Hojas de cálculo', badge: 'bg-yellow-50 text-yellow-800 dark:bg-yellow-950/70 dark:text-yellow-300', description: 'Estructura de datos JSON' },
    XML: { category: 'Hojas de cálculo', badge: 'bg-orange-50 text-orange-800 dark:bg-orange-950/70 dark:text-orange-300', description: 'Documento estructurado XML' },
    PDF: { category: 'Documentos', badge: 'bg-red-50 text-red-700 dark:bg-red-950/70 dark:text-red-300', description: 'Documento PDF' },
  };

  const TARGETS = {
    TXT: ['PDF', 'HTML', 'MD'],
    HTML: ['MD'],
    MD: ['HTML'],
    JPG: ['PDF'],
    JPEG: ['PDF'],
    PNG: ['PDF'],
    CSV: ['JSON', 'XML'],
    TSV: ['CSV'],
    JSON: ['CSV'],
    XML: ['CSV'],
  };

  const EXTENSIONS = {
    txt: 'TXT',
    html: 'HTML',
    htm: 'HTML',
    md: 'MD',
    markdown: 'MD',
    jpg: 'JPG',
    jpeg: 'JPEG',
    png: 'PNG',
    csv: 'CSV',
    tsv: 'TSV',
    json: 'JSON',
    xml: 'XML',
  };

  const SECTION_LABELS = {
    convert: 'Conversor',
    converters: 'Conversiones',
    'how-it-works': 'Cómo funciona',
    privacy: 'Privacidad',
    faq: 'FAQ',
  };

  const ICONS = {
    arrowRight: '<path d="M5 12h14"></path><path d="m12 5 7 7-7 7"></path>',
    sun: '<circle cx="12" cy="12" r="4"></circle><path d="M12 2v2"></path><path d="M12 20v2"></path><path d="m4.93 4.93 1.41 1.41"></path><path d="m17.66 17.66 1.41 1.41"></path><path d="M2 12h2"></path><path d="M20 12h2"></path><path d="m6.34 17.66-1.41 1.41"></path><path d="m19.07 4.93-1.41 1.41"></path>',
    moon: '<path d="M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401"></path>',
    x: '<path d="M18 6 6 18"></path><path d="m6 6 12 12"></path>',
    fileText: '<path d="M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z"></path><path d="M14 2v5a1 1 0 0 0 1 1h5"></path><path d="M10 9H8"></path><path d="M16 13H8"></path><path d="M16 17H8"></path>',
    download: '<path d="M12 15V3"></path><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><path d="m7 10 5 5 5-5"></path>',
    loader: '<path d="M21 12a9 9 0 1 1-6.219-8.56"></path>',
    check: '<circle cx="12" cy="12" r="10"></circle><path d="m16 9-5.5 5.5L8 12"></path>',
    alert: '<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"></path><path d="M12 9v4"></path><path d="M12 17h.01"></path>',
    info: '<circle cx="12" cy="12" r="10"></circle><path d="M12 16v-4"></path><path d="M12 8h.01"></path>',
  };

  const icon = (paths, className) =>
    `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="${className}" aria-hidden="true">${paths}</svg>`;

  const $ = (selector, root = document) => root.querySelector(selector);
  const $$ = (selector, root = document) => Array.from(root.querySelectorAll(selector));
  const esc = (value) =>
    String(value)
      .replaceAll('&', '&amp;')
      .replaceAll('<', '&lt;')
      .replaceAll('>', '&gt;')
      .replaceAll('"', '&quot;')
      .replaceAll("'", '&#39;');

  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

  function formatBytes(bytes) {
    if (!Number.isFinite(bytes) || bytes <= 0) return '0 KB';
    const units = ['B', 'KB', 'MB', 'GB'];
    const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    const value = bytes / 1024 ** index;
    const digits = index === 0 || value >= 10 ? 0 : 1;
    return `${value.toFixed(digits)} ${units[index]}`;
  }

  const formatBadge = (format, size = 'xs', extra = '') => {
    const info = FORMATS[format] || { badge: 'bg-surface-sunken text-ink-muted' };
    const sizing = size === 'sm' ? 'px-2.5 py-1 text-[10.5px]' : 'px-2 py-0.5 text-[10px]';
    return `<span class="inline-flex items-center justify-center rounded-full border border-transparent font-mono font-medium tracking-[0.08em] text-center transition-colors ${sizing} ${info.badge} ${extra}">${esc(format)}</span>`;
  };

  const dom = {
    convertBody: document.getElementById('convert-body'),
    fileInput: document.getElementById('file-input'),
    fileStatus: document.getElementById('file-status'),
    fileStage: document.getElementById('file-stage'),
    fileTitle: document.getElementById('file-title'),
    formatList: document.getElementById('format-list'),
    pairList: document.getElementById('pair-list'),
    breadcrumb: document.getElementById('breadcrumb-current'),
    menu: document.getElementById('mobile-menu'),
    menuTrigger: $('[data-menu-open]'),
    themeToggle: $('[data-theme-toggle]'),
    toastRegion: document.getElementById('toast-region'),
  };

  const idleTemplate = dom.convertBody ? dom.convertBody.innerHTML : '';

  const state = {
    file: null,
    format: null,
    target: null,
    targetOptions: [],
    status: 'idle',
    error: null,
    previewUrl: null,
    imageSize: null,
    progress: 0,
    stage: '',
    conversionError: null,
    notice: false,
    pendingPair: null,
  };

  // ---------------------------------------------------------------- Theme
  function getTheme() {
    try {
      return localStorage.getItem(THEME_KEY) === 'light' ? 'light' : 'dark';
    } catch {
      return 'dark';
    }
  }

  function updateThemeButton(isDark) {
    if (!dom.themeToggle) return;
    dom.themeToggle.innerHTML = icon(isDark ? ICONS.sun : ICONS.moon, 'size-4');
    dom.themeToggle.setAttribute('aria-label', isDark ? 'Cambiar a modo claro' : 'Cambiar a modo oscuro');
    dom.themeToggle.title = isDark ? 'Modo claro' : 'Modo oscuro';
  }

  function applyTheme(theme, persist = true) {
    const isDark = theme !== 'light';
    const background = isDark ? '#08090a' : '#f5f5f1';
    document.documentElement.classList.toggle('dark', isDark);
    document.documentElement.style.backgroundColor = background;
    document.querySelector('meta[name="theme-color"]')?.setAttribute('content', background);
    updateThemeButton(isDark);
    if (persist) {
      try {
        localStorage.setItem(THEME_KEY, isDark ? 'dark' : 'light');
      } catch {
        /* storage unavailable */
      }
    }
  }

  // ---------------------------------------------------------------- Status chip
  const STATUS_TONES = {
    muted: { pill: 'border-line bg-surface-raised text-ink-subtle', dot: 'bg-ink-faint' },
    accent: { pill: 'border-accent-line bg-accent-soft text-accent', dot: 'bg-accent' },
    danger: { pill: 'border-danger/40 bg-danger/10 text-danger', dot: 'bg-danger' },
  };

  function setStatus(tone, label) {
    if (!dom.fileStatus) return;
    const styles = STATUS_TONES[tone] || STATUS_TONES.muted;
    dom.fileStatus.className = `inline-flex shrink-0 items-center gap-1.5 rounded-full border px-2.5 py-1 font-mono text-[9px] uppercase tracking-[0.12em] ${styles.pill}`;
    dom.fileStatus.innerHTML = `<span class="size-1.5 rounded-full ${styles.dot}" aria-hidden="true"></span>${esc(label)}`;
  }

  // ---------------------------------------------------------------- Announce
  function announce(message) {
    const region = document.getElementById('drop-announcer');
    if (region) region.textContent = message;
  }

  // ---------------------------------------------------------------- Idle view
  function idleMarkupWithPending() {
    return idleTemplate;
  }

  function renderIdle() {
    if (!dom.convertBody) return;
    state.status = 'idle';
    state.file = null;
    state.format = null;
    state.target = null;
    state.targetOptions = [];
    state.error = null;
    state.conversionError = null;
    state.notice = false;
    state.progress = 0;
    state.stage = '';
    dom.convertBody.innerHTML = idleMarkupWithPending();
    const selectButton = $('#dropzone button');
    if (selectButton) selectButton.setAttribute('data-action', 'open-file');
    if (state.pendingPair) addPendingNotice();
    setStatus('muted', 'Listo');
  }

  function addPendingNotice() {
    const dropzone = document.getElementById('dropzone');
    if (!dropzone || !state.pendingPair) return;
    const note = document.createElement('p');
    note.className = 'mt-3 inline-flex items-center gap-1.5 font-mono text-[10px] uppercase tracking-[0.08em] text-accent';
    note.innerHTML = `${icon(ICONS.arrowRight, 'size-3')}El próximo archivo ${esc(state.pendingPair.source)} se convertirá a ${esc(state.pendingPair.target)}`;
    dropzone.appendChild(note);
  }

  // ---------------------------------------------------------------- Error view
  function renderError() {
    if (!dom.convertBody) return;
    state.status = 'error';
    const error = state.error || { fileName: 'Archivo', message: 'No se pudo leer el archivo.' };
    dom.convertBody.innerHTML = `
      <div aria-live="polite" class="sr-only" id="drop-announcer"></div>
      <div role="alert" class="rounded-xl border border-dashed border-danger/40 bg-danger/5 px-5 py-7 text-center">
        ${icon(ICONS.alert, 'mx-auto size-5 text-danger')}
        <p class="mt-2.5 text-[14px] font-medium text-ink">No se puede convertir este archivo</p>
        <p class="mt-1 truncate text-[12.5px] font-medium text-ink-muted">${esc(error.fileName)}</p>
        <p class="mx-auto mt-1 max-w-[46ch] text-[12px] leading-[1.6] text-ink-subtle">${esc(error.message)}</p>
        <div class="mt-4 flex flex-wrap items-center justify-center gap-2">
          <button type="button" data-action="open-file" class="inline-flex items-center justify-center gap-2 rounded-lg font-medium transition-[color,background-color,border-color,box-shadow,transform] duration-150 disabled:cursor-not-allowed disabled:opacity-50 active:scale-[0.98] border border-line bg-surface-raised text-ink hover:border-line-strong hover:bg-surface-sunken px-3 py-1.5 text-[12px]">Elegir otro archivo</button>
          <button type="button" data-action="dismiss-error" class="inline-flex items-center justify-center gap-2 rounded-lg font-medium transition-[color,background-color,border-color,box-shadow,transform] duration-150 disabled:cursor-not-allowed disabled:opacity-50 active:scale-[0.98] text-ink-subtle hover:bg-surface-sunken hover:text-ink px-3 py-1.5 text-[12px]">Descartar</button>
        </div>
      </div>`;
    setStatus('danger', 'Revisar archivo');
    announce(`No se puede convertir ${error.fileName}. ${error.message}`);
  }

  // ---------------------------------------------------------------- Selected view
  function previewMarkup() {
    const file = state.file;
    if (!file || !state.format) return '';
    const image = state.imageSize
      ? `<img src="${state.previewUrl}" alt="Vista previa de ${esc(file.name)}" width="${state.imageSize.width}" height="${state.imageSize.height}" class="max-h-full max-w-full object-contain" loading="lazy">`
      : '';
    const document = state.imageSize
      ? ''
      : `<div class="flex flex-col items-center gap-2 p-4 text-ink-subtle">${icon(ICONS.fileText, 'size-8 text-accent')}<span class="text-sm font-medium uppercase tracking-wide">${esc(state.format)}</span></div>`;
    return `<div class="relative overflow-hidden rounded-xl border border-line bg-surface-raised mx-3 mt-3 mb-0">
        <div class="relative flex aspect-[4/3] min-h-[120px] items-center justify-center overflow-hidden bg-surface-sunken">${image}${document}</div>
        <div class="border-t border-line bg-surface/60 p-3">
          <p class="truncate text-sm font-medium text-ink">${esc(file.name)}</p>
          <div class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[10.5px] tabular-nums text-ink-subtle">
            <span>${formatBytes(file.size)}</span>
            ${state.imageSize ? `<span>${state.imageSize.width}×${state.imageSize.height}</span>` : ''}
          </div>
        </div>
      </div>`;
  }

  function targetMarkup() {
    const format = state.format;
    if (!format) return '';
    const converting = state.status === 'converting';
    return `<fieldset class="min-w-0 border-t border-line px-4 pb-4 pt-3.5 text-left sm:px-5">
        <legend class="sr-only">Elige un formato de destino para ${esc(format)}</legend>
        <div class="flex items-baseline justify-between gap-3">
          <p aria-hidden="true" class="font-mono text-[9px] uppercase tracking-[0.16em] text-ink-faint">Convertir a</p>
          <p class="font-mono text-[9.5px] uppercase tracking-[0.12em] text-ink-subtle">detectado ${esc(format)}</p>
        </div>
        <div class="mt-2 flex flex-wrap gap-1.5">
          ${state.targetOptions
            .map(
              (option) => `<label class="cursor-pointer rounded-full border border-line bg-surface-sunken px-3.5 py-1.5 font-mono text-[10.5px] font-medium uppercase tracking-[0.08em] text-ink-muted transition-colors hover:border-line-strong hover:text-ink has-[:checked]:border-accent-line has-[:checked]:bg-accent-soft has-[:checked]:text-accent has-[:focus-visible]:outline-2 has-[:focus-visible]:outline-offset-2 has-[:focus-visible]:outline-accent${converting ? ' pointer-events-none opacity-60' : ''}">
                <input type="radio" name="target-format" value="${esc(option)}" class="sr-only" ${state.target === option ? 'checked' : ''} ${converting ? 'disabled' : ''}>${esc(option)}
              </label>`
            )
            .join('')}
        </div>
        ${converting ? progressMarkup() : ''}
        <button type="button" data-action="convert" class="inline-flex items-center justify-center gap-2 rounded-lg font-medium transition-[color,background-color,border-color,box-shadow,transform] duration-150 disabled:cursor-not-allowed disabled:opacity-50 active:scale-[0.98] border border-accent-fill bg-accent-fill text-accent-on shadow-primary hover:brightness-105 px-4 py-2 text-[12.5px] w-full ${converting ? 'mt-2' : 'mt-3'}" ${!state.target || converting ? 'disabled' : ''}>
          ${converting ? `${icon(ICONS.loader, 'size-4 animate-spin')}<span data-progress-button-label>${Math.round(state.progress)}% completado</span>` : 'Convertir archivo'}
        </button>
      </fieldset>`;
  }

  function progressMarkup() {
    return `<div class="mt-3 space-y-1.5">
        <div class="h-1.5 w-full overflow-hidden rounded-full bg-surface-sunken">
          <div data-progress-bar class="h-full rounded-full bg-accent transition-all duration-100 ease-linear" style="width: ${Math.max(state.progress, 2)}%" role="progressbar" aria-valuenow="${Math.round(state.progress)}" aria-valuemin="0" aria-valuemax="100" aria-label="Progreso de conversión"></div>
        </div>
        <div class="flex items-center justify-between px-0.5 font-mono text-[10px] tabular-nums text-ink-subtle">
          <span data-progress-stage class="truncate">${esc(state.stage || 'Procesando archivo...')}</span>
          <span data-progress-percent class="ml-2 shrink-0 font-medium text-accent">${Math.round(state.progress)}%</span>
        </div>
      </div>`;
  }

  function conversionErrorMarkup() {
    if (!state.conversionError) return '';
    return `<div role="alert" class="mx-4 mb-4 flex flex-col gap-2.5 rounded-lg border border-danger/40 bg-danger/10 px-3.5 py-3 text-left">
        <div class="flex items-start gap-2.5">
          ${icon(ICONS.alert, 'mt-0.5 size-4 shrink-0 text-danger')}
          <div class="min-w-0 flex-1">
            <p class="text-[12.5px] font-medium text-danger">Error en la conversión</p>
            <p class="mt-0.5 text-[12px] leading-relaxed text-ink-muted">${esc(state.conversionError.message)}</p>
          </div>
          <button type="button" data-action="dismiss-conversion-error" aria-label="Cerrar aviso de error" class="shrink-0 rounded-md p-1 text-ink-subtle transition-colors hover:bg-surface-sunken hover:text-ink">${icon(ICONS.x, 'size-3.5')}</button>
        </div>
      </div>`;
  }

  function noticeMarkup() {
    if (!state.notice) return '';
    return `<div role="status" aria-live="polite" class="mx-4 mb-4 flex items-start gap-2.5 rounded-lg border border-line bg-surface px-3.5 py-3 text-left">
        ${icon(ICONS.info, 'mt-0.5 size-4 shrink-0 text-accent')}
        <div class="min-w-0 flex-1">
          <p class="text-[12.5px] font-medium text-ink">Conversión procesada</p>
          <p class="mt-0.5 text-[12px] leading-relaxed text-ink-muted">El archivo ha sido procesado por el motor de conversión y tu descarga debería haber iniciado.</p>
        </div>
        <button type="button" data-action="dismiss-notice" aria-label="Cerrar aviso de conversión" class="shrink-0 rounded-md p-1 text-ink-faint transition-colors hover:bg-surface-sunken hover:text-ink">${icon(ICONS.x, 'size-3.5')}</button>
      </div>`;
  }

  function renderSelected() {
    if (!dom.convertBody || !state.file || !state.format) return;
    const file = state.file;
    const converting = state.status === 'converting';
    dom.convertBody.innerHTML = `
      <div aria-live="polite" class="sr-only" id="drop-announcer"></div>
      <div class="overflow-hidden rounded-xl border border-line bg-surface-raised">
        <div class="flex items-start gap-3 border-b border-line px-4 py-3.5 text-left">
          ${formatBadge(state.format, 'sm', 'mt-0.5 shrink-0')}
          <div class="min-w-0 flex-1">
            <p class="truncate text-[13.5px] font-medium text-ink">${esc(file.name)}</p>
            <p class="mt-0.5 text-[11.5px] tabular-nums text-ink-subtle">${formatBytes(file.size)}<span aria-hidden="true"> · </span>${esc(FORMATS[state.format]?.description || state.format)}</p>
          </div>
          <button type="button" data-action="remove-file" aria-label="Eliminar ${esc(file.name)}" class="shrink-0 rounded-md p-1 text-ink-subtle transition-colors hover:bg-surface-sunken hover:text-ink">${icon(ICONS.x, 'size-3.5')}</button>
        </div>
        ${previewMarkup()}
        ${targetMarkup()}
        ${noticeMarkup()}
        ${conversionErrorMarkup()}
      </div>`;
    setStatus(converting ? 'accent' : 'accent', converting ? 'Convirtiendo' : 'Archivo cargado');
    if (converting) updateProgressUI();
  }

  function updateProgressUI() {
    const bar = $('[data-progress-bar]');
    if (bar) {
      bar.style.width = `${Math.max(state.progress, 2)}%`;
      bar.setAttribute('aria-valuenow', String(Math.round(state.progress)));
    }
    const percent = $('[data-progress-percent]');
    if (percent) percent.textContent = `${Math.round(state.progress)}%`;
    const stage = $('[data-progress-stage]');
    if (stage) stage.textContent = state.stage || 'Procesando archivo...';
    const buttonLabel = $('[data-progress-button-label]');
    if (buttonLabel) buttonLabel.textContent = `${Math.round(state.progress)}% completado`;
  }

  function startProgressTicker() {
    return window.setInterval(() => {
      if (state.status !== 'converting') return;
      const step = state.progress < 30 ? 7 : state.progress < 70 ? 4 : state.progress < 90 ? 1.5 : 0;
      state.progress = Math.min(90, state.progress + step);
      updateProgressUI();
    }, 120);
  }

  // ---------------------------------------------------------------- File handling
  function cleanupPreview() {
    if (state.previewUrl) {
      URL.revokeObjectURL(state.previewUrl);
      state.previewUrl = null;
    }
    state.imageSize = null;
  }

  function validateFile(file) {
    const extension = file.name.includes('.') ? file.name.split('.').pop().toLowerCase() : '';
    const format = EXTENSIONS[extension];
    if (!format) {
      return {
        ok: false,
        message: `${extension ? `.${extension}` : 'Este tipo de archivo'} no es un formato compatible. FileFlow admite documentos de texto (TXT, HTML, MD), imágenes (JPG, PNG) y datos (CSV, TSV, JSON, XML).`,
      };
    }
    if (file.size > MAX_BYTES) {
      return { ok: false, message: `Este archivo pesa ${formatBytes(file.size)}. El límite es de ${MAX_MB} MB por archivo.` };
    }
    if (file.size === 0) {
      return { ok: false, message: 'Este archivo está vacío; no hay nada para convertir.' };
    }
    return { ok: true, format };
  }

  function getImageSize(src) {
    return new Promise((resolve, reject) => {
      const image = new Image();
      image.onload = () => resolve({ width: image.naturalWidth, height: image.naturalHeight });
      image.onerror = reject;
      image.src = src;
    });
  }

  async function handleFile(file) {
    const validation = validateFile(file);
    if (!validation.ok) {
      cleanupPreview();
      state.status = 'error';
      state.file = null;
      state.format = null;
      state.target = null;
      state.targetOptions = [];
      state.error = { fileName: file.name, message: validation.message };
      state.conversionError = null;
      state.notice = false;
      renderError();
      return;
    }

    cleanupPreview();
    state.file = file;
    state.format = validation.format;
    state.targetOptions = TARGETS[validation.format] || [];
    state.target = null;
    state.error = null;
    state.conversionError = null;
    state.notice = false;
    state.status = 'selected';

    if (state.pendingPair && state.pendingPair.source === validation.format && state.targetOptions.includes(state.pendingPair.target)) {
      state.target = state.pendingPair.target;
      state.pendingPair = null;
    }
    if (!state.target) state.target = state.targetOptions[0] || null;

    if (['JPG', 'JPEG', 'PNG'].includes(validation.format)) {
      state.previewUrl = URL.createObjectURL(file);
      try {
        state.imageSize = await getImageSize(state.previewUrl);
      } catch {
        state.imageSize = null;
      }
    }

    renderSelected();
    announce(`Archivo ${file.name} cargado. Formato ${validation.format}.`);
  }

  function openFilePicker() {
    closeMenu();
    dom.fileInput?.click();
  }

  function removeFile() {
    cleanupPreview();
    state.pendingPair = null;
    renderIdle();
    announce('Archivo eliminado.');
  }

  function setDragging(dragging) {
    const dropzone = document.getElementById('dropzone');
    if (!dropzone) return;
    if (dragging) {
      dropzone.classList.remove('border-line-strong', 'bg-surface-raised/60', 'hover:border-line-strong');
      dropzone.classList.add('border-accent', 'bg-accent-soft');
      const iconBox = dropzone.querySelector('span.flex.size-11');
      if (iconBox) {
        iconBox.classList.remove('border-line', 'bg-surface', 'text-accent');
        iconBox.classList.add('border-accent-line', 'bg-accent-soft', 'text-accent');
        iconBox.innerHTML = icon(ICONS.download, 'size-5 animate-drop-bob');
      }
      const title = dropzone.querySelector('p.mt-4');
      if (title) title.textContent = 'Suelta el archivo para seleccionarlo';
      const subtitle = document.getElementById('drop-instructions');
      if (subtitle) {
        subtitle.textContent = 'Suelta para verificar el formato y ver las opciones de conversión.';
        subtitle.classList.remove('text-ink-subtle');
        subtitle.classList.add('text-accent');
      }
      setStatus('accent', 'Suelta para cargar');
    } else if (state.status === 'idle') {
      renderIdle();
    }
  }

  // ---------------------------------------------------------------- Conversion UI
  async function convertCurrent() {
    if (state.status === 'converting' || !state.file || !state.target) return;
    const file = state.file;
    const source = state.format;
    const target = state.target;
    state.status = 'converting';
    state.progress = 0;
    state.stage = 'Iniciando...';
    state.conversionError = null;
    state.notice = false;
    renderSelected();
    const ticker = startProgressTicker();
    const started = Date.now();

    try {
      const result = await convertFile(file, source, target);
      const remaining = 900 - (Date.now() - started);
      if (remaining > 0) await sleep(remaining);
      window.clearInterval(ticker);
      state.progress = 100;
      state.stage = '¡Completado!';
      updateProgressUI();
      triggerDownload(result.blob, result.filename);
      state.status = 'selected';
      state.notice = true;
      renderSelected();
      showToast('success', 'Conversión completada', `${result.filename} se ha descargado.`);
      announce(`Conversión completada. ${result.filename} descargado.`);
    } catch (error) {
      window.clearInterval(ticker);
      const message = error instanceof Error ? error.message : 'No se pudo convertir el archivo.';
      state.status = 'selected';
      state.conversionError = { message, retryable: false };
      renderSelected();
      showToast('error', 'No se pudo convertir el archivo', message);
      announce(`Error en la conversión. ${message}`);
    }
  }

  function triggerDownload(blob, filename) {
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    link.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  function baseName(filename) {
    return filename.replace(/\.[^/.]+$/, '');
  }

  async function convertFile(file, source, target) {
    const filename = `${baseName(file.name)}.${target.toLowerCase()}`;
    if (source === 'TXT') {
      const text = stripBom(await file.text());
      if (target === 'MD') return { blob: new Blob([txtToMarkdown(text)], { type: 'text/markdown;charset=utf-8' }), filename };
      if (target === 'HTML') return { blob: new Blob([txtToHtml(text, baseName(file.name))], { type: 'text/html;charset=utf-8' }), filename };
      if (target === 'PDF') return { blob: new Blob([textToPdf(text)], { type: 'application/pdf' }), filename };
    }
    if (source === 'HTML' && target === 'MD') {
      const html = await file.text();
      return { blob: new Blob([htmlToMarkdown(html)], { type: 'text/markdown;charset=utf-8' }), filename };
    }
    if (source === 'MD' && target === 'HTML') {
      const markdown = stripBom(await file.text());
      return { blob: new Blob([markdownToHtml(markdown, baseName(file.name))], { type: 'text/html;charset=utf-8' }), filename };
    }
    if (['JPG', 'JPEG', 'PNG'].includes(source) && target === 'PDF') {
      return { blob: await imageToPdf(file), filename };
    }
    if (source === 'CSV' && target === 'JSON') {
      const text = stripBom(await file.text());
      return { blob: new Blob([csvToJson(text)], { type: 'application/json;charset=utf-8' }), filename };
    }
    if (source === 'CSV' && target === 'XML') {
      const text = stripBom(await file.text());
      return { blob: new Blob([csvToXml(text)], { type: 'application/xml;charset=utf-8' }), filename };
    }
    if (source === 'TSV' && target === 'CSV') {
      const text = stripBom(await file.text());
      return { blob: new Blob([tsvToCsv(text)], { type: 'text/csv;charset=utf-8' }), filename };
    }
    if (source === 'JSON' && target === 'CSV') {
      const text = stripBom(await file.text());
      return { blob: new Blob([jsonToCsv(text)], { type: 'text/csv;charset=utf-8' }), filename };
    }
    if (source === 'XML' && target === 'CSV') {
      const text = stripBom(await file.text());
      return { blob: new Blob([xmlToCsv(text)], { type: 'text/csv;charset=utf-8' }), filename };
    }
    throw new Error(`La conversión de ${source} a ${target} no está disponible.`);
  }

  function stripBom(text) {
    return text.replace(/^\uFEFF/, '');
  }

  // ---------------------------------------------------------------- Text / Markdown / HTML
  function txtToMarkdown(text) {
    const normalized = text.replace(/\r\n?/g, '\n').trim();
    if (!normalized) return '\n';
    return `${normalized.split(/\n{2,}/).map((paragraph) => paragraph.trim()).join('\n\n')}\n`;
  }

  function txtToHtml(text, title) {
    const paragraphs = text
      .replace(/\r\n?/g, '\n')
      .split(/\n{2,}/)
      .map((paragraph) => `<p>${esc(paragraph).replaceAll('\n', '<br>')}</p>`)
      .join('\n');
    return `<!doctype html>
<html lang="es">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(title)}</title>
<style>
body{max-width:46rem;margin:3rem auto;padding:0 1.25rem;font-family:system-ui,-apple-system,"Segoe UI",sans-serif;line-height:1.65;color:#1a1d16;background:#fff}
p{margin:0 0 1rem}
</style>
</head>
<body>
${paragraphs || '<p></p>'}
</body>
</html>`;
  }

  function inlineMarkdown(value) {
    const code = [];
    let text = esc(value);
    text = text.replace(/`([^`]+)`/g, (_, content) => {
      const index = code.push(content) - 1;
      return `@@IC${index}@@`;
    });
    text = text.replace(/!\[([^\]]*)\]\(([^)\s]+)(?:\s+"([^"]*)")?\)/g, (_, alt, url, title) => {
      return `<img src="${url}" alt="${alt}"${title ? ` title="${title}"` : ''}>`;
    });
    text = text.replace(/\[([^\]]+)\]\(([^)\s]+)(?:\s+"([^"]*)")?\)/g, (_, label, url, title) => {
      return `<a href="${url}"${title ? ` title="${title}"` : ''}>${label}</a>`;
    });
    text = text.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
    text = text.replace(/__([^_]+)__/g, '<strong>$1</strong>');
    text = text.replace(/(^|[^*])\*([^*]+)\*/g, '$1<em>$2</em>');
    text = text.replace(/(^|[^_])_([^_]+)_/g, '$1<em>$2</em>');
    text = text.replace(/~~([^~]+)~~/g, '<del>$1</del>');
    text = text.replace(/@@IC(\d+)@@/g, (_, index) => `<code>${code[Number(index)]}</code>`);
    return text;
  }

  function markdownToHtml(markdown, title) {
    let text = markdown.replace(/\r\n?/g, '\n');
    const codeBlocks = [];
    text = text.replace(/```([^\n`]*)\n([\s\S]*?)```/g, (_, language, code) => {
      const index = codeBlocks.push({ language: language.trim(), code }) - 1;
      return `\n\n@@CODE${index}@@\n\n`;
    });

    const lines = text.split('\n');
    const output = [];
    let paragraph = [];
    let list = null;

    const flushParagraph = () => {
      if (!paragraph.length) return;
      output.push(`<p>${inlineMarkdown(paragraph.join(' '))}</p>`);
      paragraph = [];
    };
    const closeList = () => {
      if (!list) return;
      output.push(`</${list}>`);
      list = null;
    };

    for (let index = 0; index < lines.length; index += 1) {
      const line = lines[index];
      const codeMatch = line.trim().match(/^@@CODE(\d+)@@$/);
      if (codeMatch) {
        flushParagraph();
        closeList();
        const block = codeBlocks[Number(codeMatch[1])];
        output.push(`<pre><code${block.language ? ` class="language-${esc(block.language)}"` : ''}>${esc(block.code)}</code></pre>`);
        continue;
      }
      if (!line.trim()) {
        flushParagraph();
        closeList();
        continue;
      }
      const heading = line.match(/^(#{1,6})\s+(.*)$/);
      if (heading) {
        flushParagraph();
        closeList();
        const level = heading[1].length;
        output.push(`<h${level}>${inlineMarkdown(heading[2].trim())}</h${level}>`);
        continue;
      }
      if (/^(?:\*\s*){3,}$/.test(line.trim()) || /^(?:-\s*){3,}$/.test(line.trim()) || /^(?:_\s*){3,}$/.test(line.trim())) {
        flushParagraph();
        closeList();
        output.push('<hr>');
        continue;
      }
      if (/^>\s?/.test(line)) {
        flushParagraph();
        closeList();
        const quoted = [];
        while (index < lines.length && /^>\s?/.test(lines[index])) {
          quoted.push(lines[index].replace(/^>\s?/, ''));
          index += 1;
        }
        index -= 1;
        output.push(`<blockquote><p>${inlineMarkdown(quoted.join(' '))}</p></blockquote>`);
        continue;
      }
      const unordered = line.match(/^\s*[-*+]\s+(.*)$/);
      if (unordered) {
        flushParagraph();
        if (list !== 'ul') {
          closeList();
          output.push('<ul>');
          list = 'ul';
        }
        output.push(`<li>${inlineMarkdown(unordered[1])}</li>`);
        continue;
      }
      const ordered = line.match(/^\s*\d+[.)]\s+(.*)$/);
      if (ordered) {
        flushParagraph();
        if (list !== 'ol') {
          closeList();
          output.push('<ol>');
          list = 'ol';
        }
        output.push(`<li>${inlineMarkdown(ordered[1])}</li>`);
        continue;
      }
      paragraph.push(line.trim());
    }
    flushParagraph();
    closeList();

    return `<!doctype html>
<html lang="es">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(title)}</title>
<style>
body{max-width:46rem;margin:3rem auto;padding:0 1.25rem;font-family:system-ui,-apple-system,"Segoe UI",sans-serif;line-height:1.65;color:#1a1d16;background:#fff}
pre{overflow:auto;padding:1rem;border-radius:.5rem;background:#f3f4f1}
code{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:.92em}
blockquote{margin:1rem 0;padding-left:1rem;border-left:3px solid #d6d9cf;color:#4c5048}
img{max-width:100%}
table{border-collapse:collapse}td,th{border:1px solid #d6d9cf;padding:.4rem .6rem}
</style>
</head>
<body>
${output.join('\n') || '<p></p>'}
</body>
</html>`;
  }

  function htmlToMarkdown(html) {
    const documentNode = new DOMParser().parseFromString(html, 'text/html');
    documentNode.querySelectorAll('script,style,noscript,template').forEach((element) => element.remove());

    const walk = (node) => {
      if (node.nodeType === Node.TEXT_NODE) return node.nodeValue.replace(/\s+/g, ' ');
      if (node.nodeType !== Node.ELEMENT_NODE) return '';
      const tag = node.tagName.toLowerCase();
      const children = () => Array.from(node.childNodes).map(walk).join('');
      switch (tag) {
        case 'br':
          return '  \n';
        case 'hr':
          return '\n\n---\n\n';
        case 'h1':
        case 'h2':
        case 'h3':
        case 'h4':
        case 'h5':
        case 'h6': {
          const level = Number(tag[1]);
          return `\n\n${'#'.repeat(level)} ${children().trim()}\n\n`;
        }
        case 'p':
          return `\n\n${children().trim()}\n\n`;
        case 'strong':
        case 'b':
          return `**${children()}**`;
        case 'em':
        case 'i':
          return `*${children()}*`;
        case 'del':
        case 's':
          return `~~${children()}~~`;
        case 'code':
          return `\`${node.textContent}\``;
        case 'pre': {
          const code = node.textContent.replace(/\n$/, '');
          return `\n\n\`\`\`\n${code}\n\`\`\`\n\n`;
        }
        case 'a': {
          const href = node.getAttribute('href') || '';
          return `[${children()}](${href})`;
        }
        case 'img': {
          const alt = node.getAttribute('alt') || '';
          const src = node.getAttribute('src') || '';
          return `![${alt}](${src})`;
        }
        case 'ul':
        case 'ol': {
          const items = Array.from(node.children)
            .filter((child) => child.tagName.toLowerCase() === 'li')
            .map((child, index) => {
              const marker = tag === 'ol' ? `${index + 1}. ` : '- ';
              const content = Array.from(child.childNodes).map(walk).join('').trim().replace(/\n+/g, ' ');
              return `${marker}${content}`;
            });
          return `\n\n${items.join('\n')}\n\n`;
        }
        case 'blockquote': {
          const content = children().trim();
          return `\n\n${content
            .split('\n')
            .map((line) => `> ${line}`)
            .join('\n')}\n\n`;
        }
        case 'table': {
          const rows = Array.from(node.querySelectorAll('tr')).map((row) =>
            Array.from(row.children).map((cell) => cell.textContent.trim().replaceAll('|', '\\|'))
          );
          if (!rows.length) return '';
          const width = Math.max(...rows.map((row) => row.length));
          const normalize = (row) => {
            const next = row.slice();
            while (next.length < width) next.push('');
            return next;
          };
          const [head, ...body] = rows;
          return `\n\n| ${normalize(head).join(' | ')} |\n| ${Array(width).fill('---').join(' | ')} |\n${body
            .map((row) => `| ${normalize(row).join(' | ')} |`)
            .join('\n')}\n\n`;
        }
        case 'div':
        case 'section':
        case 'article':
        case 'main':
        case 'header':
        case 'footer':
        case 'body':
        case 'html':
          return children();
        default:
          return children();
      }
    };

    let markdown = walk(documentNode.body || documentNode.documentElement);
    markdown = markdown.replace(/[ \t]+\n/g, '\n').replace(/\n{3,}/g, '\n\n').trim();
    return `${markdown}\n`;
  }

  // ---------------------------------------------------------------- CSV / TSV / JSON / XML
  function parseDelimited(text, delimiter) {
    const rows = [];
    let row = [];
    let field = '';
    let quoted = false;
    for (let index = 0; index < text.length; index += 1) {
      const character = text[index];
      if (quoted) {
        if (character === '"') {
          if (text[index + 1] === '"') {
            field += '"';
            index += 1;
          } else {
            quoted = false;
          }
        } else {
          field += character;
        }
      } else if (character === '"') {
        quoted = true;
      } else if (character === delimiter) {
        row.push(field);
        field = '';
      } else if (character === '\n' || character === '\r') {
        if (character === '\r' && text[index + 1] === '\n') index += 1;
        row.push(field);
        rows.push(row);
        row = [];
        field = '';
      } else {
        field += character;
      }
    }
    row.push(field);
    rows.push(row);
    while (rows.length && rows[rows.length - 1].length === 1 && rows[rows.length - 1][0] === '') rows.pop();
    return rows;
  }

  function csvCell(value) {
    const text = value == null ? '' : String(value);
    if (/[",\n\r]/.test(text) || /^\s|\s$/.test(text)) return `"${text.replaceAll('"', '""')}"`;
    return text;
  }

  function toCsv(rows) {
    return `${rows.map((row) => row.map(csvCell).join(',')).join('\n')}\n`;
  }

  function csvToJson(text) {
    const rows = parseDelimited(text, ',');
    if (!rows.length) return '[]\n';
    const headers = rows[0].map((header, index) => header.trim() || `column${index + 1}`);
    const objects = rows.slice(1).map((row) => {
      const object = {};
      headers.forEach((header, index) => {
        object[header] = row[index] ?? '';
      });
      return object;
    });
    return `${JSON.stringify(objects, null, 2)}\n`;
  }

  function xmlEscape(value) {
    return String(value)
      .replaceAll('&', '&amp;')
      .replaceAll('<', '&lt;')
      .replaceAll('>', '&gt;')
      .replaceAll('"', '&quot;')
      .replaceAll("'", '&apos;');
  }

  function xmlNames(headers) {
    const used = new Map();
    return headers.map((header, index) => {
      let name = String(header || '').trim().replace(/[^\w.-]/g, '_');
      if (!name) name = `col${index + 1}`;
      if (!/^[A-Za-z_]/.test(name)) name = `col_${name}`;
      const count = used.get(name) || 0;
      used.set(name, count + 1);
      return count ? `${name}_${count + 1}` : name;
    });
  }

  function csvToXml(text) {
    const rows = parseDelimited(text, ',');
    if (!rows.length) return '<?xml version="1.0" encoding="UTF-8"?>\n<rows></rows>\n';
    const headers = xmlNames(rows[0]);
    const body = rows
      .slice(1)
      .map((row) => {
        const fields = headers.map((header, index) => `    <${header}>${xmlEscape(row[index] ?? '')}</${header}>`).join('\n');
        return `  <row>\n${fields}\n  </row>`;
      })
      .join('\n');
    return `<?xml version="1.0" encoding="UTF-8"?>\n<rows>\n${body}\n</rows>\n`;
  }

  function tsvToCsv(text) {
    return toCsv(parseDelimited(text, '\t'));
  }

  function jsonToCsv(text) {
    let data;
    try {
      data = JSON.parse(text);
    } catch {
      throw new Error('El archivo JSON no es válido. Revisa la sintaxis e inténtalo de nuevo.');
    }
    const source = Array.isArray(data) ? data : [data];
    if (!source.length) return '\n';
    const objects = source.map((item) => (item && typeof item === 'object' && !Array.isArray(item) ? item : { value: item }));
    const headers = [];
    objects.forEach((object) => {
      Object.keys(object).forEach((key) => {
        if (!headers.includes(key)) headers.push(key);
      });
    });
    if (!headers.length) headers.push('value');
    const rows = [headers];
    objects.forEach((object) => {
      rows.push(
        headers.map((header) => {
          const value = object[header];
          if (value == null) return '';
          return typeof value === 'object' ? JSON.stringify(value) : String(value);
        })
      );
    });
    return toCsv(rows);
  }

  function xmlToCsv(text) {
    const documentNode = new DOMParser().parseFromString(text, 'application/xml');
    if (documentNode.querySelector('parsererror')) {
      throw new Error('El archivo XML no es válido. Revisa la sintaxis e inténtalo de nuevo.');
    }
    const root = documentNode.documentElement;
    if (!root) throw new Error('El archivo XML está vacío.');
    let rows = Array.from(root.children);
    if (!rows.length) throw new Error('El XML no contiene datos para convertir.');
    if (rows.every((row) => row.children.length === 0)) rows = [root];
    const columns = [];
    rows.forEach((row) => {
      Array.from(row.children).forEach((child) => {
        if (!columns.includes(child.tagName)) columns.push(child.tagName);
      });
    });
    if (!columns.length) {
      return toCsv([['value'], ...rows.map((row) => [row.textContent.trim()])]);
    }
    const matrix = [columns];
    rows.forEach((row) => {
      matrix.push(
        columns.map((column) => {
          const child = Array.from(row.children).find((element) => element.tagName === column);
          return child ? child.textContent.trim() : '';
        })
      );
    });
    return toCsv(matrix);
  }

  // ---------------------------------------------------------------- PDF generation
  const encoder = new TextEncoder();

  const WIN_ANSI = {
    0x20ac: 0x80, 0x201a: 0x82, 0x0192: 0x83, 0x201e: 0x84, 0x2026: 0x85, 0x2020: 0x86,
    0x2021: 0x87, 0x02c6: 0x88, 0x2030: 0x89, 0x0160: 0x8a, 0x2039: 0x8b, 0x0152: 0x8c,
    0x017d: 0x8e, 0x2018: 0x91, 0x2019: 0x92, 0x201c: 0x93, 0x201d: 0x94, 0x2022: 0x95,
    0x2013: 0x96, 0x2014: 0x97, 0x02dc: 0x98, 0x2122: 0x99, 0x0161: 0x9a, 0x203a: 0x9b,
    0x0153: 0x9c, 0x017e: 0x9e, 0x0178: 0x9f,
  };

  function winAnsi(value) {
    let output = '';
    for (const character of value) {
      const code = character.codePointAt(0);
      if (code === 9) output += ' ';
      else if (code < 0x20) output += ' ';
      else if (code <= 0xff) output += String.fromCharCode(code);
      else if (WIN_ANSI[code]) output += String.fromCharCode(WIN_ANSI[code]);
      else output += '?';
    }
    return output;
  }

  function pdfEscape(value) {
    return value.replaceAll('\\', '\\\\').replaceAll('(', '\\(').replaceAll(')', '\\)');
  }

  function concatBytes(parts) {
    const length = parts.reduce((total, part) => total + part.length, 0);
    const output = new Uint8Array(length);
    let offset = 0;
    parts.forEach((part) => {
      output.set(part, offset);
      offset += part.length;
    });
    return output;
  }

  function assemblePdf(objects) {
    const chunks = [encoder.encode('%PDF-1.4\n%FileFlow\n')];
    let length = chunks[0].length;
    const offsets = [];
    for (let index = 1; index < objects.length; index += 1) {
      const object = objects[index];
      if (!object) continue;
      offsets[index] = length;
      const head = encoder.encode(`${index} 0 obj\n${object.dict}\n`);
      chunks.push(head);
      length += head.length;
      if (object.stream) {
        const streamStart = encoder.encode('stream\n');
        chunks.push(streamStart);
        length += streamStart.length;
        chunks.push(object.stream);
        length += object.stream.length;
        const streamEnd = encoder.encode('\nendstream\n');
        chunks.push(streamEnd);
        length += streamEnd.length;
      }
      const end = encoder.encode('endobj\n');
      chunks.push(end);
      length += end.length;
    }

    const xrefOffset = length;
    let xref = `xref\n0 ${objects.length}\n0000000000 65535 f \n`;
    for (let index = 1; index < objects.length; index += 1) {
      xref += `${String(offsets[index] || 0).padStart(10, '0')} 00000 n \n`;
    }
    xref += `trailer\n<< /Size ${objects.length} /Root 1 0 R >>\nstartxref\n${xrefOffset}\n%%EOF\n`;
    chunks.push(encoder.encode(xref));
    return concatBytes(chunks);
  }

  function wrapText(line, maxCharacters) {
    if (!line) return [''];
    const words = line.split(/\s+/);
    const output = [];
    let current = '';
    words.forEach((word) => {
      const candidate = current ? `${current} ${word}` : word;
      if (candidate.length > maxCharacters && current) {
        output.push(current);
        current = word;
      } else {
        current = candidate;
      }
      while (current.length > maxCharacters) {
        output.push(current.slice(0, maxCharacters));
        current = current.slice(maxCharacters);
      }
    });
    if (current) output.push(current);
    return output.length ? output : [''];
  }

  function textToPdf(text) {
    const pageWidth = 595.28;
    const pageHeight = 841.89;
    const margin = 56.7;
    const fontSize = 11;
    const lineHeight = 15.4;
    const maxCharacters = 86;
    const maxLines = Math.floor((pageHeight - margin * 2) / lineHeight);
    const lines = [];
    text
      .replace(/\r\n?/g, '\n')
      .split('\n')
      .forEach((line) => lines.push(...wrapText(line, maxCharacters)));
    const pages = [];
    for (let index = 0; index < lines.length; index += maxLines) {
      pages.push(lines.slice(index, index + maxLines));
    }
    if (!pages.length) pages.push(['']);

    const objects = [];
    objects[1] = { dict: '<< /Type /Catalog /Pages 2 0 R >>' };
    const pageIds = pages.map((_, index) => 4 + index * 2);
    const contentIds = pages.map((_, index) => 5 + index * 2);
    objects[2] = { dict: `<< /Type /Pages /Kids [${pageIds.map((id) => `${id} 0 R`).join(' ')}] /Count ${pages.length} >>` };
    objects[3] = { dict: '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>' };

    pages.forEach((pageLines, index) => {
      const textCommands = pageLines
        .map((line) => `(${pdfEscape(winAnsi(line))}) Tj\nT*\n`)
        .join('');
      const content = concatBytes([
        encoder.encode(`BT\n/F1 ${fontSize} Tf\n${lineHeight} TL\n1 0 0 1 ${margin} ${(pageHeight - margin - fontSize).toFixed(2)} Tm\n`),
        encoder.encode(textCommands),
        encoder.encode('ET\n'),
      ]);
      objects[pageIds[index]] = {
        dict: `<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${pageWidth} ${pageHeight}] /Resources << /Font << /F1 3 0 R >> >> /Contents ${contentIds[index]} 0 R >>`,
      };
      objects[contentIds[index]] = {
        dict: `<< /Length ${content.length} >>`,
        stream: content,
      };
    });

    return assemblePdf(objects);
  }

  function loadImage(source) {
    return new Promise((resolve, reject) => {
      const image = new Image();
      image.onload = () => resolve(image);
      image.onerror = () => reject(new Error('No se pudo leer la imagen.'));
      image.src = source;
    });
  }

  async function imageToPdf(file) {
    const url = URL.createObjectURL(file);
    try {
      const image = await loadImage(url);
      let width = image.naturalWidth;
      let height = image.naturalHeight;
      const maxDimension = 4096;
      if (Math.max(width, height) > maxDimension) {
        const scale = maxDimension / Math.max(width, height);
        width = Math.round(width * scale);
        height = Math.round(height * scale);
      }
      const canvas = document.createElement('canvas');
      canvas.width = width;
      canvas.height = height;
      const context = canvas.getContext('2d');
      context.fillStyle = '#ffffff';
      context.fillRect(0, 0, width, height);
      context.drawImage(image, 0, 0, width, height);
      const dataUrl = canvas.toDataURL('image/jpeg', 0.92);
      const base64 = dataUrl.split(',')[1];
      const binary = atob(base64);
      const jpeg = new Uint8Array(binary.length);
      for (let index = 0; index < binary.length; index += 1) jpeg[index] = binary.charCodeAt(index);

      const pageWidth = 595.28;
      const pageHeight = 841.89;
      const margin = 36;
      const scale = Math.min((pageWidth - margin * 2) / width, (pageHeight - margin * 2) / height);
      const drawWidth = width * scale;
      const drawHeight = height * scale;
      const x = (pageWidth - drawWidth) / 2;
      const y = (pageHeight - drawHeight) / 2;
      const content = encoder.encode(
        `q\n${drawWidth.toFixed(2)} 0 0 ${drawHeight.toFixed(2)} ${x.toFixed(2)} ${y.toFixed(2)} cm\n/Im0 Do\nQ\n`
      );

      const objects = [];
      objects[1] = { dict: '<< /Type /Catalog /Pages 2 0 R >>' };
      objects[2] = { dict: '<< /Type /Pages /Kids [3 0 R] /Count 1 >>' };
      objects[3] = {
        dict: `<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${pageWidth} ${pageHeight}] /Resources << /XObject << /Im0 5 0 R >> >> /Contents 4 0 R >>`,
      };
      objects[4] = { dict: `<< /Length ${content.length} >>`, stream: content };
      objects[5] = {
        dict: `<< /Type /XObject /Subtype /Image /Width ${width} /Height ${height} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length ${jpeg.length} >>`,
        stream: jpeg,
      };
      return new Blob([assemblePdf(objects)], { type: 'application/pdf' });
    } finally {
      URL.revokeObjectURL(url);
    }
  }

  // ---------------------------------------------------------------- Toasts
  function showToast(type, title, message) {
    if (!dom.toastRegion) return;
    const styles = {
      success: { icon: ICONS.check, className: 'text-success' },
      error: { icon: ICONS.alert, className: 'text-danger' },
      info: { icon: ICONS.info, className: 'text-info' },
    };
    const style = styles[type] || styles.info;
    const wrapper = document.createElement('div');
    wrapper.className = 'pointer-events-auto transition-transform duration-300';
    wrapper.innerHTML = `<article role="alert" aria-live="${type === 'error' ? 'assertive' : 'polite'}" aria-atomic="true" data-type="${type}" class="relative flex items-center gap-3 p-3.5 rounded-xl max-w-sm w-full overflow-hidden border border-line bg-surface-raised shadow-panel transition-colors duration-300 pointer-events-auto">
        ${icon(style.icon, `shrink-0 size-5 ${style.className}`)}
        <div class="flex-1 min-w-0">
          <h4 class="font-medium text-sm text-ink">${esc(title)}</h4>
          ${message ? `<p class="mt-0.5 text-xs text-ink-subtle">${esc(message)}</p>` : ''}
        </div>
        <button type="button" class="shrink-0 text-ink-faint hover:text-ink flex items-center gap-1" aria-label="Cerrar notificación">${icon(ICONS.x, 'size-4')}</button>
        <div class="absolute bottom-0 left-0 right-0 h-0.5 overflow-hidden rounded-full bg-surface-sunken">
          <div class="h-full rounded-full bg-accent transition-[width] duration-300 ease-linear" style="width: 100%"></div>
        </div>
      </article>`;
    dom.toastRegion.prepend(wrapper);
    const close = () => wrapper.remove();
    wrapper.querySelector('button')?.addEventListener('click', close);
    window.setTimeout(close, type === 'error' ? 6000 : 4000);
  }

  // ---------------------------------------------------------------- Filters
  function setPill(button, active, sizeClasses) {
    button.setAttribute('aria-pressed', active ? 'true' : 'false');
    button.className = `rounded-md border ${sizeClasses} font-mono text-[10px] uppercase tracking-[0.08em] transition-colors ${
      active
        ? 'border-accent-line bg-accent-soft text-accent'
        : 'border-line bg-surface-sunken text-ink-subtle hover:border-line-strong hover:text-ink'
    }`;
  }

  function filterFormats(category) {
    $$('#supported-formats [data-format-filter]').forEach((button) => {
      setPill(button, button.dataset.formatFilter === category, 'px-2.5 py-1');
    });
    $$('#format-list li[data-format]').forEach((item) => {
      item.hidden = category !== 'Todos' && item.dataset.category !== category;
    });
  }

  function filterPairs(category) {
    $$('#converters [data-pair-filter]').forEach((button) => {
      setPill(button, button.dataset.pairFilter === category, 'px-3 py-1');
    });
    $$('#pair-list > li').forEach((item) => {
      item.hidden = category !== 'Todos' && item.dataset.sourceCategory !== category;
    });
  }

  // ---------------------------------------------------------------- Privacy rows
  function setupPrivacyRows() {
    $$('[data-privacy-row]').forEach((row) => {
      const sourceGroup = $('[data-privacy-sources]', row);
      const targetGroup = $('[data-privacy-targets]', row);
      const count = $('[data-privacy-count]', row);
      const sourceButtons = $$('[data-privacy-source]', row);
      if (!sourceGroup || !targetGroup || !sourceButtons.length) return;

      const render = (source) => {
        sourceButtons.forEach((button) => {
          const active = button.dataset.privacySource === source;
          button.setAttribute('aria-pressed', active ? 'true' : 'false');
          button.classList.toggle('ring-2', active);
          button.classList.toggle('ring-accent', active);
          button.classList.toggle('ring-offset-2', active);
          button.classList.toggle('ring-offset-surface', active);
          button.classList.toggle('opacity-60', !active);
          button.classList.toggle('hover:opacity-100', !active);
        });
        const targets = TARGETS[source] || [];
        if (count) count.textContent = `${source} → ${targets.length}`;
        targetGroup.setAttribute('aria-label', `Destinos compatibles desde ${source}`);
        targetGroup.innerHTML = targets
          .map(
            (target) => `<button type="button" title="Convertir de ${esc(source)} a ${esc(target)}" aria-label="Convertir de ${esc(source)} a ${esc(target)}" data-action="convert-pair" data-source="${esc(source)}" data-target="${esc(target)}" class="inline-flex items-center justify-center rounded-full border border-transparent font-mono font-medium tracking-[0.08em] text-center transition-colors px-2 py-0.5 text-[10px] ${FORMATS[target]?.badge || ''} cursor-pointer select-none">${esc(target)}</button>`
          )
          .join('');
      };

      sourceButtons.forEach((button) => {
        button.addEventListener('click', () => render(button.dataset.privacySource));
      });
      const initial = sourceButtons.find((button) => button.getAttribute('aria-pressed') === 'true') || sourceButtons[0];
      if (initial) render(initial.dataset.privacySource);
    });
  }

  // ---------------------------------------------------------------- Conversion request / scroll
  function requestConversion(source, target) {
    const options = TARGETS[state.format] || [];
    if (state.file && state.format === source && options.includes(target)) {
      state.target = target;
      state.pendingPair = null;
      state.conversionError = null;
      renderSelected();
    } else {
      state.pendingPair = { source, target };
      if (state.status === 'idle') renderIdle();
    }
    scrollToConverter();
  }

  function scrollToConverter() {
    const section = document.getElementById('convert');
    if (!section) return;
    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    section.scrollIntoView({ behavior: reduced ? 'auto' : 'smooth', block: 'center' });
  }

  // ---------------------------------------------------------------- Mobile menu
  let menuOpen = false;
  let menuPreviousFocus = null;

  function focusableElements() {
    if (!dom.menu) return [];
    return $$('a[href], button:not([disabled]), summary, input, select, textarea, [tabindex]:not([tabindex="-1"])', dom.menu).filter(
      (element) => element.offsetParent !== null
    );
  }

  function openMenu() {
    if (!dom.menu || menuOpen) return;
    menuOpen = true;
    menuPreviousFocus = document.activeElement;
    dom.menu.hidden = false;
    dom.menuTrigger?.setAttribute('aria-expanded', 'true');
    document.body.style.overflow = 'hidden';
    const focusables = focusableElements();
    (focusables.find((element) => element.getAttribute('aria-label') === 'Cerrar menú') || focusables[0])?.focus();
  }

  function closeMenu() {
    if (!dom.menu || !menuOpen) return;
    menuOpen = false;
    dom.menu.hidden = true;
    dom.menuTrigger?.setAttribute('aria-expanded', 'false');
    document.body.style.overflow = '';
    if (menuPreviousFocus && typeof menuPreviousFocus.focus === 'function') menuPreviousFocus.focus();
    menuPreviousFocus = null;
  }

  function onMenuKeydown(event) {
    if (!menuOpen) return;
    if (event.key === 'Escape') {
      event.preventDefault();
      closeMenu();
      return;
    }
    if (event.key !== 'Tab') return;
    const focusables = focusableElements();
    if (!focusables.length) return;
    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  // ---------------------------------------------------------------- Active section
  function setActiveSection(section) {
    const label = SECTION_LABELS[section];
    if (!label) return;
    if (dom.breadcrumb) dom.breadcrumb.textContent = label;
    $$('a[data-section]').forEach((link) => {
      const active = link.dataset.section === section;
      const mobile = Boolean(link.closest('#mobile-navigation'));
      if (mobile) {
        link.className = `flex items-center gap-2.5 rounded-lg px-2.5 py-2.5 text-[13px] transition-colors ${
          active ? 'bg-accent-soft text-accent' : 'text-ink-subtle hover:bg-surface-raised hover:text-ink'
        }`;
      } else {
        link.className = `relative flex items-center gap-2.5 rounded-lg px-2.5 py-2 text-[13px] transition-colors ${
          active ? 'bg-accent-soft text-accent' : 'text-ink-subtle hover:bg-surface-raised hover:text-ink'
        }`;
        let indicator = link.querySelector('[data-active-indicator]');
        if (active && !indicator) {
          indicator = document.createElement('span');
          indicator.dataset.activeIndicator = '';
          indicator.setAttribute('aria-hidden', 'true');
          indicator.className = 'absolute left-0 top-1/2 h-4 w-0.5 -translate-y-1/2 rounded-full bg-accent';
          link.prepend(indicator);
        } else if (!active && indicator) {
          indicator.remove();
        }
      }
      if (active) link.setAttribute('aria-current', 'location');
      else link.removeAttribute('aria-current');
    });
  }

  function setupActiveSections() {
    $$('aside nav a[data-section] span.absolute').forEach((span) => {
      span.dataset.activeIndicator = '';
    });
    const sections = ['convert', 'converters', 'how-it-works', 'privacy', 'faq']
      .map((id) => document.getElementById(id))
      .filter(Boolean);
    if (!sections.length || typeof IntersectionObserver === 'undefined') return;
    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((entry) => entry.isIntersecting)
          .sort((a, b) => b.intersectionRatio - a.intersectionRatio);
        if (visible[0]) setActiveSection(visible[0].target.id);
      },
      { rootMargin: '-18% 0px -66% 0px', threshold: [0, 0.2, 0.5, 1] }
    );
    sections.forEach((section) => observer.observe(section));
    setActiveSection('convert');
  }

  // ---------------------------------------------------------------- Events
  function bindEvents() {
    document.addEventListener('click', (event) => {
      const target = event.target;

      const themeButton = target.closest('[data-theme-toggle]');
      if (themeButton) {
        applyTheme(getTheme() === 'dark' ? 'light' : 'dark');
        return;
      }

      if (target.closest('[data-menu-open]')) {
        openMenu();
        return;
      }
      if (target.closest('[data-menu-close]') || target.closest('[data-menu-overlay]')) {
        closeMenu();
        return;
      }

      const menuLink = target.closest('[data-menu-link]');
      if (menuLink) {
        closeMenu();
      }

      const actionElement = target.closest('[data-action]');
      if (actionElement) {
        const action = actionElement.dataset.action;
        if (action === 'open-file') {
          event.preventDefault();
          openFilePicker();
          return;
        }
        if (action === 'remove-file') {
          event.preventDefault();
          removeFile();
          return;
        }
        if (action === 'convert') {
          event.preventDefault();
          convertCurrent();
          return;
        }
        if (action === 'dismiss-error') {
          event.preventDefault();
          renderIdle();
          announce('Archivo descartado.');
          return;
        }
        if (action === 'dismiss-conversion-error') {
          event.preventDefault();
          state.conversionError = null;
          renderSelected();
          return;
        }
        if (action === 'dismiss-notice') {
          event.preventDefault();
          state.notice = false;
          renderSelected();
          return;
        }
        if (action === 'convert-pair') {
          event.preventDefault();
          requestConversion(actionElement.dataset.source, actionElement.dataset.target);
          return;
        }
      }

      const formatFilter = target.closest('[data-format-filter]');
      if (formatFilter) {
        filterFormats(formatFilter.dataset.formatFilter);
        return;
      }
      const pairFilter = target.closest('[data-pair-filter]');
      if (pairFilter) {
        filterPairs(pairFilter.dataset.pairFilter);
      }
    });

    document.addEventListener('change', (event) => {
      const input = event.target.closest('input[name="target-format"]');
      if (input) state.target = input.value;
    });

    document.addEventListener('keydown', onMenuKeydown);

    dom.fileInput?.addEventListener('change', (event) => {
      const file = event.target.files?.[0];
      event.target.value = '';
      if (file) handleFile(file);
    });

    if (dom.convertBody) {
      let dragDepth = 0;
      const hasFiles = (event) => Array.from(event.dataTransfer?.types || []).includes('Files');

      dom.convertBody.addEventListener('dragenter', (event) => {
        if (!hasFiles(event)) return;
        event.preventDefault();
        dragDepth += 1;
        if (state.status === 'idle') setDragging(true);
      });
      dom.convertBody.addEventListener('dragover', (event) => {
        if (!hasFiles(event)) return;
        event.preventDefault();
        event.dataTransfer.dropEffect = 'copy';
      });
      dom.convertBody.addEventListener('dragleave', (event) => {
        if (!hasFiles(event)) return;
        if (dragDepth > 0) dragDepth -= 1;
        if (dragDepth === 0 && state.status === 'idle') setDragging(false);
      });
      dom.convertBody.addEventListener('drop', (event) => {
        if (!hasFiles(event)) return;
        event.preventDefault();
        dragDepth = 0;
        if (state.status === 'idle') setDragging(false);
        const file = event.dataTransfer.files?.[0];
        if (file) handleFile(file);
      });
    }

    window.addEventListener('resize', () => {
      if (window.innerWidth >= 1024) closeMenu();
    });

    window.addEventListener('dragover', (event) => {
      if (Array.from(event.dataTransfer?.types || []).includes('Files') && !event.target.closest('#convert')) {
        event.preventDefault();
      }
    });
    window.addEventListener('drop', (event) => {
      if (Array.from(event.dataTransfer?.types || []).includes('Files') && !event.target.closest('#convert')) {
        event.preventDefault();
      }
    });
  }

  function setYear() {
    const element = $$('footer span').find((span) => /^©\s*\d{4}/.test(span.textContent.trim()));
    if (element) element.textContent = element.textContent.replace(/©\s*\d{4}/, `© ${new Date().getFullYear()}`);
  }

  function init() {
    applyTheme(getTheme(), false);
    bindEvents();
    setupPrivacyRows();
    setupActiveSections();
    const selectButton = $('#dropzone button');
    if (selectButton) selectButton.setAttribute('data-action', 'open-file');
    setYear();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
