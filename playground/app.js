// Playground behaviour: load the wasm renderer, re-render on edits, show
// diagnostics, and let the user download the current PNG.
//
// Contract with the module (see crates/rupa-wasm/src/abi.rs):
//   rupa_alloc(len) -> ptr
//   rupa_render(src_ptr, src_len) -> frame_ptr
//   frame = [u32 total][u32 meta_len][meta JSON][PNG bytes]
//   rupa_free(ptr, len), rupa_free_frame(frame_ptr)

const STARTER_SOURCE = `size 320px x 240px
background white

circle center at (160px, 120px)
    radius 80px
    fill "#7950F2"

rectangle at (196px, 148px)
    size 96px x 64px
    fill orange
    opacity 0.85
`;

const EXAMPLES = [
  { file: "minimal.rupa", label: "Minimal" },
  { file: "inline-shapes.rupa", label: "Inline properties" },
  { file: "shapes.rupa", label: "Shapes" },
  { file: "paints.rupa", label: "Gradients and patterns" },
  { file: "layers.rupa", label: "Layers and groups" },
  { file: "layer-fill.rupa", label: "Layer background" },
  { file: "procedural.rupa", label: "Procedural loops" },
  { file: "raster.rupa", label: "Raster drawing" },
  { file: "text.rupa", label: "Text with the default font" },
];

const DEBOUNCE_MS = 250;

// Word classes are filled from the module's `rupa_metadata()` export, so the
// highlighter, lexer, named-colour table, and language reference share one
// source. Nothing about the language is hard-coded here.
const TOKEN_CLASSES = {
  keyword: "tok-keyword",
  property: "tok-property",
  literal: "tok-literal",
};
let WORD_CLASSES = new Map();
let TOKEN_PATTERN = null;

