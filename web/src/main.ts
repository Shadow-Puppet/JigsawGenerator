import init, {
  generate_svg,
  compute_pieces,
  init_panic_hook,
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
let jitterSlider: HTMLInputElement;
let radiusSlider: HTMLInputElement;
let kerfSlider: HTMLInputElement;
let seedInput: HTMLInputElement;
let svgContainer: HTMLElement;
let pieceCount: HTMLElement;
let errorDisplay: HTMLElement;

let tabReadout: HTMLElement;
let jitterReadout: HTMLElement;
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
    tab: { size_pct: parseFloat(tabSlider.value) },
    jitter: { amount: parseFloat(jitterSlider.value) },
    border: { corner_radius: parseFloat(radiusSlider.value) },
    seed: seedInput.value,
    kerf_width: parseFloat(kerfSlider.value),
  };
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
}

// ─── Readout Updaters ───────────────────────────────────────

function updateReadouts(): void {
  tabReadout.textContent = `${Math.round(parseFloat(tabSlider.value) * 100)}%`;
  jitterReadout.textContent = parseFloat(jitterSlider.value).toFixed(2);
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
  jitterSlider = document.getElementById("jitter") as HTMLInputElement;
  radiusSlider = document.getElementById("radius") as HTMLInputElement;
  kerfSlider = document.getElementById("kerf") as HTMLInputElement;
  seedInput = document.getElementById("seed") as HTMLInputElement;
  svgContainer = document.getElementById("svg-container")!;
  pieceCount = document.getElementById("piece-count")!;
  errorDisplay = document.getElementById("error-display")!;

  tabReadout = document.getElementById("tab-readout")!;
  jitterReadout = document.getElementById("jitter-readout")!;
  radiusReadout = document.getElementById("radius-readout")!;
  kerfReadout = document.getElementById("kerf-readout")!;

  // Initialize slider readouts from default values
  updateReadouts();

  // Generate random seed for initial load
  seedInput.value = randomSeed();

  // ─── Event Wiring ───────────────────────────────────────

  // Number inputs — instant regeneration
  const numberInputs = [rowsInput, colsInput, widthInput, heightInput];
  for (const input of numberInputs) {
    input.addEventListener("input", generatePuzzle);
  }

  // Range sliders — update readout + regenerate
  const sliders = [tabSlider, jitterSlider, radiusSlider, kerfSlider];
  for (const slider of sliders) {
    slider.addEventListener("input", () => {
      updateReadouts();
      generatePuzzle();
    });
  }

  // Unit select
  unitSelect.addEventListener("change", generatePuzzle);

  // Seed text input
  seedInput.addEventListener("input", generatePuzzle);

  // Randomize button
  const randomizeBtn = document.getElementById("randomize")!;
  randomizeBtn.addEventListener("click", () => {
    seedInput.value = randomSeed();
    generatePuzzle();
  });

  // ─── Initial Generate ─────────────────────────────────────

  generatePuzzle();
}

main();
