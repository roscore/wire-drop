//! Wire Drop browser front end: DOM wiring, i18n, Canvas 2D ladder and gauge plot.
//! Everything runs locally; no network or persistent browser storage is used.

use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wd_core::*;
use web_sys::{CanvasRenderingContext2d as Ctx, Document, Element, Event, HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent};

const COLORS: [&str; 8] = ["#5ac8fa", "#ff9f43", "#a78bfa", "#34d399", "#f472b6", "#facc15", "#60a5fa", "#fb7185"];

struct App {
    s: Settings,
    runs: Vec<Run>,
    sel: usize,
    en: bool,
    hover: Option<usize>,
}

thread_local! {
    static APP: RefCell<App> = RefCell::new(App {
        s: Settings { supply_v: 48.0, budget_pct: 3.0, material: Material::Copper, temp_c: 60.0 },
        runs: Vec::new(), sel: 0, en: false, hover: None,
    });
}

fn tr(k: &str, en: bool) -> &'static str {
    let (ko, e) = match k {
        "tagline" => ("로봇 DC 전원 하네스 전선 굵기 계산기", "Robot DC power harness wire sizer"),
        "preset" => ("프리셋 (예시, 수정 가능)", "Preset (illustrative, editable)"),
        "p0" => ("48 V 휴머노이드 다리", "48 V humanoid leg"),
        "p1" => ("24 V 모바일 베이스", "24 V mobile base"),
        "p2" => ("12 V 센서/컴퓨트 트리", "12 V sensor/compute tree"),
        "add" => ("+ 구간 추가", "+ Add run"),
        "csv" => ("CSV 내보내기", "Export CSV"),
        "lang" => ("EN", "한국어"),
        "settings" => ("설정", "Settings"),
        "hint" => ("예시 하네스가 이미 로드되어 있어요. 표의 값을 바꾸면 사다리 그림과 결과가 즉시 갱신됩니다. 사다리를 클릭하거나 포커스 후 ↑/↓ 키로 구간을 선택하세요.",
                   "An example harness is already loaded. Edit any value and the ladder and results update instantly. Click the ladder, or focus it and use ↑/↓, to select a run."),
        "dismiss" => ("닫기", "Dismiss"),
        "supply_card" => ("전원", "Supply"),
        "supply_v" => ("공급 전압 (V)", "Supply voltage (V)"),
        "budget" => ("허용 전압 강하 (공급 대비 %)", "Allowed drop (% of supply)"),
        "cond_card" => ("도체", "Conductor"),
        "material" => ("재질", "Material"),
        "cu" => ("구리", "Copper"),
        "tcu" => ("주석 도금 구리", "Tinned copper"),
        "al" => ("알루미늄", "Aluminium"),
        "temp" => ("도체 온도 (°C, α 보정)", "Conductor temperature (°C, α-corrected)"),
        "model_card" => ("모델과 한계", "Model & limits"),
        "model" => ("정상 상태(steady-state) DC 추정치입니다. R = ρ(T)·L·n/A, ρ(T) = ρ20·(1 + α·(T − 20 °C)), 단면적은 AWG 공식/공칭 mm²에서 계산합니다. 커넥터·크림프 접촉 저항, 묶음(bundling) 디레이팅, 주변 온도, 절연 등급, 과도 유도성 효과는 반영되지 않으므로 실측과 데이터시트로 확인하세요.",
                    "This is a steady-state DC estimate. R = ρ(T)·L·n/A with ρ(T) = ρ20·(1 + α·(T − 20 °C)); areas come from the AWG formula or nominal mm². Connector/crimp resistance, bundling derating, ambient temperature, insulation rating and transient inductive effects are not modelled and need measurement and datasheets."),
        "amp_note" => ("허용 전류: 근사치 — approx., check your cable datasheet (보수적 섀시 배선 추정, 피크는 √(60 s/지속시간) 배까지 최대 3배)",
                       "Ampacity: approx., check your cable datasheet (conservative chassis-wiring estimate; peaks allowed up to √(60 s/duty), max 3×)"),
        "runs" => ("하네스 구간", "Harness runs"),
        "results" => ("결과", "Results"),
        "plot" => ("선택 구간: 굵기별 연속 전류 전압 강하 %", "Selected run: continuous drop % vs gauge"),
        "legend" => ("막대 두께 ∝ 단면적 · 막대 끝 높이 = 전압 강하 · 점선 = 최소 허용 전압 · 가는 선 = 피크", "Bar thickness ∝ area · bar end = voltage drop · dashed = minimum allowed voltage · thin line = peak"),
        "privacy" => ("모든 계산은 브라우저 안에서만 실행되며 입력값은 어디에도 전송·저장되지 않습니다. MIT 라이선스.",
                      "All calculations run in your browser; inputs are never sent or stored anywhere. MIT licensed."),
        "h_name" => ("이름", "Name"),
        "h_len" => ("편도 길이 (m)", "One-way length (m)"),
        "h_cont" => ("연속 (A)", "Continuous (A)"),
        "h_peak" => ("피크 (A)", "Peak (A)"),
        "h_dur" => ("피크 지속 (s)", "Peak duty (s)"),
        "h_rt" => ("왕복 (2선)", "Round trip (2 cond.)"),
        "h_size" => ("굵기", "Size"),
        "h_r" => ("저항 (mΩ)", "R (mΩ)"),
        "h_drop" => ("강하 연속", "Drop cont."),
        "h_pdrop" => ("강하 피크", "Drop peak"),
        "h_vload" => ("부하 전압 (V)", "V at load"),
        "h_loss" => ("손실 (W)", "Loss (W)"),
        "h_j" => ("전류 밀도 (A/mm²)", "J (A/mm²)"),
        "h_amp" => ("허용 전류* (A)", "Ampacity* (A)"),
        "h_st" => ("상태", "Status"),
        "h_best" => ("최소 통과 굵기", "Smallest passing"),
        "del" => ("삭제", "Remove"),
        "use" => ("적용", "Use"),
        "none" => ("없음", "none"),
        "pass" => ("통과", "pass"),
        "warn" => ("주의", "warn"),
        "fail" => ("실패", "fail"),
        "hud_loss" => ("총 손실", "Total loss"),
        "hud_worst" => ("최악 구간", "Worst run"),
        "hud_status" => ("상태", "Status"),
        "min_v" => ("최소", "min"),
        "new_run" => ("새 구간", "New run"),
        _ => ("?", "?"),
    };
    if en { e } else { ko }
}