const elements = {
  editor: document.querySelector("#editor"),
  gutter: document.querySelector("#gutter"),
  highlight: document.querySelector("#highlight"),
  diagnostics: document.querySelector("#diagnostics"),
  canvas: document.querySelector("#preview"),
  placeholder: document.querySelector("#viewer-placeholder"),
  statusDot: document.querySelector("#status-dot"),
  statusText: document.querySelector("#status-text"),
  canvasSize: document.querySelector("#canvas-size"),
  examples: document.querySelector("#examples"),
  formatButton: document.querySelector("#format"),
  renderButton: document.querySelector("#render"),
  downloadButton: document.querySelector("#download"),
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();
const context = elements.canvas.getContext("2d");

/** Module exports, or null until the wasm module is ready. */
let moduleExports = null;
/** PNG bytes of the last successful render, for download. */
let latestPng = null;
/** Increments per render so stale image decodes are discarded. */
let renderGeneration = 0;
let debounceTimer = 0;

/** Render line numbers matching the editor's current line count. */
function updateGutter() {
  const lines = elements.editor.value.split("\n").length;
  elements.gutter.textContent = Array.from(
    { length: lines },
    (_, index) => index + 1,
  ).join("\n");
}

const HTML_ESCAPES = { "&": "&amp;", "<": "&lt;", ">": "&gt;" };

/** Escape text for insertion into the highlight layer. */
const escapeHtml = (text) =>
  text.replace(/[&<>]/g, (character) => HTML_ESCAPES[character]);

/** Escape one literal unit for a regular-expression alternation. */
const escapeRegExp = (text) => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/** Build the word map and numeric-unit regex from the module's metadata. */
function setupMetadata(metadata) {
  WORD_CLASSES = new Map();

  for (const [name, words] of Object.entries(metadata.keyword_classes ?? {})) {
    const className = TOKEN_CLASSES[name];
    if (!className) continue;
    for (const word of words) WORD_CLASSES.set(word, className);
  }
  for (const color of metadata.named_colors ?? []) {
    WORD_CLASSES.set(color, "tok-color");
  }

  const units = [...(metadata.units ?? [])]
    .sort((a, b) => b.length - a.length)
    .map(escapeRegExp)
    .join("|");
  TOKEN_PATTERN = new RegExp(
    `(\\/\\/[^\\n]*)|("(?:[^"\\\\]|\\\\.)*")|(#[0-9a-fA-F]{3,8}\\b|-?\\d+(?:\\.\\d+)?(?:${units})?)|([A-Za-z][A-Za-z0-9_-]*)`,
    "g",
  );
}

/** Build the highlighted HTML for one source string. */
function highlightSource(source) {
  if (!TOKEN_PATTERN) return `${escapeHtml(source)}\n`;

  let html = "";
  let consumed = 0;

  for (const match of source.matchAll(TOKEN_PATTERN)) {
    const text = match[0];
    const start = match.index ?? 0;
    if (start > consumed) html += escapeHtml(source.slice(consumed, start));

    let className = "";
    if (match[1] !== undefined) className = "tok-comment";
    else if (match[2] !== undefined) className = "tok-string";
    else if (match[3] !== undefined) className = "tok-number";
    else if (match[4] !== undefined) className = WORD_CLASSES.get(match[4]) ?? "";

    const tail = escapeHtml(text);
    html += className ? `<span class="${className}">${tail}</span>` : tail;
    consumed = start + text.length;
  }

  html += escapeHtml(source.slice(consumed));
  // A trailing blank line still needs height in the layer behind the textarea.
  return `${html}\n`;
}

function syncEditorScroll() {
  elements.highlight.scrollTop = elements.editor.scrollTop;
  elements.highlight.scrollLeft = elements.editor.scrollLeft;
}

/** Repaint the highlight layer and keep it aligned with the editor scroll. */
function updateHighlight() {
  elements.highlight.innerHTML = highlightSource(elements.editor.value);
  syncEditorScroll();
}

/** Refresh both decorations that mirror the source: line numbers and colour. */
function refreshEditor() {
  updateGutter();
  updateHighlight();
}

/** Insert four spaces at the caret, preserving the selection. */
function indentSelection() {
  const { selectionStart, selectionEnd, value } = elements.editor;
  const indented = `${value.slice(0, selectionStart)}    ${value.slice(selectionEnd)}`;
  elements.editor.value = indented;
  const caret = selectionStart + 4;
  elements.editor.selectionStart = caret;
  elements.editor.selectionEnd = caret;
  refreshEditor();
}

/** Create a small text element for the diagnostics list. */
function textElement(tag, className, text) {
  const node = document.createElement(tag);
  node.className = className;
  node.textContent = text;
  return node;
}

/** Update the status indicator. */
function setStatus(state, text) {
  elements.statusDot.dataset.state = state;
  elements.statusText.textContent = text;
}

/** Replace the diagnostics list with compile/render problems. */
function showDiagnostics(diagnostics, renderError) {
  const list = elements.diagnostics;
  list.replaceChildren();

  const entries = [...diagnostics];
  if (entries.length === 0 && !renderError) {
    const empty = document.createElement("li");
    empty.className = "diagnostics__empty";
    empty.textContent = "No problems.";
    list.append(empty);
    return;
  }

  for (const diagnostic of entries) {
    const item = document.createElement("li");
    item.className = "diagnostic";
    item.dataset.severity = diagnostic.severity ?? "error";

    item.append(
      textElement("span", "diagnostic__code", diagnostic.code ?? "—"),
      textElement("span", "diagnostic__message", diagnostic.message ?? ""),
      textElement(
        "span",
        "diagnostic__location",
        `${diagnostic.line ?? 1}:${diagnostic.column ?? 1}`,
      ),
    );

    if (diagnostic.help) {
      item.append(textElement("span", "diagnostic__help", diagnostic.help));
    }
    list.append(item);
  }

  if (renderError) {
    const item = document.createElement("li");
    item.className = "diagnostic";
    item.dataset.severity = "error";

    item.append(
      textElement("span", "diagnostic__code", "render"),
      textElement("span", "diagnostic__message", renderError),
    );
    list.append(item);
  }
}

/**
 * Copy the source into module memory, run one export that returns a frame, and
 * return the parsed metadata with the raw body (PNG bytes or formatted text).
 */
function callWithSource(source, exportFn) {
  const bytes = encoder.encode(source);
  const sourcePointer = moduleExports.rupa_alloc(bytes.length);
  if (!sourcePointer) throw new Error("module ran out of memory for the source");
  new Uint8Array(moduleExports.memory.buffer, sourcePointer, bytes.length).set(bytes);

  let framePointer = 0;
  try {
    framePointer = exportFn(sourcePointer, bytes.length);
  } finally {
    moduleExports.rupa_free(sourcePointer, bytes.length);
  }
  if (!framePointer) throw new Error("module ran out of memory for the result");

  try {
    return readFrame(framePointer);
  } finally {
    moduleExports.rupa_free_frame(framePointer);
  }
}

/** Read one frame header, metadata object, and body from module memory. */
function readFrame(framePointer) {
  // Copy immediately: any later call may grow memory and detach these views.
  const header = new DataView(moduleExports.memory.buffer, framePointer, 8);
  const total = header.getUint32(0, true);
  const metaLength = header.getUint32(4, true);
  const frame = new Uint8Array(
    moduleExports.memory.buffer,
    framePointer,
    total,
  ).slice();
  return {
    meta: JSON.parse(decoder.decode(frame.subarray(8, 8 + metaLength))),
    body: frame.subarray(8 + metaLength),
  };
}

/** Render one source string and return its metadata plus PNG bytes. */
function renderSourceInModule(source) {
  const { meta, body } = callWithSource(source, moduleExports.rupa_render);
  return { meta, png: body };
}

/** Format one source string exactly as `rupa fmt` would. */
function formatSourceInModule(source) {
  const { meta, body } = callWithSource(source, moduleExports.rupa_format);
  return { meta, text: decoder.decode(body) };
}

/** Read the language metadata embedded in the module. */
function metadataInModule() {
  const framePointer = moduleExports.rupa_metadata();
  if (!framePointer) throw new Error("module ran out of memory for metadata");
  try {
    const { body } = readFrame(framePointer);
    return JSON.parse(decoder.decode(body));
  } finally {
    moduleExports.rupa_free_frame(framePointer);
  }
}

/** Draw a rendered PNG onto the preview canvas. */
async function drawPreview(png, width, height) {
  const generation = renderGeneration;
  const bitmap = await createImageBitmap(new Blob([png], { type: "image/png" }));
  if (generation !== renderGeneration) {
    bitmap.close();
    return;
  }
  elements.canvas.width = width;
  elements.canvas.height = height;
  context.clearRect(0, 0, width, height);
  context.drawImage(bitmap, 0, 0);
  bitmap.close();
  elements.canvasSize.textContent = `${width} × ${height}`;
  elements.placeholder.hidden = true;
  latestPng = png;
  elements.downloadButton.disabled = false;
}

/** Compile and render the editor contents. */
async function render() {
  if (!moduleExports) return;
  renderGeneration += 1;
  const generation = renderGeneration;
  setStatus("busy", "Rendering…");

  let result;
  const startedAt = performance.now();
  try {
    result = renderSourceInModule(elements.editor.value);
  } catch (error) {
    if (generation === renderGeneration) {
      setStatus("error", "Renderer failed");
      showDiagnostics([], String(error));
    }
    return;
  }
  const elapsed = Math.round(performance.now() - startedAt);

  if (generation !== renderGeneration) return;

  showDiagnostics(result.meta.diagnostics ?? [], result.meta.renderError);

  if (result.meta.status === "ok") {
    await drawPreview(result.png, result.meta.width, result.meta.height);
    if (generation !== renderGeneration) return;
    const warnings = (result.meta.diagnostics ?? []).filter(
      (diagnostic) => diagnostic.severity === "warning",
    ).length;
    setStatus(
      "ok",
      `${result.meta.width}×${result.meta.height} in ${elapsed} ms${warnings ? ` · ${warnings} warning${warnings === 1 ? "" : "s"}` : ""}`,
    );
    return;
  }

  const problems = (result.meta.diagnostics ?? []).length;
  setStatus(
    "error",
    problems
      ? `${problems} problem${problems === 1 ? "" : "s"}`
      : "Render failed",
  );
}

/**
 * Replace the editor contents with the canonical `rupa fmt` layout.
 *
 * The module formats token-wise, so a syntactically broken but readable source
 * still gets tidied; the following render reports what is wrong with it.
 */
function formatEditor() {
  if (!moduleExports) return;

  let result;
  try {
    result = formatSourceInModule(elements.editor.value);
  } catch (error) {
    setStatus("error", "Formatter failed");
    showDiagnostics([], String(error));
    return;
  }

  if (result.meta.status !== "ok") {
    showDiagnostics(result.meta.diagnostics ?? [], result.meta.error);
    setStatus("error", "Cannot format");
    return;
  }

  if (result.text !== elements.editor.value) {
    const { selectionStart, selectionEnd, value, scrollTop, scrollLeft } =
      elements.editor;
    const caretAtEnd =
      selectionStart === value.length && selectionEnd === value.length;
    const limit = result.text.length;

    elements.editor.value = result.text;
    // Formatting changes offsets, so the caret is clamped, not re-mapped; a
    // caret that was at the end stays at the end of the new text.
    elements.editor.selectionStart = caretAtEnd
      ? limit
      : Math.min(selectionStart, limit);
    elements.editor.selectionEnd = caretAtEnd
      ? limit
      : Math.min(selectionEnd, limit);
    // Assigning `value` scrolls back to the top; keep the reader's place.
    elements.editor.scrollTop = scrollTop;
    elements.editor.scrollLeft = scrollLeft;
    refreshEditor();
  }

  renderNow();
}

/** Cancel a pending debounce and render right away. */
function renderNow() {
  window.clearTimeout(debounceTimer);
  render();
}

/** Debounced render for typing. */
function scheduleRender() {
  window.clearTimeout(debounceTimer);
  debounceTimer = window.setTimeout(render, DEBOUNCE_MS);
}

/** Load the wasm module and enable rendering. */
async function loadRenderer() {
  setStatus("busy", "Loading renderer…");
  try {
    const response = await fetch("rupa_wasm.wasm");
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    let instance;
    if (typeof WebAssembly.instantiateStreaming === "function") {
      try {
        ({ instance } = await WebAssembly.instantiateStreaming(response.clone(), {}));
      } catch {
        ({ instance } = await WebAssembly.instantiate(
          await response.arrayBuffer(),
          {},
        ));
      }
    } else {
      ({ instance } = await WebAssembly.instantiate(
        await response.arrayBuffer(),
        {},
      ));
    }
    moduleExports = instance.exports;
    setupMetadata(metadataInModule());
    refreshEditor();
    elements.renderButton.disabled = false;
    elements.formatButton.disabled = false;
    await render();
  } catch (error) {
    setStatus("error", "Renderer unavailable");
    elements.placeholder.textContent = `Failed to load the renderer: ${error}. Run playground/build.sh and serve the page over HTTP.`;
    showDiagnostics([], String(error));
  }
}

/** Populate the examples dropdown and wire its selection. */
function setupExamples() {
  for (const example of EXAMPLES) {
    const option = document.createElement("option");
    option.value = example.file;
    option.textContent = example.label;
    elements.examples.append(option);
  }

  elements.examples.addEventListener("change", async () => {
    const file = elements.examples.value;
    if (!file) return;
    try {
      const response = await fetch(`examples/${file}`);
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      elements.editor.value = await response.text();
      elements.editor.scrollTop = 0;
      refreshEditor();
      await render();
    } catch (error) {
      setStatus("error", "Example unavailable");
      showDiagnostics([], `cannot load ${file}: ${error}`);
    }
  });
}

/** Download the last successful render. */
function downloadPng() {
  if (!latestPng) return;
  const url = URL.createObjectURL(new Blob([latestPng], { type: "image/png" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = "rupa-preview.png";
  link.click();
  URL.revokeObjectURL(url);
}

elements.editor.value = STARTER_SOURCE;
refreshEditor();
elements.renderButton.disabled = true;
elements.formatButton.disabled = true;
elements.editor.addEventListener("input", () => {
  refreshEditor();
  scheduleRender();
});
elements.editor.addEventListener("scroll", () => {
  elements.gutter.scrollTop = elements.editor.scrollTop;
  syncEditorScroll();
});
elements.editor.addEventListener("keydown", (event) => {
  if (event.key === "Tab") {
    event.preventDefault();
    indentSelection();
    scheduleRender();
    return;
  }
  if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
    event.preventDefault();
    renderNow();
  }
});
elements.renderButton.addEventListener("click", renderNow);
elements.formatButton.addEventListener("click", formatEditor);
elements.downloadButton.addEventListener("click", downloadPng);

setupExamples();
loadRenderer();
