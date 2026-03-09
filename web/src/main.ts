import init, {
  generate_edges_binary,
  get_cached_svg,
  init_panic_hook,
} from "puzzle-wasm";
import "./style.css";

function randomSeed(): string {
  return Math.random().toString(36).substring(2, 10);
}

// ─── DOM References ─────────────────────────────────────────

let rowsInput: HTMLInputElement;
let colsInput: HTMLInputElement;
let widthInput: HTMLInputElement;
let heightInput: HTMLInputElement;
let unitSelect: HTMLSelectElement;
let tabSlider: HTMLInputElement;
let taperSlider: HTMLInputElement;
let radiusSlider: HTMLInputElement;
let seedInput: HTMLInputElement;
let pieceCount: HTMLElement;
let errorDisplay: HTMLElement;

let tabReadout: HTMLElement;
let taperReadout: HTMLElement;
let radiusReadout: HTMLElement;
let tabRandomize: HTMLInputElement;
let tabMaxSlider: HTMLInputElement;
let taperRandomize: HTMLInputElement;
let taperMaxSlider: HTMLInputElement;
let tabTrack: HTMLElement;
let taperTrack: HTMLElement;

let pieceTargetInput: HTMLInputElement;
let pieceSizeWarning: HTMLElement;
let gridLockBtn: HTMLElement;
let dimsLockBtn: HTMLElement;
let gridLocked = false;
let dimsLocked = false;

let rulerWidth: HTMLElement;
let rulerHeight: HTMLElement;
let svgViewport: HTMLElement;
let zoomLevelDisplay: HTMLElement;
let zoomInBtn: HTMLElement;
let zoomOutBtn: HTMLElement;
let zoomResetBtn: HTMLElement;

// ─── Zoom/Pan State ──────────────────────────────────────────

let zoomLevel = 1;
let panX = 0;
let panY = 0;
let isPanning = false;
let panStartX = 0;
let panStartY = 0;

let rafPending = false;

const MIN_ZOOM = 0.5;
const MAX_ZOOM = 20;
const ZOOM_STEP = 1.15; // 15% per wheel tick

// ─── Canvas State ────────────────────────────────────────────

let canvas: HTMLCanvasElement | null = null;
let ctx: CanvasRenderingContext2D | null = null;
let edgesData: Float64Array | null = null;
let borderData: Float64Array | null = null;
let puzzleWidth = 0;
let puzzleHeight = 0;
const EDGE_STRIDE = 36;

// ─── Config Builder ─────────────────────────────────────────

function buildConfig(): object {
  const tabConfig: Record<string, unknown> = {
    size_pct: parseFloat(tabSlider.value),
    taper: 0.57 + parseFloat(taperSlider.value) * 0.75,
  };
  if (tabRandomize.checked) {
    tabConfig.size_pct_max = parseFloat(tabMaxSlider.value);
  }
  if (taperRandomize.checked) {
    tabConfig.taper_max = 0.57 + parseFloat(taperMaxSlider.value) * 0.75;
  }
  return {
    rows: parseInt(rowsInput.value, 10),
    cols: parseInt(colsInput.value, 10),
    width: parseFloat(widthInput.value),
    height: parseFloat(heightInput.value),
    unit: unitSelect.value,
    tab: tabConfig,
    border: { corner_radius: parseFloat(radiusSlider.value) },
    seed: seedInput.value,
  };
}

// ─── URL Param Sync ──────────────────────────────────────────

function loadFromURL(): boolean {
  const params = new URLSearchParams(window.location.search);
  if (params.size === 0) return false;

  const rows = parseInt(params.get("rows") ?? "6", 10);
  const cols = parseInt(params.get("cols") ?? "8", 10);
  const w = parseFloat(params.get("w") ?? "297");
  const h = parseFloat(params.get("h") ?? "210");
  const unitParam = params.get("unit") ?? "mm";
  const unit = unitParam === "in" ? "Inches" : "Millimeters";
  const tab = Math.max(0.15, Math.min(0.25, parseInt(params.get("tab") ?? "25", 10) / 100));
  const radius = parseFloat(params.get("radius") ?? "2");
  const taperUser = parseInt(params.get("taper") ?? "0", 10) / 100;
  const taper = Math.max(0, Math.min(1, taperUser));
  const seed = params.get("seed") ?? "";

  rowsInput.value = String(rows);
  colsInput.value = String(cols);
  widthInput.value = String(w);
  heightInput.value = String(h);
  unitSelect.value = unit;
  tabSlider.value = String(tab);
  taperSlider.value = String(taper);
  radiusSlider.value = String(radius);
  seedInput.value = seed || randomSeed();

  // Restore randomize state
  if (params.get("tabr") === "1") {
    tabRandomize.checked = true;
    const tabMax = Math.max(0.15, Math.min(0.25, parseInt(params.get("tabmax") ?? "25", 10) / 100));
    tabMaxSlider.value = String(tabMax);
    tabMaxSlider.style.display = "";
  }
  if (params.get("taperr") === "1") {
    taperRandomize.checked = true;
    const taperMax = Math.max(0, Math.min(1, parseInt(params.get("tapermax") ?? "0", 10) / 100));
    taperMaxSlider.value = String(taperMax);
    taperMaxSlider.style.display = "";
  }

  return true;
}

