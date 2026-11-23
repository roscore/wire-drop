# Task: Robot DC power harness wire sizer (Rust → WebAssembly)

Build an original, small, public static web app named **Wire Drop**. Write original code and assets only.
It helps robotics developers answer: "Which wire gauge keeps the voltage drop, heating and power loss of
this DC harness (battery → distribution → motor drivers / compute) within budget?"

## Product scope
One core feature plus two small extras; target roughly 400–800 lines of Rust and 4–8 unit tests.

### Core feature: harness run calculator + comparison
- Single-page Korean-first UI with an English toggle.
- Inputs: supply voltage (V, e.g. 48 V), allowed drop (% of supply), conductor material (copper / tinned
  copper / aluminium), conductor temperature (°C, resistivity corrected with alpha), and a list of runs.
- Each run: editable name, one-way length (m), continuous current (A), peak current (A) and peak duty
  (seconds), round trip (2 conductors) toggle, selected AWG (30…0) or mm² size.
- Built-in table: AWG 30–0 and metric 0.25–50 mm² cross-sections; compute resistance per metre from area
  and resistivity (not hard-coded ohm tables), plus a conservative chassis-wiring ampacity estimate
  labelled "approx., check your cable datasheet".
- Compute and show per run: resistance, voltage drop at continuous and peak current (V and %), voltage at
  the load, I²R power loss (W), current density (A/mm²), status pass/warn/fail versus the drop budget and
  ampacity, and the smallest gauge that passes both.
- Hero visualization: an app-drawn (Canvas 2D or SVG, no chart library) horizontal "voltage ladder":
  supply rail at the left, each run as a coloured bar whose thickness ∝ wire area and whose right end
  drops by the computed voltage, with a dashed minimum-voltage line, hover/focus tooltip, and a HUD
  with total loss, worst run and status. Plus a small plot of drop % vs gauge for the selected run with
  the budget line and current choice marked.
- Explain that it is a steady-state DC estimate: connector/crimp resistance, bundling derating, ambient,
  insulation rating and transient inductive effects need measurement and datasheets.

### Small extras
1. Three synthetic presets: "48 V humanoid leg", "24 V mobile base", "12 V sensor/compute tree".
   Label them illustrative and editable.
2. Export the current table as a local CSV download. No import, sharing, accounts or storage.

### Polished UI
- Dark design tokens, system fonts, subtle gradient background, full-width visualization hero, slim
  toolbar, compact collapsible settings cards, colour chips per run, 150–200 ms transitions, mobile
  bottom-sheet layout, dismissible first-run hint. Keyboard accessible, visible focus, reduced-motion.
- The initial example must be useful immediately. No 3D needed.

## Architecture and tests
- Workspace with `crates/wd_core` (pure Rust) and `crates/wd_web` (wasm-bindgen/web-sys/js-sys only).
- 4–8 hand-checkable tests: AWG area formula (e.g. AWG 10 ≈ 5.26 mm²), copper resistance at 20 °C and
  temperature correction, round-trip drop, I²R loss, smallest-passing-gauge search, CSV escaping.

## Repository / release requirements
- `README.md`: Korean first, then English; what it does, model/limitations, controls, local build,
  privacy, MIT.
- MIT `LICENSE`, copyright `wire-drop contributors`. `.gitignore` for target and dist.
- `Trunk.toml` with `public_url = "/wire-drop/"`, `filehash = false`.
- `index.html` CSP: `default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self'; object-src 'none'; base-uri 'self'`.
- `.github/workflows/pages.yml` (already present, official Pages actions) and `tools/privacy_check.sh`
  (already present) must keep working; the privacy check must pass.
- Commit identity: `wire-drop contributors <59640857+roscore@users.noreply.github.com>`.
- Do not push to main. Commit the implementation on a branch and open a PR.

## Done when
- `cargo test` passes with 4–8 tests; `trunk build --release` succeeds; `bash tools/privacy_check.sh` clean.
- App works at the `/wire-drop/` subpath with no external runtime requests.
- README (KO + EN), LICENSE, Pages workflow present. A PR is open; do not change repo settings.