fn doc() -> Document {
    web_sys::window().unwrap().document().unwrap()
}
fn by_id(id: &str) -> Option<Element> {
    doc().get_element_by_id(id)
}
fn set_html(id: &str, h: &str) {
    if let Some(e) = by_id(id) {
        e.set_inner_html(h);
    }
}
fn prop(t: &JsValue, k: &str) -> JsValue {
    js_sys::Reflect::get(t, &JsValue::from_str(k)).unwrap_or(JsValue::UNDEFINED)
}
fn set_prop(t: &JsValue, k: &str, v: &JsValue) {
    let _ = js_sys::Reflect::set(t, &JsValue::from_str(k), v);
}
fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
fn st_key(s: Status) -> &'static str {
    match s {
        Status::Pass => "pass",
        Status::Warn => "warn",
        Status::Fail => "fail",
    }
}
fn st_color(s: Status) -> &'static str {
    match s {
        Status::Pass => "#34d399",
        Status::Warn => "#fbbf24",
        Status::Fail => "#f87171",
    }
}
fn color(i: usize) -> &'static str {
    COLORS[i % COLORS.len()]
}

fn listen<E: wasm_bindgen::convert::FromWasmAbi + 'static>(t: &web_sys::EventTarget, ev: &str, f: impl FnMut(E) + 'static) {
    let cb = Closure::<dyn FnMut(E)>::new(f);
    let _ = t.add_event_listener_with_callback(ev, cb.as_ref().unchecked_ref());
    cb.forget();
}

