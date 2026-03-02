import init, {
  compute_pieces,
  init_panic_hook,
} from "../../crates/puzzle-wasm/pkg";
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

type ComputeResult = PieceBreakdown | ErrorResponse;

function isError(result: ComputeResult): result is ErrorResponse {
  return "error" in result;
}

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

  const form = document.getElementById("form") as HTMLFormElement;
  const rowsInput = document.getElementById("rows") as HTMLInputElement;
  const colsInput = document.getElementById("cols") as HTMLInputElement;
  const resultEl = document.getElementById("result")!;

  form.addEventListener("submit", (e) => {
    e.preventDefault();

    const rows = parseInt(rowsInput.value, 10);
    const cols = parseInt(colsInput.value, 10);

    const configJson = JSON.stringify({ rows, cols });
    const responseJson = compute_pieces(configJson);
    const result: ComputeResult = JSON.parse(responseJson);

    if (isError(result)) {
      resultEl.innerHTML = `<p class="error">${result.error}</p>`;
    } else {
      resultEl.innerHTML = `
        <dl class="breakdown">
          <dt>Total</dt>
          <dd>${result.total}</dd>
          <dt>Corners</dt>
          <dd>${result.corners}</dd>
          <dt>Edges</dt>
          <dd>${result.edges}</dd>
          <dt>Interior</dt>
          <dd>${result.interior}</dd>
        </dl>
      `;
    }
  });
}

main();