function updateURL(): void {
  const config = buildConfig() as Record<string, unknown>;
  const tabObj = config.tab as Record<string, number>;
  const borderObj = config.border as { corner_radius: number };
  const params = new URLSearchParams();
  params.set("rows", String(config.rows));
  params.set("cols", String(config.cols));
  params.set("w", String(config.width));
  params.set("h", String(config.height));
  params.set("unit", config.unit === "Inches" ? "in" : "mm");
  params.set("tab", String(Math.round(tabObj.size_pct * 100)));
  params.set("taper", String(Math.round(parseFloat(taperSlider.value) * 100)));
  params.set("radius", String(borderObj.corner_radius));
  params.set("seed", String(config.seed));
  if (tabRandomize.checked) {
    params.set("tabr", "1");
    params.set("tabmax", String(Math.round(parseFloat(tabMaxSlider.value) * 100)));
  }
  if (taperRandomize.checked) {
    params.set("taperr", "1");
    params.set("tapermax", String(Math.round(parseFloat(taperMaxSlider.value) * 100)));
  }
  history.replaceState(null, "", "?" + params.toString());
}

// ─── Debounced URL Sync ──────────────────────────────────────

let urlTimeout: ReturnType<typeof setTimeout> | null = null;
function scheduleURLUpdate(): void {
  if (urlTimeout !== null) clearTimeout(urlTimeout);
  urlTimeout = setTimeout(updateURL, 300);
}

// ─── Dynamic Tab Size Clamping ───────────────────────────────

function updateTabMax(): void {
  const rows = parseInt(rowsInput.value, 10) || 1;
  const cols = parseInt(colsInput.value, 10) || 1;
  const w = parseFloat(widthInput.value) || 1;
  const h = parseFloat(heightInput.value) || 1;
  const cellW = w / cols;
  const cellH = h / rows;
  const maxH = cellH / (2.0 * cellW * 1.2);
  const maxV = cellW / (2.0 * cellH * 1.2);
  const maxApproach = 1.0 / (2.0 * 1.2);
  const safeMax = Math.min(maxH, maxV, maxApproach) * 0.9;
  const tabMax = Math.min(safeMax, 0.25);

  tabSlider.max = String(tabMax);
  tabMaxSlider.max = String(tabMax);
  if (parseFloat(tabSlider.value) > tabMax) tabSlider.value = String(tabMax);
  if (parseFloat(tabMaxSlider.value) > tabMax) tabMaxSlider.value = String(tabMax);
}

// ─── Ruler Update ───────────────────────────────────────────

function updateRuler(): void {
  const w = parseFloat(widthInput.value);
  const h = parseFloat(heightInput.value);
  const unit = unitSelect.value === "Inches" ? "in" : "mm";
  const fmt = unit === "mm" ? 0 : 2;
  rulerWidth.textContent = `${w.toFixed(fmt)} ${unit}`;
  rulerHeight.textContent = `${h.toFixed(fmt)} ${unit}`;
}

// ─── Canvas Resize ───────────────────────────────────────────

function resizeCanvas(): void {
  if (!canvas || !ctx) return;
  const dpr = window.devicePixelRatio || 1;
  const rect = svgViewport.getBoundingClientRect();
  canvas.width = rect.width * dpr;
  canvas.height = rect.height * dpr;
  canvas.style.width = rect.width + "px";
  canvas.style.height = rect.height + "px";
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
}

// ─── Canvas Drawing ──────────────────────────────────────────

function drawBorder(c: CanvasRenderingContext2D): void {
  if (!borderData) return;
  c.beginPath();
  let i = 0;
  while (i < borderData.length) {
    const cmd = borderData[i];
    if (cmd === 0) {
      // moveTo
      c.moveTo(borderData[i + 1], borderData[i + 2]);
      i += 3;
    } else if (cmd === 1) {
      // lineTo
      c.lineTo(borderData[i + 1], borderData[i + 2]);
      i += 3;
    } else if (cmd === 2) {
      // curveTo
      c.bezierCurveTo(
        borderData[i + 1],
        borderData[i + 2],
        borderData[i + 3],
        borderData[i + 4],
        borderData[i + 5],
        borderData[i + 6],
      );
      i += 7;
    } else if (cmd === 3) {
      // closePath
      c.closePath();
      i += 1;
    } else {
      i += 1;
    }
  }
  c.stroke();
}