fn load_preset(a: &mut App, i: usize) {
    let p = presets().into_iter().nth(i).unwrap();
    a.s = p.settings;
    a.runs = p
        .runs
        .iter()
        .map(|&(ko, en, len, c, pk, ps, size)| Run {
            name: (if a.en { en } else { ko }).into(),
            length_m: len, cont_a: c, peak_a: pk, peak_s: ps, round_trip: true, family: p.family, size,
        })
        .collect();
    a.sel = 0;
}

fn sync_settings(a: &App) {
    for (id, v) in [("supply", a.s.supply_v), ("budget", a.s.budget_pct), ("temp", a.s.temp_c)] {
        if let Some(e) = by_id(id) {
            set_prop(&e, "value", &JsValue::from_str(&format!("{}", v)));
        }
    }
    if let Some(e) = by_id("material") {
        set_prop(&e, "value", &JsValue::from_str(a.s.material.key()));
    }
}

fn render_text(a: &App) {
    let d = doc();
    if let Some(h) = d.document_element() {
        let _ = h.set_attribute("lang", if a.en { "en" } else { "ko" });
    }
    let nodes = d.query_selector_all("[data-i18n]").unwrap();
    for i in 0..nodes.length() {
        if let Some(n) = nodes.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
            let k = n.get_attribute("data-i18n").unwrap_or_default();
            n.set_text_content(Some(tr(&k, a.en)));
        }
    }
}

fn size_options(f: Family, sel: usize) -> String {
    (0..f.len())
        .map(|i| format!("<option value=\"{i}\"{}>{}</option>", if i == sel { " selected" } else { "" }, f.label(i)))
        .collect()
}

fn render_editor(a: &App) {
    let en = a.en;
    let mut h = format!(
        "<thead><tr><th></th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th></th></tr></thead><tbody>",
        tr("h_name", en), tr("h_len", en), tr("h_cont", en), tr("h_peak", en), tr("h_dur", en), tr("h_rt", en), tr("h_size", en)
    );
    for (i, r) in a.runs.iter().enumerate() {
        let num = |f: &str, v: f64, step: &str| {
            format!("<td><input type=\"number\" inputmode=\"decimal\" min=\"0\" step=\"{step}\" data-run=\"{i}\" data-f=\"{f}\" value=\"{v}\" aria-label=\"{} {}\"></td>", esc(&r.name), f)
        };
        h += &format!(
            "<tr class=\"{}\"><td><span class=\"chip\" style=\"background:{}\"></span></td>\
             <td><input type=\"text\" data-run=\"{i}\" data-f=\"name\" value=\"{}\" aria-label=\"{}\"></td>{}{}{}{}\
             <td><input type=\"checkbox\" data-run=\"{i}\" data-f=\"rt\"{} aria-label=\"{}\"></td>\
             <td class=\"size\"><select data-run=\"{i}\" data-f=\"fam\" aria-label=\"AWG / mm²\"><option value=\"awg\"{}>AWG</option><option value=\"mm\"{}>mm²</option></select>\
             <select data-run=\"{i}\" data-f=\"size\" aria-label=\"{}\">{}</select></td>\
             <td><button class=\"ghost\" data-action=\"del\" data-run=\"{i}\" aria-label=\"{} {}\">✕</button></td></tr>",
            if i == a.sel { "sel" } else { "" }, color(i), esc(&r.name), tr("h_name", en),
            num("len", r.length_m, "0.1"), num("cont", r.cont_a, "0.1"), num("peak", r.peak_a, "0.1"), num("dur", r.peak_s, "0.1"),
            if r.round_trip { " checked" } else { "" }, tr("h_rt", en),
            if r.family == Family::Awg { " selected" } else { "" }, if r.family == Family::Metric { " selected" } else { "" },
            tr("h_size", en), size_options(r.family, r.size), tr("del", en), esc(&r.name)
        );
    }
    set_html("runs", &(h + "</tbody>"));
}

