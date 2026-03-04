import init, {
  generate_svg,
  compute_pieces,
  init_panic_hook,
  safe_tab_max,
} from "puzzle-wasm";
import "./style.css";

interface PieceBreakdown {
  total: number;
  corners: number;
  edges: number;
  interior: number;
}

interface ErrorResponse {
  error: string;
}

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
let svgContainer: HTMLElement;
let pieceCount: HTMLElement;
let errorDisplay: HTMLElement;

let tabReadout: HTMLElement;
let taperReadout: HTMLElement;
let radiusReadout: HTMLElement;
let tabRandomize: HTMLInputElement;
let tabMaxSlider: HTMLInputElement;
let taperRandomize: HTMLInputElement;
let taperMaxSlider: HTMLInputElement;

let pieceTargetInput: HTMLInputElement;
let pieceSizeWarning: HTMLElement;

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

const MIN_ZOOM = 0.5;
const MAX_ZOOM = 20;
const ZOOM_STEP = 1.15; // 15% per wheel tick

// ─── Config Builder ─────────────────────────────────────────

function buildConfig(): object {
  const tabConfig: Record<string, unknown> = {
    size_pct: parseFloat(tabSlider.value),
    taper: 0.5 + parseFloat(taperSlider.value) * 0.7,
  };
  if (tabRandomize.checked) {
    tabConfig.size_pct_max = parseFloat(tabMaxSlider.value);
  }
  if (taperRandomize.checked) {
    tabConfig.taper_max = 0.5 + parseFloat(taperMaxSlider.value) * 0.7;
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

// ─── Dynamic Tab Size Clamping ───────────────────────────────

function updateTabMax(): void {
  const config = buildConfig();
  const configJson = JSON.stringify(config);
  try {
    const result = JSON.parse(safe_tab_max(configJson));
    if (result.max) {
      const max = Math.round(result.max * 100) / 100; // round to 2 decimals
      tabSlider.max = String(max);
      tabMaxSlider.max = String(max);
      // Clamp current value if it exceeds new max
      if (parseFloat(tabSlider.value) > max) {
        tabSlider.value = String(max);
      }
      if (parseFloat(tabMaxSlider.value) > max) {
        tabMaxSlider.value = String(max);
      }
    }
  } catch {
    // Fallback: keep current max
  }
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

// ─── Zoom/Pan Helpers ───────────────────────────────────────

function applyTransform(): void {
  svgContainer.style.transform = `translate(${panX}px, ${panY}px) scale(${zoomLevel})`;
  zoomLevelDisplay.textContent = `${Math.round(zoomLevel * 100)}%`;
}

function resetZoom(): void {
  // First reset transform so we can measure natural SVG height
  zoomLevel = 1;
  panX = 0;
  panY = 0;
  svgContainer.style.transform = "translate(0px, 0px) scale(1)";

  // Vertically center: offset by half the difference between viewport and SVG height
  const svgEl = svgContainer.querySelector("svg");
  if (svgEl && svgViewport) {
    const viewportH = svgViewport.clientHeight;
    const svgH = svgEl.getBoundingClientRect().height;
    panY = Math.max(0, (viewportH - svgH) / 2);
  }
  applyTransform();
}

// ─── SVG Generation ─────────────────────────────────────────

function generatePuzzle(): void {
  const config = buildConfig();
  const configJson = JSON.stringify(config);

  // Generate SVG
  const svgResult = generate_svg(configJson);
  if (svgResult.startsWith("<svg")) {
    svgContainer.innerHTML = svgResult;

    // Normalize SVG: remove fixed width/height, ensure viewBox fills container
    const svgEl = svgContainer.querySelector("svg");
    if (svgEl) {
      const wAttr = svgEl.getAttribute("width");
      const hAttr = svgEl.getAttribute("height");
      // If no viewBox, create one from width/height before removing them
      if (!svgEl.getAttribute("viewBox") && wAttr && hAttr) {
        const numW = parseFloat(wAttr);
        const numH = parseFloat(hAttr);
        if (!isNaN(numW) && !isNaN(numH)) {
          svgEl.setAttribute("viewBox", `0 0 ${numW} ${numH}`);
        }
      }
      svgEl.removeAttribute("width");
      svgEl.removeAttribute("height");
    }

    errorDisplay.style.display = "none";

    // Also get piece breakdown
    const piecesResult = compute_pieces(configJson);
    try {
      const parsed: PieceBreakdown | ErrorResponse = JSON.parse(piecesResult);
      if (!("error" in parsed)) {
        const p = parsed as PieceBreakdown;
        pieceCount.textContent = `${p.total} pieces (${p.corners} corner, ${p.edges} edge, ${p.interior} interior)`;
      }
    } catch {
      // Ignore piece count parse errors — SVG is still valid
    }
  } else {
    // Error — keep previous SVG visible, show error message
    try {
      const err: ErrorResponse = JSON.parse(svgResult);
      errorDisplay.textContent = err.error || "Unknown error";
    } catch {
      errorDisplay.textContent = "SVG generation failed";
    }
    errorDisplay.style.display = "block";
  }

  // Update URL with current params (replaceState — no history spam)
  updateURL();

  // Update ruler (zoom/pan state preserved across regenerations)
  updateRuler();
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
}

// ─── Randomize Toggle Helpers ────────────────────────────────

function toggleRandomize(
  checkbox: HTMLInputElement,
  maxSlider: HTMLInputElement,
  minSlider: HTMLInputElement
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
  generatePuzzle();
}

function clampMinMax(
  minSlider: HTMLInputElement,
  maxSlider: HTMLInputElement
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
  maxSlider: HTMLInputElement
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
  generatePuzzle();
}

function syncPieceCount(): void {
  const rows = parseInt(rowsInput.value, 10);
  const cols = parseInt(colsInput.value, 10);
  if (!isNaN(rows) && !isNaN(cols)) {
    pieceTargetInput.value = String(rows * cols);
  }
}

function checkPieceSize(): void {
  const rows = parseInt(rowsInput.value, 10);
  const cols = parseInt(colsInput.value, 10);
  const w = parseFloat(widthInput.value);
  const h = parseFloat(heightInput.value);
  if (isNaN(rows) || isNaN(cols) || isNaN(w) || isNaN(h) || rows < 1 || cols < 1) {
    pieceSizeWarning.textContent = "";
    return;
  }

  const factor = unitSelect.value === "Inches" ? 25.4 : 1;
  const widthMM = w * factor;
  const heightMM = h * factor;
  const pieceW = widthMM / cols;
  const pieceH = heightMM / rows;
  const minDim = Math.min(pieceW, pieceH);

  if (minDim < 10) {
    const display = minDim < 1 ? minDim.toFixed(1) : String(Math.round(minDim));
    pieceSizeWarning.textContent = `Pieces are very small (~${display}mm). May be difficult to cut/handle.`;
  } else {
    pieceSizeWarning.textContent = "";
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
  svgContainer = document.getElementById("svg-container")!;
  pieceCount = document.getElementById("piece-count")!;
  errorDisplay = document.getElementById("error-display")!;

  tabReadout = document.getElementById("tab-readout")!;
  taperReadout = document.getElementById("taper-readout")!;
  radiusReadout = document.getElementById("radius-readout")!;
  tabRandomize = document.getElementById("tab-randomize") as HTMLInputElement;
  tabMaxSlider = document.getElementById("tab-max") as HTMLInputElement;
  taperRandomize = document.getElementById("taper-randomize") as HTMLInputElement;
  taperMaxSlider = document.getElementById("taper-max") as HTMLInputElement;

    pieceTargetInput = document.getElementById("piece-target") as HTMLInputElement;
    pieceSizeWarning = document.getElementById("piece-size-warning")!;

    rulerWidth = document.getElementById("ruler-width")!;
    rulerHeight = document.getElementById("ruler-height")!;
  svgViewport = document.getElementById("svg-viewport")!;
  zoomLevelDisplay = document.getElementById("zoom-level")!;
  zoomInBtn = document.getElementById("zoom-in")!;
  zoomOutBtn = document.getElementById("zoom-out")!;
  zoomResetBtn = document.getElementById("zoom-reset")!;

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

    // Number inputs — instant regeneration + recalculate tab max
    const numberInputs = [rowsInput, colsInput, widthInput, heightInput];
    for (const input of numberInputs) {
      input.addEventListener("input", () => {
        updateTabMax();
        updateReadouts();
        generatePuzzle();
        // Sync piece count when rows/cols change; check piece size on any dimension change
        if (input === rowsInput || input === colsInput) {
          syncPieceCount();
        }
        checkPieceSize();
      });
    }

    // Piece count input — auto-calculate best grid
    pieceTargetInput.addEventListener("input", () => {
      const target = parseInt(pieceTargetInput.value, 10);
      if (!isNaN(target) && target >= 4) {
        calcBestGrid(target);
        syncPieceCount(); // Update to show actual total (may differ from target)
        checkPieceSize();
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
      generatePuzzle();
    });
  }

  // Max sliders — clamp and regenerate
  tabMaxSlider.addEventListener("input", () => {
    clampMaxMin(tabSlider, tabMaxSlider);
    updateReadouts();
    generatePuzzle();
  });
  taperMaxSlider.addEventListener("input", () => {
    clampMaxMin(taperSlider, taperMaxSlider);
    updateReadouts();
    generatePuzzle();
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
      checkPieceSize();
    });

  // Seed text input
  seedInput.addEventListener("input", generatePuzzle);

  // Randomize button
  const randomizeBtn = document.getElementById("randomize")!;
  randomizeBtn.addEventListener("click", () => {
    seedInput.value = randomSeed();
    generatePuzzle();
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

      applyTransform();
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
    applyTransform();
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
        applyTransform();
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
          applyTransform();
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
    // Re-generate SVG for download (with original physical dimensions from WASM)
    const config = buildConfig();
    const configJson = JSON.stringify(config);
    const svgContent = generate_svg(configJson);
    if (!svgContent.startsWith("<svg")) return;
    const configObj = config as Record<string, unknown>;
    const filename = `puzzle-${configObj.rows}x${configObj.cols}-seed-${configObj.seed}.svg`;
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

    syncPieceCount(); // Populate piece count from current rows * cols
    checkPieceSize(); // Check initial piece dimensions
    generatePuzzle();
    resetZoom(); // Center vertically on first load
}

main();