function drawVisibleEdges(
  c: CanvasRenderingContext2D,
  vpL: number,
  vpT: number,
  vpR: number,
  vpB: number,
): void {
  if (!edgesData) return;
  const data = edgesData;
  const len = data.length;

  c.beginPath();

  for (let i = 0; i < len; i += EDGE_STRIDE) {
    // Read edge bounding box from header (start/end points)
    const sx = data[i],
      sy = data[i + 1],
      ex = data[i + 2],
      ey = data[i + 3];

    // Quick AABB cull: edge bounding box vs viewport
    const edgeLen = Math.abs(ex - sx) + Math.abs(ey - sy);
    const margin = edgeLen * 0.35;
    const minX = Math.min(sx, ex) - margin;
    const maxX = Math.max(sx, ex) + margin;
    const minY = Math.min(sy, ey) - margin;
    const maxY = Math.max(sy, ey) + margin;

    if (maxX < vpL || minX > vpR || maxY < vpT || minY > vpB) {
      continue;
    }

    // MoveTo (first curve's p0)
    c.moveTo(data[i + 4], data[i + 5]);

    // 5 curves, 6 floats each, starting at offset 6
    for (let ci = 0; ci < 5; ci++) {
      const base = i + 6 + ci * 6;
      c.bezierCurveTo(
        data[base],
        data[base + 1],
        data[base + 2],
        data[base + 3],
        data[base + 4],
        data[base + 5],
      );
    }
  }

  c.stroke();
}

function drawPuzzle(): void {
  if (!ctx || !edgesData || !borderData) return;

  const vpW = svgViewport.clientWidth;
  const vpH = svgViewport.clientHeight;

  // Clear
  ctx.clearRect(0, 0, vpW, vpH);

  // Compute transform: puzzle mm coords -> screen pixels
  const baseScale = vpW / puzzleWidth;
  const scale = baseScale * zoomLevel;

  // Viewport bounds in puzzle mm coordinates (for culling)
  const vpLeft = -panX / scale;
  const vpTop = -panY / scale;
  const vpRight = vpLeft + vpW / scale;
  const vpBottom = vpTop + vpH / scale;

  // Set up canvas transform
  ctx.save();
  ctx.translate(panX, panY);
  ctx.scale(scale, scale);

  // Style
  ctx.strokeStyle = "#000000";
  ctx.lineWidth = Math.max(1, 0.2 * scale) / scale;
  ctx.lineCap = "round";
  ctx.lineJoin = "round";

  // Draw border (always visible, small data)
  drawBorder(ctx);

  // Draw internal edges with viewport culling
  drawVisibleEdges(ctx, vpLeft, vpTop, vpRight, vpBottom);

  ctx.restore();
}

// ─── Zoom/Pan Helpers ───────────────────────────────────────

function applyTransform(): void {
  drawPuzzle();
  zoomLevelDisplay.textContent = `${Math.round(zoomLevel * 100)}%`;
}

function resetZoom(): void {
  zoomLevel = 1;
  panX = 0;
  panY = 0;
  if (canvas && puzzleWidth > 0) {
    const vpH = svgViewport.clientHeight;
    const baseScale = svgViewport.clientWidth / puzzleWidth;
    const svgH = puzzleHeight * baseScale;
    panY = Math.max(0, (vpH - svgH) / 2);
  }
  applyTransform();
}

// ─── rAF-Throttled Transform ─────────────────────────────────

let transformRafPending = false;
function scheduleTransform(): void {
  if (transformRafPending) return;
  transformRafPending = true;
  requestAnimationFrame(() => {
    transformRafPending = false;
    applyTransform();
  });
}

// ─── Throttled Generation ────────────────────────────────────

function scheduleGenerate(): void {
  if (rafPending) return;
  rafPending = true;
  requestAnimationFrame(() => {
    rafPending = false;
    generatePuzzle();
  });
}

// ─── Puzzle Generation ───────────────────────────────────────

function generatePuzzle(): void {
  const config = buildConfig();
  const configJson = JSON.stringify(config);

  // Generate binary edge data (also caches SVG internally for download)
  const result = generate_edges_binary(configJson);
  if (result && result.error) {
    try {
      errorDisplay.textContent = result.error || "Unknown error";
    } catch {
      errorDisplay.textContent = "Generation failed";
    }
    errorDisplay.style.display = "block";
    return;
  }

  edgesData = result.edges;
  borderData = result.border;
  puzzleWidth = result.width;
  puzzleHeight = result.height;

  errorDisplay.style.display = "none";

  // Compute piece breakdown in JS
  const rows = parseInt(rowsInput.value, 10);
  const cols = parseInt(colsInput.value, 10);
  const total = rows * cols;
  const corners = 4;
  const edges = 2 * (rows - 2) + 2 * (cols - 2);
  const interior = (rows - 2) * (cols - 2);
  pieceCount.textContent = `${total} pieces (${corners} corner, ${edges} edge, ${interior} interior)`;

  // Resize canvas and draw
  resizeCanvas();
  drawPuzzle();

  // Update URL with current params (debounced)
  scheduleURLUpdate();

  // Update ruler
  updateRuler();
}