fn update(a: &App) {
    let en = a.en;
    let res: Vec<RunResult> = a.runs.iter().map(|r| evaluate(&a.s, r)).collect();
    let mut h = format!(
        "<thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody>",
        tr("h_name", en), tr("h_r", en), tr("h_drop", en), tr("h_pdrop", en), tr("h_vload", en),
        tr("h_loss", en), tr("h_j", en), tr("h_amp", en), tr("h_st", en), tr("h_best", en)
    );
    for (i, (r, e)) in a.runs.iter().zip(&res).enumerate() {
        let best = match e.smallest_ok {
            Some(b) if b == r.size => format!("{} ✓", r.family.label(b)),
            Some(b) => format!("{} <button class=\"ghost small\" data-action=\"use\" data-run=\"{i}\">{}</button>", r.family.label(b), tr("use", en)),
            None => tr("none", en).into(),
        };
        h += &format!(
            "<tr class=\"{}\" data-action=\"sel\" data-run=\"{i}\"><td><span class=\"chip\" style=\"background:{}\"></span>{}</td>\
             <td>{:.1}</td><td>{:.3} V · {:.2}%</td><td>{:.3} V · {:.2}%</td><td>{:.2}</td><td>{:.2}</td><td>{:.1}</td>\
             <td>{:.0} / {:.0}</td><td><span class=\"st st-{k}\">{}</span></td><td>{}</td></tr>",
            if i == a.sel { "sel" } else { "" }, color(i), esc(&r.name), e.r_ohm * 1000.0, e.drop_v, e.drop_pct,
            e.peak_drop_v, e.peak_drop_pct, e.v_load, e.loss_w, e.density, e.ampacity_a, e.peak_limit_a,
            tr(st_key(e.status), en), best, k = st_key(e.status)
        );
    }
    set_html("results", &(h + "</tbody>"));

    let total: f64 = res.iter().map(|e| e.loss_w).sum();
    let worst = res.iter().enumerate().max_by(|x, y| (x.1.status, x.1.drop_pct).partial_cmp(&(y.1.status, y.1.drop_pct)).unwrap());
    let overall = res.iter().map(|e| e.status).max().unwrap_or(Status::Pass);
    let worst_s = worst.map(|(i, e)| format!("{} · {:.2}%", esc(&a.runs[i].name), e.drop_pct)).unwrap_or("—".into());
    set_html(
        "hud",
        &format!(
            "<div><small>{}</small><b>{:.2} W</b></div><div><small>{}</small><b>{}</b></div><div><small>{}</small><b class=\"st st-{k}\">{}</b></div>",
            tr("hud_loss", en), total, tr("hud_worst", en), worst_s, tr("hud_status", en), tr(st_key(overall), en), k = st_key(overall)
        ),
    );
    draw_ladder(a, &res);
    draw_plot(a);
    show_tip(a, &res);
}

fn canvas(id: &str, css_h: f64) -> Option<(Ctx, f64, f64)> {
    let c: HtmlCanvasElement = by_id(id)?.dyn_into().ok()?;
    let dpr = web_sys::window()?.device_pixel_ratio().max(1.0);
    let w = (c.client_width() as f64).max(200.0);
    let _ = c.style().set_property("height", &format!("{css_h}px"));
    c.set_width((w * dpr) as u32);
    c.set_height((css_h * dpr) as u32);
    let ctx: Ctx = c.get_context("2d").ok()??.dyn_into().ok()?;
    let _ = ctx.scale(dpr, dpr);
    Some((ctx, w, css_h))
}

