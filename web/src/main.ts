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
let kerfSlider: HTMLInputElement;
let seedInput: HTMLInputElement;
let svgContainer: HTMLElement;
let pieceCount: HTMLElement;
let errorDisplay: HTMLElement;

let tabReadout: HTMLElement;
let taperReadout: HTMLElement;
let radiusReadout: HTMLElement;
let kerfReadout: HTMLElement;

// ─── Config Builder ─────────────────────────────────────────

function buildConfig(): object {
  return {
    rows: parseInt(rowsInput.value, 10),
    cols: parseInt(colsInput.value, 10),
    width: parseFloat(widthInput.value),
    height: parseFloat(heightInput.value),
    unit: unitSelect.value,
    tab: { size_pct: parseFloat(tabSlider.value), taper: parseFloat(taperSlider.value) },
    border: { corner_radius: parseFloat(radiusSlider.value) },
    seed: seedInput.value,
    kerf_width: parseFloat(kerfSlider.value),
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
  const tab = parseInt(params.get("tab") ?? "25", 10) / 100;
  const radius = parseFloat(params.get("radius") ?? "2");
  const taperRaw = parseInt(params.get("taper") ?? "50", 10) / 100;
  const taper = Math.max(0.30, Math.min(1.10, taperRaw)); // clamp to valid range
  const kerf = parseFloat(params.get("kerf") ?? "0");
  const seed = params.get("seed") ?? "";

  rowsInput.value = String(rows);
  colsInput.value = String(cols);
  widthInput.value = String(w);
  heightInput.value = String(h);
  unitSelect.value = unit;
  tabSlider.value = String(tab);
  taperSlider.value = String(taper);
  radiusSlider.value = String(radius);
  kerfSlider.value = String(kerf);
  seedInput.value = seed || randomSeed();

  return true;
}

function updateURL(): void {
  const config = buildConfig() as Record<string, unknown>;
  const tabObj = config.tab as { size_pct: number; taper: number };
  const borderObj = config.border as { corner_radius: number };
  const params = new URLSearchParams();
  params.set("rows", String(config.rows));
  params.set("cols", String(config.cols));
  params.set("w", String(config.width));
  params.set("h", String(config.height));
  params.set("unit", config.unit === "Inches" ? "in" : "mm");
  params.set("tab", String(Math.round(tabObj.size_pct * 100)));
  params.set("taper", String(Math.round(tabObj.taper * 100)));
  params.set("radius", String(borderObj.corner_radius));
  params.set("kerf", String(config.kerf_width));
  params.set("seed", String(config.seed));
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
      // Clamp current value if it exceeds new max
      if (parseFloat(tabSlider.value) > max) {
        tabSlider.value = String(max);
      }
    }
  } catch {
    // Fallback: keep current max
  }
}

// ─── SVG Generation ─────────────────────────────────────────

function generatePuzzle(): void {
  const config = buildConfig();
  const configJson = JSON.stringify(config);

  // Generate SVG
  const svgResult = generate_svg(configJson);
  if (svgResult.startsWith("<svg")) {
    svgContainer.innerHTML = svgResult;
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
}

// ─── Readout Updaters ───────────────────────────────────────

function updateReadouts(): void {
  tabReadout.textContent = `${Math.round(parseFloat(tabSlider.value) * 100)}%`;
  taperReadout.textContent = parseFloat(taperSlider.value).toFixed(2);
  radiusReadout.textContent = parseFloat(radiusSlider.value).toFixed(1);
  kerfReadout.textContent = parseFloat(kerfSlider.value).toFixed(2);
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
  kerfSlider = document.getElementById("kerf") as HTMLInputElement;
  seedInput = document.getElementById("seed") as HTMLInputElement;
  svgContainer = document.getElementById("svg-container")!;
  pieceCount = document.getElementById("piece-count")!;
  errorDisplay = document.getElementById("error-display")!;

  tabReadout = document.getElementById("tab-readout")!;
  taperReadout = document.getElementById("taper-readout")!;
  radiusReadout = document.getElementById("radius-readout")!;
  kerfReadout = document.getElementById("kerf-readout")!;

  // Load params from URL (if any), otherwise generate random seed
  const hasUrlParams = loadFromURL();
  if (!hasUrlParams) {
    seedInput.value = randomSeed();
  }

  // Compute initial safe tab max and update slider readouts
  updateTabMax();
  updateReadouts();

  // ─── Event Wiring ───────────────────────────────────────

  // Number inputs — instant regeneration + recalculate tab max
  const numberInputs = [rowsInput, colsInput, widthInput, heightInput];
  for (const input of numberInputs) {
    input.addEventListener("input", () => {
      updateTabMax();
      updateReadouts();
      generatePuzzle();
    });
  }

  // Range sliders — update readout + regenerate
  const sliders = [tabSlider, taperSlider, radiusSlider, kerfSlider];
  for (const slider of sliders) {
    slider.addEventListener("input", () => {
      updateReadouts();
      generatePuzzle();
    });
  }

  // Unit select — also recalculate tab max (mm vs inches changes cell dimensions)
  unitSelect.addEventListener("change", () => {
    updateTabMax();
    generatePuzzle();
  });

  // Seed text input
  seedInput.addEventListener("input", generatePuzzle);

  // Randomize button
  const randomizeBtn = document.getElementById("randomize")!;
  randomizeBtn.addEventListener("click", () => {
    seedInput.value = randomSeed();
    generatePuzzle();
  });

  // ─── Download SVG ──────────────────────────────────────

  const downloadBtn = document.getElementById("download")!;
  downloadBtn.addEventListener("click", () => {
    const svgContent = svgContainer.innerHTML;
    if (!svgContent) return;
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

  generatePuzzle();
}

main();