// ─── Range Highlight ─────────────────────────────────────────

function updateRangeHighlight(
  minSlider: HTMLInputElement,
  maxSlider: HTMLInputElement,
  track: HTMLElement,
  active: boolean,
): void {
  if (!active) {
    track.style.setProperty("--range-bg", "#ddd");
    return;
  }
  const min = parseFloat(minSlider.min);
  const max = parseFloat(minSlider.max);
  const range = max - min || 1;
  const leftPct = ((parseFloat(minSlider.value) - min) / range) * 100;
  const rightPct = ((parseFloat(maxSlider.value) - min) / range) * 100;
  track.style.setProperty(
    "--range-bg",
    `linear-gradient(to right, #ddd ${leftPct}%, #4a90d9 ${leftPct}%, #4a90d9 ${rightPct}%, #ddd ${rightPct}%)`,
  );
}

// ─── Readout Updaters ───────────────────────────────────────

function updateReadouts(): void {
  if (tabRandomize.checked) {
    const minPct = Math.round(parseFloat(tabSlider.value) * 100);
    const maxPct = Math.round(parseFloat(tabMaxSlider.value) * 100);
    tabReadout.textContent = `${minPct}%-${maxPct}%`;
  } else {
    tabReadout.textContent = `${Math.round(parseFloat(tabSlider.value) * 100)}%`;
  }
  if (taperRandomize.checked) {
    taperReadout.textContent = `${parseFloat(taperSlider.value).toFixed(2)}-${parseFloat(taperMaxSlider.value).toFixed(2)}`;
  } else {
    taperReadout.textContent = parseFloat(taperSlider.value).toFixed(2);
  }
  radiusReadout.textContent = parseFloat(radiusSlider.value).toFixed(1);
  updateRangeHighlight(tabSlider, tabMaxSlider, tabTrack, tabRandomize.checked);
  updateRangeHighlight(taperSlider, taperMaxSlider, taperTrack, taperRandomize.checked);
}

// ─── Randomize Toggle Helpers ────────────────────────────────

function toggleRandomize(
  checkbox: HTMLInputElement,
  maxSlider: HTMLInputElement,
  minSlider: HTMLInputElement,
): void {
  if (checkbox.checked) {
    maxSlider.style.display = "";
    // Ensure max > min (at least one step apart)
    const step = parseFloat(minSlider.step) || 0.01;
    const minVal = parseFloat(minSlider.value);
    const maxVal = parseFloat(maxSlider.value);
    const sliderMax = parseFloat(maxSlider.max);
    if (maxVal <= minVal) {
      if (minVal + step <= sliderMax) {
        maxSlider.value = String(minVal + step);
      } else {
        minSlider.value = String(sliderMax - step);
        maxSlider.value = String(sliderMax);
      }
    }
  } else {
    maxSlider.style.display = "none";
  }
  updateReadouts();
  scheduleGenerate();
}

function clampMinMax(
  minSlider: HTMLInputElement,
  maxSlider: HTMLInputElement,
): void {
  const step = parseFloat(minSlider.step) || 0.01;
  const max = parseFloat(maxSlider.max);
  // Don't let min get so high that max can't stay one step above it
  const minCeiling = max - step;
  if (parseFloat(minSlider.value) > minCeiling) {
    minSlider.value = String(minCeiling);
  }
  if (parseFloat(maxSlider.value) <= parseFloat(minSlider.value)) {
    maxSlider.value = String(parseFloat(minSlider.value) + step);
  }
}

function clampMaxMin(
  minSlider: HTMLInputElement,
  maxSlider: HTMLInputElement,
): void {
  const step = parseFloat(minSlider.step) || 0.01;
  const min = parseFloat(minSlider.min);
  // Don't let max get so low that min can't stay one step below it
  const maxFloor = min + step;
  if (parseFloat(maxSlider.value) < maxFloor) {
    maxSlider.value = String(maxFloor);
  }
  if (parseFloat(minSlider.value) >= parseFloat(maxSlider.value)) {
    minSlider.value = String(parseFloat(maxSlider.value) - step);
  }
}

// ─── Unit Conversion ─────────────────────────────────────────