const BAND: f64 = 66.0;
const TOP: f64 = 14.0;

fn dash(ctx: &Ctx, on: bool) {
    let arr = js_sys::Array::new();
    if on {
        arr.push(&JsValue::from_f64(5.0));
        arr.push(&JsValue::from_f64(4.0));
    }
    let _ = ctx.set_line_dash(&arr);
}

fn line(ctx: &Ctx, x0: f64, y0: f64, x1: f64, y1: f64) {
    ctx.begin_path();
    ctx.move_to(x0, y0);
    ctx.line_to(x1, y1);
    ctx.stroke();
}

fn draw_ladder(a: &App, res: &[RunResult]) {
    let n = a.runs.len().max(1) as f64;
    let Some((ctx, w, h)) = canvas("ladder", TOP * 2.0 + BAND * n) else { return };
    ctx.clear_rect(0.0, 0.0, w, h);
    let max_area = res.iter().map(|e| e.area_mm2).fold(0.0, f64::max).max(1e-9);
    let max_pct = res.iter().map(|e| e.peak_drop_pct).fold(a.s.budget_pct * 1.5, f64::max);
    let (x0, x1) = (34.0, w - 112.0);
    // Supply rail.
    ctx.set_stroke_style_str("#e2e8f0");
    ctx.set_line_width(4.0);
    line(&ctx, x0 - 6.0, TOP, x0 - 6.0, h - TOP);
    ctx.set_font("11px system-ui, sans-serif");
    ctx.set_fill_style_str("#94a3b8");
    let _ = ctx.save();
    let _ = ctx.translate(14.0, h / 2.0);
    let _ = ctx.rotate(-std::f64::consts::FRAC_PI_2);
    ctx.set_text_align("center");
    let _ = ctx.fill_text(&format!("{:.1} V", a.s.supply_v), 0.0, 0.0);
    let _ = ctx.restore();
    for (i, (r, e)) in a.runs.iter().zip(res).enumerate() {
        let top = TOP + BAND * i as f64;
        let y = |pct: f64| top + 30.0 + (pct / max_pct).min(1.0) * (BAND - 38.0);
        if i == a.sel || a.hover == Some(i) {
            ctx.set_fill_style_str(if i == a.sel { "rgba(148,163,184,0.12)" } else { "rgba(148,163,184,0.06)" });
            ctx.fill_rect(x0 - 2.0, top + 2.0, w - x0, BAND - 4.0);
        }
        // Minimum-voltage line.
        ctx.set_stroke_style_str("rgba(248,113,113,0.7)");
        ctx.set_line_width(1.0);
        dash(&ctx, true);
        let yb = y(a.s.budget_pct);
        line(&ctx, x0, yb, x1, yb);
        // Peak ghost.
        ctx.set_stroke_style_str("rgba(226,232,240,0.45)");
        line(&ctx, x0, y(0.0), x1, y(e.peak_drop_pct));
        dash(&ctx, false);
        // Main bar: thickness ∝ area.
        ctx.set_stroke_style_str(color(i));
        ctx.set_line_width((16.0 * e.area_mm2 / max_area).max(2.0));
        line(&ctx, x0, y(0.0), x1, y(e.drop_pct));
        ctx.set_fill_style_str(st_color(e.status));
        ctx.begin_path();
        let _ = ctx.arc(x1 + 8.0, y(e.drop_pct), 4.0, 0.0, std::f64::consts::TAU);
        ctx.fill();
        ctx.set_text_align("left");
        ctx.set_fill_style_str("#cbd5e1");
        ctx.set_font("12px system-ui, sans-serif");
        let _ = ctx.fill_text(&format!("{} · {}", r.name, r.family.label(r.size)), x0 + 4.0, top + 12.0);
        ctx.set_fill_style_str("#f1f5f9");
        let _ = ctx.fill_text(&format!("{:.2} V", e.v_load), x1 + 16.0, y(e.drop_pct) + 4.0);
        ctx.set_fill_style_str("#94a3b8");
        ctx.set_font("10px system-ui, sans-serif");
        let _ = ctx.fill_text(&format!("−{:.2}%", e.drop_pct), x1 + 16.0, y(e.drop_pct) + 16.0);
        if i == 0 {
            ctx.set_fill_style_str("rgba(248,113,113,0.9)");
            ctx.set_text_align("right");
            let vmin = a.s.supply_v * (1.0 - a.s.budget_pct / 100.0);
            let _ = ctx.fill_text(&format!("{} {:.2} V", tr("min_v", a.en), vmin), x1, yb + 12.0);
        }
    }
}

