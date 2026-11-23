# Wire Drop

[한국어](#한국어) · [English](#english)

## 한국어

**Wire Drop**은 로봇 DC 전원 하네스(배터리 → 분배 보드 → 모터 드라이버 / 컴퓨트)의 전선 굵기를
고르는 작은 정적 웹 앱입니다. Rust → WebAssembly로 빌드되며 브라우저 안에서만 동작합니다.

### 기능

- 공급 전압, 허용 전압 강하(%), 도체 재질(구리 / 주석 도금 구리 / 알루미늄), 도체 온도를 설정합니다.
- 구간(run)마다 이름, 편도 길이, 연속/피크 전류, 피크 지속 시간, 왕복(2선) 여부, AWG(30…0) 또는 mm²
  굵기를 입력합니다.
- 구간별 저항, 연속·피크 전압 강하(V, %), 부하 전압, I²R 손실, 전류 밀도, 허용 전류 근사치, 통과/주의/실패
  상태, 그리고 두 조건을 모두 만족하는 **가장 가는 굵기**를 보여줍니다.
- **전압 사다리** 시각화: 왼쪽 공급 레일에서 시작해 각 구간 막대(두께 ∝ 단면적)의 오른쪽 끝이 전압
  강하만큼 내려갑니다. 점선은 최소 허용 전압, HUD에는 총 손실·최악 구간·전체 상태가 표시됩니다.
- 선택 구간의 굵기별 전압 강하(%) 그래프(예산선, 현재 선택, 최소 통과 굵기 표시).
- 예시 프리셋 3종: "48 V humanoid leg", "24 V mobile base", "12 V sensor/compute tree" (가상의 예시이며 수정 가능).
- 현재 표를 로컬 CSV 파일로 내보내기.

### 모델과 한계

- 정상 상태 DC 추정치입니다. `R = ρ(T) · L · n / A`, `ρ(T) = ρ20 · (1 + α · (T − 20 °C))`.
- AWG 단면적은 `d = 0.127 mm · 92^((36 − n) / 39)` 공식에서 계산하며, 저항표를 하드코딩하지 않습니다.
- ρ20 (Ω·mm²/m): 구리 0.01724, 주석 도금 구리 0.01793, 알루미늄 0.0282. α: 0.00393 / 0.00393 / 0.00403.
- 허용 전류는 보수적인 섀시 배선 추정치(`9 · A^0.8` A, 재질 보정)이며 **approx., check your cable datasheet**.
  피크는 `√(60 s / 지속시간)` 배(1–3배)까지 허용합니다.
- 상태: 연속 강하 > 예산, 연속 전류 > 허용 전류, 또는 피크 전류 > 피크 허용치이면 실패. 피크 강하 > 예산
  또는 연속 전류 > 허용 전류의 80 %이면 주의.
- 커넥터·크림프 저항, 묶음 디레이팅, 주변 온도, 절연 등급, 과도 유도성 효과는 반영하지 않습니다.
  실측과 케이블/커넥터 데이터시트로 반드시 확인하세요.

### 조작

- 표의 값을 바꾸면 즉시 다시 계산됩니다. "적용" 버튼은 최소 통과 굵기를 해당 구간에 적용합니다.
- 사다리를 클릭하거나, 포커스(Tab) 후 ↑/↓ 키로 구간을 선택합니다. 마우스를 올리면 툴팁이 나타납니다.
- 툴바: 프리셋 선택, 구간 추가, CSV 내보내기, EN/한국어 전환. 모바일에서는 설정이 하단 시트로 열립니다.

### 로컬 빌드

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked --version 0.22.2
cargo test
trunk build --release        # dist/ 에 /wire-drop/ 하위 경로용 결과물 생성
trunk serve                  # 로컬 미리보기: localhost:8080/wire-drop/
bash tools/privacy_check.sh
```

### 개인정보

분석 도구, 추적기, 외부 폰트/CDN, 원격 측정, 서버가 없습니다. 실행 중 네트워크 요청을 하지 않으며,
입력값은 브라우저 밖으로 나가지 않고 저장되지도 않습니다.

### 라이선스

MIT — `LICENSE` 참고.

## English

**Wire Drop** is a small static web app that helps pick wire gauges for a robot DC power harness
(battery → distribution → motor drivers / compute). It is built with Rust → WebAssembly and runs
entirely in the browser.

### What it does

- Global settings: supply voltage, allowed drop (% of supply), conductor material (copper / tinned
  copper / aluminium) and conductor temperature.
- Per run: name, one-way length, continuous and peak current, peak duty (s), round trip (2 conductors),
  and an AWG (30…0) or mm² size.
- Per run results: resistance, continuous and peak voltage drop (V and %), voltage at the load, I²R loss,
  current density, approximate ampacity, pass/warn/fail status, and the **smallest gauge that passes**.
- **Voltage ladder** hero: the supply rail on the left, each run as a bar whose thickness is
  proportional to wire area and whose right end drops by the computed voltage, a dashed minimum-voltage
  line, hover/focus tooltip and a HUD with total loss, worst run and status.
- A drop-% vs gauge plot for the selected run, with the budget line, current choice and smallest
  passing size marked.
- Three synthetic presets: "48 V humanoid leg", "24 V mobile base", "12 V sensor/compute tree"
  (illustrative and editable).
- Export the current table as a local CSV download.

### Model and limitations

- Steady-state DC estimate: `R = ρ(T) · L · n / A`, `ρ(T) = ρ20 · (1 + α · (T − 20 °C))`.
- AWG area from `d = 0.127 mm · 92^((36 − n) / 39)`; no hard-coded ohm tables.
- ρ20 (Ω·mm²/m): copper 0.01724, tinned copper 0.01793, aluminium 0.0282. α: 0.00393 / 0.00393 / 0.00403.
- Ampacity is a conservative chassis-wiring estimate (`9 · A^0.8` A, scaled per material):
  **approx., check your cable datasheet**. Peaks are allowed up to `√(60 s / duty)` times (1–3×).
- Status: fail if continuous drop > budget, continuous current > ampacity, or peak current > peak
  allowance; warn if peak drop > budget or continuous current > 80 % of ampacity.
- Connector/crimp resistance, bundling derating, ambient temperature, insulation rating and transient
  inductive effects are not modelled. Verify with measurements and cable/connector datasheets.

### Controls

- Edit any table value to recalculate instantly. "Use" applies the smallest passing size to that run.
- Click the ladder, or focus it (Tab) and press ↑/↓, to select a run. Hover for a tooltip.
- Toolbar: preset, add run, export CSV, EN/한국어 toggle. On mobile the settings open as a bottom sheet.

### Local build

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked --version 0.22.2
cargo test
trunk build --release        # output in dist/, served under the /wire-drop/ subpath
trunk serve                  # local preview at localhost:8080/wire-drop/
bash tools/privacy_check.sh
```

### Privacy

No analytics, trackers, external fonts/CDNs, telemetry or server component. The app makes no runtime
network requests, and entered values never leave the browser and are not stored.

### License

MIT — see `LICENSE`.