function convertDimensions(oldUnit: string, newUnit: string): void {
  if (oldUnit === newUnit) return;
  const factor = newUnit === "Inches" ? 1 / 25.4 : 25.4;
  const w = parseFloat(widthInput.value);
  const h = parseFloat(heightInput.value);
  if (!isNaN(w)) {
    widthInput.value = parseFloat((w * factor).toFixed(2)).toString();
  }
  if (!isNaN(h)) {
    heightInput.value = parseFloat((h * factor).toFixed(2)).toString();
  }
}

// ─── Piece Count / Grid Auto-calc ─────────────────────────────

function calcBestGrid(target: number): void {
  const w = parseFloat(widthInput.value);
  const h = parseFloat(heightInput.value);
  if (isNaN(w) || isNaN(h) || w <= 0 || h <= 0 || isNaN(target) || target < 4) return;

  let bestRows = 2;
  let bestCols = 2;
  let bestDist = Infinity;
  let bestAspectDiff = Infinity;

  const maxR = Math.min(target, 100);
  for (let r = 2; r <= maxR; r++) {
    let c = Math.round(target / r);
    c = Math.max(2, Math.min(100, c));
    // Skip grid ratios more extreme than 1:5
    const gridRatio = Math.max(r, c) / Math.min(r, c);
    if (gridRatio > 5) continue;

    const total = r * c;
    const dist = Math.abs(total - target);
    // Piece aspect ratio: (w/c) / (h/r) — want closest to 1
    const pieceAspect = (w / c) / (h / r);
    const aspectDiff = Math.abs(pieceAspect - 1);

    if (dist < bestDist || (dist === bestDist && aspectDiff < bestAspectDiff)) {
      bestRows = r;
      bestCols = c;
      bestDist = dist;
      bestAspectDiff = aspectDiff;
    }
  }

  rowsInput.value = String(bestRows);
  colsInput.value = String(bestCols);
  updateTabMax();
  updateReadouts();
  scheduleGenerate();
}

function syncPieceCount(): void {
  const rows = parseInt(rowsInput.value, 10);
  const cols = parseInt(colsInput.value, 10);
  if (!isNaN(rows) && !isNaN(cols)) {
    pieceTargetInput.value = String(rows * cols);
  }
}

function toggleLock(btn: HTMLElement, currentlyLocked: boolean, label: string): boolean {
  const next = !currentlyLocked;
  btn.innerHTML = next ? "&#128274;" : "&#128275;";
  btn.classList.toggle("locked", next);
  btn.title = next ? `Unlock ${label}` : `Lock ${label}`;
  return next;
}

function showWarnings(warnings: string[]): void {
  pieceSizeWarning.innerHTML = warnings.map((w) => `<li>${w}</li>`).join("");
}

function enforceConstraints(source: "grid" | "dims"): void {
  let rows = parseInt(rowsInput.value, 10);
  let cols = parseInt(colsInput.value, 10);
  let w = parseFloat(widthInput.value);
  let h = parseFloat(heightInput.value);
  if (isNaN(rows) || isNaN(cols) || isNaN(w) || isNaN(h) || rows < 2 || cols < 2 || w <= 0 || h <= 0) {
    pieceSizeWarning.innerHTML = "";
    return;
  }

  const factor = unitSelect.value === "Inches" ? 25.4 : 1;
  const warnings: string[] = [];
  let adjusted = false;

  if (source === "grid") {
    // User changed grid — adjust dimensions (if unlocked)
    const widthMM = w * factor;
    const heightMM = h * factor;
    const pieceW = widthMM / cols;
    const pieceH = heightMM / rows;
    const minDim = Math.min(pieceW, pieceH);

    if (minDim < 10) {
      if (dimsLocked) {
        const display = minDim < 1 ? minDim.toFixed(1) : String(Math.round(minDim));
        warnings.push(`Pieces are very small (~${display}mm). Unlock dimensions to auto-adjust.`);
      } else {
        // Scale up dimensions so smallest piece = 10mm
        const needW = cols * 10;
        const needH = rows * 10;
        const newWMM = Math.max(widthMM, needW);
        const newHMM = Math.max(heightMM, needH);
        const newW = newWMM / factor;
        const newH = newHMM / factor;
        widthInput.value = unitSelect.value === "Inches"
          ? parseFloat(newW.toFixed(2)).toString()
          : String(Math.round(newW));
        heightInput.value = unitSelect.value === "Inches"
          ? parseFloat(newH.toFixed(2)).toString()
          : String(Math.round(newH));
        adjusted = true;
      }
    }

    // Grid ratio check
    const gridRatio = Math.max(rows, cols) / Math.min(rows, cols);
    if (gridRatio > 5) {
      warnings.push(`Grid ratio ${rows}:${cols} is very extreme. Max recommended ratio is 1:5.`);
    }
  } else {
    // User changed dimensions — adjust grid (if unlocked)
    const widthMM = w * factor;
    const heightMM = h * factor;
    const pieceW = widthMM / cols;
    const pieceH = heightMM / rows;
    const minDim = Math.min(pieceW, pieceH);

    if (minDim < 10) {
      if (gridLocked) {
        const display = minDim < 1 ? minDim.toFixed(1) : String(Math.round(minDim));
        warnings.push(`Pieces are very small (~${display}mm). Unlock grid size to auto-adjust.`);
      } else {
        // Reduce grid so pieces >= 10mm
        const maxCols = Math.max(2, Math.floor(widthMM / 10));
        const maxRows = Math.max(2, Math.floor(heightMM / 10));
        if (cols > maxCols) {
          cols = maxCols;
          colsInput.value = String(cols);
          adjusted = true;
        }
        if (rows > maxRows) {
          rows = maxRows;
          rowsInput.value = String(rows);
          adjusted = true;
        }
        if (adjusted) {
          syncPieceCount();
        }
      }
    }

    // Grid ratio check after potential adjustment
    const gridRatio = Math.max(rows, cols) / Math.min(rows, cols);
    if (gridRatio > 5) {
      if (gridLocked) {
        warnings.push(`Grid ratio ${rows}:${cols} is very extreme. Unlock grid size to auto-adjust.`);
      } else {
        // Clamp the larger dimension to 5x the smaller
        if (rows > cols) {
          rows = Math.min(rows, cols * 5);
          rowsInput.value = String(rows);
        } else {
          cols = Math.min(cols, rows * 5);
          colsInput.value = String(cols);
        }
        syncPieceCount();
        adjusted = true;
      }
    }
  }

  showWarnings(warnings);

  if (adjusted) {
    updateTabMax();
    updateReadouts();
  }
}