fn draw_plot(a: &App) {
    let Some((ctx, w, h)) = canvas("plot", 170.0) else { return };
    ctx.clear_rect(0.0, 0.0, w, h);
    let Some(r) = a.runs.get(a.sel) else { return };
    let curve = drop_curve(&a.s, r);
    let ymax = (a.s.budget_pct * 3.0).max(0.1);
    let (l, rr, t, b) = (36.0, w - 10.0, 10.0, h - 24.0);
    let px = |i: usize| l + (rr - l) * i as f64 / (curve.len() - 1).max(1) as f64;
    let py = |v: f64| b - (v / ymax).min(1.05) * (b - t);
    ctx.set_stroke_style_str("rgba(148,163,184,0.3)");
    ctx.set_line_width(1.0);
    line(&ctx, l, b, rr, b);
    line(&ctx, l, t, l, b);
    ctx.set_fill_style_str("#94a3b8");
    ctx.set_font("10px system-ui, sans-serif");
    ctx.set_text_align("right");
    for k in 0..=3 {
        let v = ymax * k as f64 / 3.0;
        let _ = ctx.fill_text(&format!("{:.1}%", v), l - 4.0, py(v) + 3.0);
    }
    ctx.set_text_align("center");
    let step = if curve.len() > 16 { 5 } else { 2 };
    for i in (0..curve.len()).step_by(step) {
        let lab = r.family.label(i).replace(" AWG", "").replace(" mm²", "");
        let _ = ctx.fill_text(&lab, px(i), h - 8.0);
    }
    ctx.set_stroke_style_str("rgba(248,113,113,0.8)");
    dash(&ctx, true);
    line(&ctx, l, py(a.s.budget_pct), rr, py(a.s.budget_pct));
    dash(&ctx, false);
    if let Some(bi) = smallest_passing(&a.s, r) {
        ctx.set_stroke_style_str("rgba(52,211,153,0.6)");
        line(&ctx, px(bi), t, px(bi), b);
    }
    ctx.set_stroke_style_str(color(a.sel));
    ctx.set_line_width(2.0);
    ctx.begin_path();
    for (i, v) in curve.iter().enumerate() {
        if i == 0 { ctx.move_to(px(i), py(*v)) } else { ctx.line_to(px(i), py(*v)) }
    }
    ctx.stroke();
    let s = r.size.min(curve.len() - 1);
    ctx.set_fill_style_str("#0b1020");
    ctx.set_line_width(2.5);
    ctx.begin_path();
    let _ = ctx.arc(px(s), py(curve[s]), 5.5, 0.0, std::f64::consts::TAU);
    ctx.fill();
    ctx.stroke();
}

fn show_tip(a: &App, res: &[RunResult]) {
    let Some(tip) = by_id("tip").and_then(|e| e.dyn_into::<HtmlElement>().ok()) else { return };
    let Some(i) = a.hover.filter(|&i| i < res.len()) else {
        let _ = tip.class_list().remove_1("show");
        return;
    };
    let (r, e) = (&a.runs[i], &res[i]);
    tip.set_inner_html(&format!(
        "<b>{}</b> · {}<br>{:.3} V ({:.2}%) → {:.2} V<br>peak {:.3} V ({:.2}%)<br>{:.2} W · {:.1} A/mm² · <span class=\"st st-{k}\">{}</span>",
        esc(&r.name), r.family.label(r.size), e.drop_v, e.drop_pct, e.v_load, e.peak_drop_v, e.peak_drop_pct,
        e.loss_w, e.density, tr(st_key(e.status), a.en), k = st_key(e.status)
    ));
    let _ = tip.style().set_property("top", &format!("{}px", TOP + BAND * i as f64 + BAND - 6.0));
    let _ = tip.class_list().add_1("show");
}

fn with<R>(f: impl FnOnce(&mut App) -> R) -> R {
    APP.with(|a| f(&mut a.borrow_mut()))
}

fn num(v: &JsValue) -> Option<f64> {
    v.as_string()?.trim().parse::<f64>().ok().filter(|x| x.is_finite() && *x >= 0.0)
}

fn on_input(ev: Event) {
    let Some(t) = ev.target().and_then(|t| t.dyn_into::<Element>().ok()) else { return };
    let val = prop(&t, "value");
    let id = t.id();
    with(|a| {
        let mut rebuild = false;
        match id.as_str() {
            "supply" => if let Some(v) = num(&val).filter(|v| *v > 0.0) { a.s.supply_v = v },
            "budget" => if let Some(v) = num(&val).filter(|v| *v > 0.0) { a.s.budget_pct = v },
            "temp" => if let Some(v) = val.as_string().and_then(|s| s.parse::<f64>().ok()) { a.s.temp_c = v.clamp(-60.0, 250.0) },
            "material" => a.s.material = Material::from_key(&val.as_string().unwrap_or_default()),
            "preset" => {
                if let Some(i) = val.as_string().and_then(|s| s.parse::<usize>().ok()) {
                    load_preset(a, i);
                    sync_settings(a);
                    rebuild = true;
                }
            }
            _ => {
                let Some(i) = t.get_attribute("data-run").and_then(|s| s.parse::<usize>().ok()) else { return };
                let Some(r) = a.runs.get_mut(i) else { return };
                match t.get_attribute("data-f").unwrap_or_default().as_str() {
                    "name" => r.name = val.as_string().unwrap_or_default(),
                    "len" => if let Some(v) = num(&val) { r.length_m = v },
                    "cont" => if let Some(v) = num(&val) { r.cont_a = v },
                    "peak" => if let Some(v) = num(&val) { r.peak_a = v },
                    "dur" => if let Some(v) = num(&val) { r.peak_s = v },
                    "rt" => r.round_trip = prop(&t, "checked").as_bool().unwrap_or(true),
                    "size" => if let Some(v) = val.as_string().and_then(|s| s.parse().ok()) { r.size = v },
                    "fam" => {
                        let f = if val.as_string().as_deref() == Some("mm") { Family::Metric } else { Family::Awg };
                        if f != r.family {
                            let area = r.family.area(r.size);
                            r.size = (0..f.len()).find(|&k| f.area(k) >= area * 0.98).unwrap_or(f.len() - 1);
                            r.family = f;
                            rebuild = true;
                        }
                    }
                    _ => {}
                }
                a.sel = i;
            }
        }
        if rebuild {
            render_editor(a);
        }
        update(a);
    });
}

fn download_csv(a: &App) {
    let data = format!("\u{feff}{}", to_csv(&a.s, &a.runs));
    let parts = js_sys::Array::of1(&JsValue::from_str(&data));
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type("text/csv;charset=utf-8");
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts) else { return };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else { return };
    if let Ok(link) = doc().create_element("a").and_then(|e| e.dyn_into::<web_sys::HtmlAnchorElement>().map_err(|e| e.into())) {
        link.set_href(&url);
        link.set_download("wire-drop.csv");
        link.click();
    }
    let _ = web_sys::Url::revoke_object_url(&url);
}