// ─── Main ───────────────────────────────────────────────────

async function main(): Promise<void> {
  const loadingEl = document.getElementById("loading")!;
  const appEl = document.getElementById("app")!;

  try {
    await init();
    init_panic_hook();

    loadingEl.style.display = "none";
    appEl.style.display = "block";
  } catch (err) {
    loadingEl.textContent = `Failed to load WASM module: ${err}`;
    console.error("WASM init failed:", err);
    return;
  }

  // Cache DOM references
  rowsInput = document.getElementById("rows") as HTMLInputElement;
  colsInput = document.getElementById("cols") as HTMLInputElement;
  widthInput = document.getElementById("width") as HTMLInputElement;
  heightInput = document.getElementById("height") as HTMLInputElement;
  unitSelect = document.getElementById("unit") as HTMLSelectElement;
  tabSlider = document.getElementById("tab") as HTMLInputElement;
  taperSlider = document.getElementById("taper") as HTMLInputElement;
  radiusSlider = document.getElementById("radius") as HTMLInputElement;
  seedInput = document.getElementById("seed") as HTMLInputElement;
  pieceCount = document.getElementById("piece-count")!;
  errorDisplay = document.getElementById("error-display")!;

  tabReadout = document.getElementById("tab-readout")!;
  taperReadout = document.getElementById("taper-readout")!;
  radiusReadout = document.getElementById("radius-readout")!;
  tabRandomize = document.getElementById("tab-randomize") as HTMLInputElement;
  tabMaxSlider = document.getElementById("tab-max") as HTMLInputElement;
  taperRandomize = document.getElementById("taper-randomize") as HTMLInputElement;
  taperMaxSlider = document.getElementById("taper-max") as HTMLInputElement;
  tabTrack = document.getElementById("tab-track")!;
  taperTrack = document.getElementById("taper-track")!;

  pieceTargetInput = document.getElementById("piece-target") as HTMLInputElement;
  pieceSizeWarning = document.getElementById("piece-size-warning")!;
  gridLockBtn = document.getElementById("grid-lock")!;
  dimsLockBtn = document.getElementById("dims-lock")!;

  rulerWidth = document.getElementById("ruler-width")!;
  rulerHeight = document.getElementById("ruler-height")!;
  svgViewport = document.getElementById("svg-viewport")!;
  zoomLevelDisplay = document.getElementById("zoom-level")!;
  zoomInBtn = document.getElementById("zoom-in")!;
  zoomOutBtn = document.getElementById("zoom-out")!;
  zoomResetBtn = document.getElementById("zoom-reset")!;

  // Initialize canvas
  canvas = document.getElementById("puzzle-canvas") as HTMLCanvasElement;
  ctx = canvas.getContext("2d");

  // ResizeObserver for viewport resize
  const resizeObserver = new ResizeObserver(() => {
    resizeCanvas();
    drawPuzzle();
  });
  resizeObserver.observe(svgViewport);

  // Load params from URL (if any), otherwise generate random seed
  const hasUrlParams = loadFromURL();
  if (!hasUrlParams) {
    seedInput.value = randomSeed();
  }

  // Compute initial safe tab max and update slider readouts
  updateTabMax();
  updateReadouts();

  // Track previous unit for dimension conversion on unit change
  let previousUnit = unitSelect.value;

  // ─── Event Wiring ───────────────────────────────────────

  // Lock toggle buttons
  gridLockBtn.addEventListener("click", () => {
    gridLocked = toggleLock(gridLockBtn, gridLocked, "grid size");
  });
  dimsLockBtn.addEventListener("click", () => {
    dimsLocked = toggleLock(dimsLockBtn, dimsLocked, "dimensions");
  });

  // Grid inputs — rows/cols changed by user
  for (const input of [rowsInput, colsInput]) {
    input.addEventListener("input", () => {
      syncPieceCount();
      enforceConstraints("grid");
      updateTabMax();
      updateReadouts();
      scheduleGenerate();
    });
  }

  // Dimension inputs — width/height changed by user
  for (const input of [widthInput, heightInput]) {
    input.addEventListener("input", () => {
      enforceConstraints("dims");
      updateTabMax();
      updateReadouts();
      scheduleGenerate();
    });
  }

  // Piece count input — auto-calculate best grid
  pieceTargetInput.addEventListener("input", () => {
    const target = parseInt(pieceTargetInput.value, 10);
    if (!isNaN(target) && target >= 4) {
      calcBestGrid(target);
      syncPieceCount();
      enforceConstraints("grid");
    }
  });

  // Range sliders — update readout + regenerate
  const sliders = [tabSlider, taperSlider, radiusSlider];
  for (const slider of sliders) {
    slider.addEventListener("input", () => {
      // When randomize is on, clamp min <= max
      if (slider === tabSlider && tabRandomize.checked) {
        clampMinMax(tabSlider, tabMaxSlider);
      }
      if (slider === taperSlider && taperRandomize.checked) {
        clampMinMax(taperSlider, taperMaxSlider);
      }
      updateReadouts();
      scheduleGenerate();
    });
  }

  // Max sliders — clamp and regenerate
  tabMaxSlider.addEventListener("input", () => {
    clampMaxMin(tabSlider, tabMaxSlider);
    updateReadouts();
    scheduleGenerate();
  });
  taperMaxSlider.addEventListener("input", () => {
    clampMaxMin(taperSlider, taperMaxSlider);
    updateReadouts();
    scheduleGenerate();
  });

  // Randomize checkboxes
  tabRandomize.addEventListener("change", () => {
    toggleRandomize(tabRandomize, tabMaxSlider, tabSlider);
  });
  taperRandomize.addEventListener("change", () => {
    toggleRandomize(taperRandomize, taperMaxSlider, taperSlider);
  });

  // Unit select — convert dimensions and recalculate tab max
  unitSelect.addEventListener("change", () => {
    const newUnit = unitSelect.value;
    convertDimensions(previousUnit, newUnit);
    previousUnit = newUnit;
    updateTabMax();
    generatePuzzle();
    enforceConstraints("dims");
  });

  // Seed text input
  seedInput.addEventListener("input", scheduleGenerate);

  // Randomize button
  const randomizeBtn = document.getElementById("randomize")!;
  randomizeBtn.addEventListener("click", () => {
    seedInput.value = randomSeed();
    scheduleGenerate();
  });

  // ─── Zoom/Pan Event Wiring ──────────────────────────────

  // Wheel zoom — zoom toward cursor position
  svgViewport.addEventListener(
    "wheel",
    (e: WheelEvent) => {
      e.preventDefault();
      const rect = svgViewport.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const oldZoom = zoomLevel;
      if (e.deltaY < 0) {
        zoomLevel = Math.min(MAX_ZOOM, zoomLevel * ZOOM_STEP);
      } else {
        zoomLevel = Math.max(MIN_ZOOM, zoomLevel / ZOOM_STEP);
      }

      // Adjust pan so zoom centers on cursor
      const zoomRatio = zoomLevel / oldZoom;
      panX = mouseX - zoomRatio * (mouseX - panX);
      panY = mouseY - zoomRatio * (mouseY - panY);

      scheduleTransform();
    },
    { passive: false },
  );

  // Mouse drag pan
  svgViewport.addEventListener("mousedown", (e: MouseEvent) => {
    if (e.button !== 0) return; // left click only
    isPanning = true;
    panStartX = e.clientX - panX;
    panStartY = e.clientY - panY;
    e.preventDefault();
  });

  window.addEventListener("mousemove", (e: MouseEvent) => {
    if (!isPanning) return;
    panX = e.clientX - panStartX;
    panY = e.clientY - panStartY;
    scheduleTransform();
  });

  window.addEventListener("mouseup", () => {
    isPanning = false;
  });

  // Double-click to reset zoom
  svgViewport.addEventListener("dblclick", () => {
    resetZoom();
  });

  // Zoom button handlers
  zoomInBtn.addEventListener("click", () => {
    const rect = svgViewport.getBoundingClientRect();
    const cx = rect.width / 2;
    const cy = rect.height / 2;
    const oldZoom = zoomLevel;
    zoomLevel = Math.min(MAX_ZOOM, zoomLevel * ZOOM_STEP);
    const zoomRatio = zoomLevel / oldZoom;
    panX = cx - zoomRatio * (cx - panX);
    panY = cy - zoomRatio * (cy - panY);
    applyTransform();
  });

  zoomOutBtn.addEventListener("click", () => {
    const rect = svgViewport.getBoundingClientRect();
    const cx = rect.width / 2;
    const cy = rect.height / 2;
    const oldZoom = zoomLevel;
    zoomLevel = Math.max(MIN_ZOOM, zoomLevel / ZOOM_STEP);
    const zoomRatio = zoomLevel / oldZoom;
    panX = cx - zoomRatio * (cx - panX);
    panY = cy - zoomRatio * (cy - panY);
    applyTransform();
  });

  zoomResetBtn.addEventListener("click", () => {
    resetZoom();
  });

  // Touch support — pinch zoom and drag
  let lastTouchDist = 0;

  svgViewport.addEventListener(
    "touchstart",
    (e: TouchEvent) => {
      if (e.touches.length === 1) {
        isPanning = true;
        panStartX = e.touches[0].clientX - panX;
        panStartY = e.touches[0].clientY - panY;
      } else if (e.touches.length === 2) {
        isPanning = false;
        const dx = e.touches[0].clientX - e.touches[1].clientX;
        const dy = e.touches[0].clientY - e.touches[1].clientY;
        lastTouchDist = Math.sqrt(dx * dx + dy * dy);
      }
    },
    { passive: true },
  );

  svgViewport.addEventListener(
    "touchmove",
    (e: TouchEvent) => {
      e.preventDefault();
      if (e.touches.length === 1 && isPanning) {
        panX = e.touches[0].clientX - panStartX;
        panY = e.touches[0].clientY - panStartY;
        scheduleTransform();
      } else if (e.touches.length === 2) {
        const dx = e.touches[0].clientX - e.touches[1].clientX;
        const dy = e.touches[0].clientY - e.touches[1].clientY;
        const dist = Math.sqrt(dx * dx + dy * dy);
        const midX = (e.touches[0].clientX + e.touches[1].clientX) / 2;
        const midY = (e.touches[0].clientY + e.touches[1].clientY) / 2;
        const rect = svgViewport.getBoundingClientRect();

        if (lastTouchDist > 0) {
          const oldZoom = zoomLevel;
          zoomLevel = Math.max(
            MIN_ZOOM,
            Math.min(MAX_ZOOM, zoomLevel * (dist / lastTouchDist)),
          );
          const zoomRatio = zoomLevel / oldZoom;
          const cx = midX - rect.left;
          const cy = midY - rect.top;
          panX = cx - zoomRatio * (cx - panX);
          panY = cy - zoomRatio * (cy - panY);
          scheduleTransform();
        }

        lastTouchDist = dist;
      }
    },
    { passive: false },
  );

  svgViewport.addEventListener("touchend", () => {
    isPanning = false;
    lastTouchDist = 0;
  });

  // ─── Download SVG ──────────────────────────────────────

  const downloadBtn = document.getElementById("download")!;
  downloadBtn.addEventListener("click", () => {
    const svgContent = get_cached_svg();
    if (!svgContent || !svgContent.startsWith("<svg")) return;
    const config = buildConfig() as Record<string, unknown>;
    const filename = `puzzle-${config.rows}x${config.cols}-seed-${config.seed}.svg`;
    const blob = new Blob([svgContent], { type: "image/svg+xml" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  });

  // ─── Copy Link ─────────────────────────────────────────

  const copyLinkBtn = document.getElementById("copy-link")!;
  copyLinkBtn.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(window.location.href);
      const original = copyLinkBtn.textContent;
      copyLinkBtn.textContent = "Copied!";
      setTimeout(() => {
        copyLinkBtn.textContent = original;
      }, 1500);
    } catch {
      // Fallback for non-HTTPS contexts
      const input = document.createElement("input");
      input.value = window.location.href;
      document.body.appendChild(input);
      input.select();
      document.execCommand("copy");
      document.body.removeChild(input);
      const original = copyLinkBtn.textContent;
      copyLinkBtn.textContent = "Copied!";
      setTimeout(() => {
        copyLinkBtn.textContent = original;
      }, 1500);
    }
  });

  // ─── Initial Generate ─────────────────────────────────────

  syncPieceCount();
  enforceConstraints("grid");
  generatePuzzle();
  resetZoom();
}

main();