fn on_click(ev: Event) {
    let Some(t) = ev.target().and_then(|t| t.dyn_into::<Element>().ok()) else { return };
    let Some(el) = t.closest("[data-action]").ok().flatten() else { return };
    let run = el.get_attribute("data-run").and_then(|s| s.parse::<usize>().ok());
    with(|a| {
        match el.get_attribute("data-action").unwrap_or_default().as_str() {
            "add" => {
                let last = a.runs.last().cloned();
                a.runs.push(Run {
                    name: format!("{} {}", tr("new_run", a.en), a.runs.len() + 1),
                    length_m: 1.0, cont_a: 2.0, peak_a: 5.0, peak_s: 1.0, round_trip: true,
                    family: last.as_ref().map(|r| r.family).unwrap_or(Family::Awg),
                    size: last.map(|r| r.size).unwrap_or(awg_index(18)),
                });
                a.sel = a.runs.len() - 1;
            }
            "del" => if let Some(i) = run.filter(|&i| i < a.runs.len() && a.runs.len() > 1) {
                a.runs.remove(i);
                a.sel = a.sel.min(a.runs.len() - 1);
                a.hover = None;
            },
            "use" => if let Some(r) = run.and_then(|i| a.runs.get_mut(i)) {
                if let Some(b) = smallest_passing(&a.s, r) { r.size = b }
            },
            "sel" => if let Some(i) = run { a.sel = i },
            "csv" => return download_csv(a),
            "lang" => {
                a.en = !a.en;
                render_text(a);
            }
            "hint" => return { by_id("hint").map(|h| h.remove()); },
            "sheet" => return { by_id("sheet").map(|s| s.class_list().toggle("open")); },
            _ => return,
        }
        render_editor(a);
        update(a);
    });
}

fn row_at(y: f64, n: usize) -> Option<usize> {
    let i = ((y - TOP) / BAND).floor();
    (i >= 0.0 && (i as usize) < n).then_some(i as usize)
}

#[wasm_bindgen(start)]
pub fn start() {
    let d = doc();
    with(|a| {
        load_preset(a, 0);
        render_text(a);
        sync_settings(a);
        render_editor(a);
        update(a);
    });
    listen::<Event>(&d, "input", on_input);
    listen::<Event>(&d, "click", on_click);
    if let Some(c) = by_id("ladder") {
        let c2 = c.clone();
        listen::<MouseEvent>(&c, "mousemove", move |e| {
            let y = e.client_y() as f64 - c2.get_bounding_client_rect().top();
            with(|a| {
                let h = row_at(y, a.runs.len());
                if h != a.hover {
                    a.hover = h;
                    update(a);
                }
            })
        });
        listen::<MouseEvent>(&c, "mouseleave", |_| with(|a| { a.hover = None; update(a) }));
        let c3 = c.clone();
        listen::<MouseEvent>(&c, "click", move |e| {
            let y = e.client_y() as f64 - c3.get_bounding_client_rect().top();
            with(|a| if let Some(i) = row_at(y, a.runs.len()) {
                a.sel = i;
                render_editor(a);
                update(a);
            })
        });
        listen::<KeyboardEvent>(&c, "keydown", |e| with(|a| {
            let n = a.runs.len();
            let s = match e.key().as_str() {
                "ArrowDown" => (a.sel + 1) % n,
                "ArrowUp" => (a.sel + n - 1) % n,
                _ => return,
            };
            e.prevent_default();
            a.sel = s;
            a.hover = Some(s);
            render_editor(a);
            update(a);
        }));
        listen::<Event>(&c, "focus", |_| with(|a| { a.hover = Some(a.sel); update(a) }));
        listen::<Event>(&c, "blur", |_| with(|a| { a.hover = None; update(a) }));
    }
    let w = web_sys::window().unwrap();
    listen::<Event>(&w, "resize", |_| with(|a| update(a)));
}
