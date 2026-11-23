//! Wire Drop core: steady-state DC voltage drop, I²R loss and approximate ampacity
//! for robot power harness runs. Pure Rust, no I/O.

/// Conductor material with 20 °C resistivity (Ω·mm²/m) and temperature coefficient (1/K).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Material {
    Copper,
    TinnedCopper,
    Aluminium,
}

impl Material {
    pub const ALL: [Material; 3] = [Material::Copper, Material::TinnedCopper, Material::Aluminium];

    pub fn rho20(self) -> f64 {
        match self {
            Material::Copper => 0.017_24,
            Material::TinnedCopper => 0.017_93,
            Material::Aluminium => 0.028_2,
        }
    }

    pub fn alpha(self) -> f64 {
        match self {
            Material::Copper | Material::TinnedCopper => 0.003_93,
            Material::Aluminium => 0.004_03,
        }
    }

    /// Resistivity corrected linearly to conductor temperature `t_c`.
    pub fn rho_at(self, t_c: f64) -> f64 {
        self.rho20() * (1.0 + self.alpha() * (t_c - 20.0))
    }

    pub fn key(self) -> &'static str {
        match self {
            Material::Copper => "cu",
            Material::TinnedCopper => "tcu",
            Material::Aluminium => "al",
        }
    }

    pub fn from_key(k: &str) -> Material {
        Material::ALL.into_iter().find(|m| m.key() == k).unwrap_or(Material::Copper)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Family {
    Awg,
    Metric,
}

/// Metric cross-sections offered, mm².
pub const METRIC_MM2: [f64; 14] =
    [0.25, 0.34, 0.5, 0.75, 1.0, 1.5, 2.5, 4.0, 6.0, 10.0, 16.0, 25.0, 35.0, 50.0];

/// AWG gauges offered, thinnest first (30 … 0).
pub const AWG_MIN: i32 = 0;
pub const AWG_MAX: i32 = 30;

/// AWG cross-section in mm² from the defining diameter formula
/// d = 0.127 mm · 92^((36 − n) / 39).
pub fn awg_area_mm2(n: i32) -> f64 {
    let d = 0.127 * 92f64.powf((36 - n) as f64 / 39.0);
    std::f64::consts::PI / 4.0 * d * d
}

impl Family {
    pub fn len(self) -> usize {
        match self {
            Family::Awg => (AWG_MAX - AWG_MIN + 1) as usize,
            Family::Metric => METRIC_MM2.len(),
        }
    }

    /// Area of size index `i` (ascending area).
    pub fn area(self, i: usize) -> f64 {
        match self {
            Family::Awg => awg_area_mm2(AWG_MAX - i as i32),
            Family::Metric => METRIC_MM2[i],
        }
    }

    pub fn label(self, i: usize) -> String {
        match self {
            Family::Awg => format!("{} AWG", AWG_MAX - i as i32),
            Family::Metric => format!("{} mm²", METRIC_MM2[i]),
        }
    }
}

/// Global harness settings.
#[derive(Clone, Debug)]
pub struct Settings {
    pub supply_v: f64,
    pub budget_pct: f64,
    pub material: Material,
    pub temp_c: f64,
}

/// One harness run (e.g. distribution board → motor driver).
#[derive(Clone, Debug)]
pub struct Run {
    pub name: String,
    pub length_m: f64,
    pub cont_a: f64,
    pub peak_a: f64,
    pub peak_s: f64,
    pub round_trip: bool,
    pub family: Family,
    pub size: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Pass,
    Warn,
    Fail,
}

#[derive(Clone, Debug)]
pub struct RunResult {
    pub area_mm2: f64,
    pub r_ohm: f64,
    pub drop_v: f64,
    pub drop_pct: f64,
    pub peak_drop_v: f64,
    pub peak_drop_pct: f64,
    pub v_load: f64,
    pub v_load_peak: f64,
    pub loss_w: f64,
    pub density: f64,
    pub ampacity_a: f64,
    pub peak_limit_a: f64,
    pub status: Status,
    /// Smallest size index in the run's family that passes, if any.
    pub smallest_ok: Option<usize>,
}

/// Total loop resistance of a run: ρ(T) · L · conductors / A.
pub fn run_resistance(s: &Settings, length_m: f64, round_trip: bool, area_mm2: f64) -> f64 {
    let conductors = if round_trip { 2.0 } else { 1.0 };
    s.material.rho_at(s.temp_c) * length_m.max(0.0) * conductors / area_mm2
}

/// Conservative chassis-wiring ampacity estimate (approx., check your cable datasheet).
/// Fitted below common single-wire chassis tables: I ≈ 9 · A^0.8 for copper, scaled by
/// √(ρ_cu / ρ_mat) for equal heating.
pub fn ampacity_a(m: Material, area_mm2: f64) -> f64 {
    9.0 * area_mm2.powf(0.8) * (Material::Copper.rho20() / m.rho20()).sqrt()
}

/// Short-duty allowance: peaks up to `ampacity · clamp(√(60 s / duty), 1, 3)`.
pub fn peak_limit_a(ampacity: f64, peak_s: f64) -> f64 {
    let k = if peak_s <= 0.0 { 3.0 } else { (60.0 / peak_s).sqrt().clamp(1.0, 3.0) };
    ampacity * k
}

fn status_of(s: &Settings, r: &Run, area: f64) -> (Status, [f64; 6]) {
    let res = run_resistance(s, r.length_m, r.round_trip, area);
    let v = s.supply_v.max(1e-9);
    let drop = r.cont_a * res;
    let peak_drop = r.peak_a.max(r.cont_a) * res;
    let amp = ampacity_a(s.material, area);
    let plim = peak_limit_a(amp, r.peak_s);
    let drop_pct = drop / v * 100.0;
    let peak_pct = peak_drop / v * 100.0;
    let st = if drop_pct > s.budget_pct || r.cont_a > amp || r.peak_a > plim {
        Status::Fail
    } else if peak_pct > s.budget_pct || r.cont_a > 0.8 * amp {
        Status::Warn
    } else {
        Status::Pass
    };
    (st, [res, drop, peak_drop, amp, plim, drop_pct])
}

/// Smallest size index in `r.family` whose status is Pass.
pub fn smallest_passing(s: &Settings, r: &Run) -> Option<usize> {
    (0..r.family.len()).find(|&i| status_of(s, r, r.family.area(i)).0 == Status::Pass)
}

pub fn evaluate(s: &Settings, r: &Run) -> RunResult {
    let size = r.size.min(r.family.len() - 1);
    let area = r.family.area(size);
    let (status, [res, drop, peak_drop, amp, plim, drop_pct]) = status_of(s, r, area);
    RunResult {
        area_mm2: area,
        r_ohm: res,
        drop_v: drop,
        drop_pct,
        peak_drop_v: peak_drop,
        peak_drop_pct: peak_drop / s.supply_v.max(1e-9) * 100.0,
        v_load: s.supply_v - drop,
        v_load_peak: s.supply_v - peak_drop,
        loss_w: r.cont_a * r.cont_a * res,
        density: r.cont_a / area,
        ampacity_a: amp,
        peak_limit_a: plim,
        status,
        smallest_ok: smallest_passing(s, r),
    }
}

/// Continuous drop % for every size of the run's family (for the gauge plot).
pub fn drop_curve(s: &Settings, r: &Run) -> Vec<f64> {
    (0..r.family.len())
        .map(|i| r.cont_a * run_resistance(s, r.length_m, r.round_trip, r.family.area(i)) / s.supply_v.max(1e-9) * 100.0)
        .collect()
}

/// RFC 4180-style field escaping.
pub fn csv_escape(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn status_str(st: Status) -> &'static str {
    match st {
        Status::Pass => "pass",
        Status::Warn => "warn",
        Status::Fail => "fail",
    }
}

pub fn to_csv(s: &Settings, runs: &[Run]) -> String {
    let mut out = String::from(
        "name,length_m,conductors,size,area_mm2,cont_a,peak_a,peak_s,resistance_ohm,drop_v,drop_pct,\
         peak_drop_v,peak_drop_pct,v_load,loss_w,density_a_mm2,ampacity_approx_a,status,smallest_passing\n",
    );
    for r in runs {
        let e = evaluate(s, r);
        let best = e.smallest_ok.map(|i| r.family.label(i)).unwrap_or_else(|| "none".into());
        let cols = [
            csv_escape(&r.name),
            format!("{}", r.length_m),
            (if r.round_trip { "2" } else { "1" }).into(),
            csv_escape(&r.family.label(r.size)),
            format!("{:.3}", e.area_mm2),
            format!("{}", r.cont_a),
            format!("{}", r.peak_a),
            format!("{}", r.peak_s),
            format!("{:.5}", e.r_ohm),
            format!("{:.4}", e.drop_v),
            format!("{:.3}", e.drop_pct),
            format!("{:.4}", e.peak_drop_v),
            format!("{:.3}", e.peak_drop_pct),
            format!("{:.3}", e.v_load),
            format!("{:.3}", e.loss_w),
            format!("{:.3}", e.density),
            format!("{:.1}", e.ampacity_a),
            status_str(e.status).into(),
            csv_escape(&best),
        ];
        out.push_str(&cols.join(","));
        out.push('\n');
    }
    out
}

/// Synthetic, illustrative presets: (settings, runs as (name_ko, name_en, len, cont, peak, peak_s, size)).
pub struct Preset {
    pub settings: Settings,
    pub family: Family,
    pub runs: Vec<(&'static str, &'static str, f64, f64, f64, f64, usize)>,
}

/// AWG gauge number → index in `Family::Awg`.
pub fn awg_index(n: i32) -> usize {
    (AWG_MAX - n) as usize
}

pub fn presets() -> [Preset; 3] {
    let st = |v, b| Settings { supply_v: v, budget_pct: b, material: Material::Copper, temp_c: 60.0 };
    [
        Preset {
            settings: st(48.0, 3.0),
            family: Family::Awg,
            runs: vec![
                ("배터리 → 분배 보드", "Battery → distribution", 0.4, 30.0, 90.0, 2.0, awg_index(8)),
                ("고관절 피치 드라이버", "Hip pitch driver", 0.9, 8.0, 40.0, 1.0, awg_index(14)),
                ("무릎 드라이버", "Knee driver", 1.3, 7.0, 35.0, 1.0, awg_index(16)),
                ("발목 드라이버", "Ankle driver", 1.7, 4.0, 20.0, 0.5, awg_index(18)),
            ],
        },
        Preset {
            settings: st(24.0, 3.0),
            family: Family::Metric,
            runs: vec![
                ("배터리 → 메인 퓨즈", "Battery → main fuse", 0.5, 25.0, 60.0, 3.0, 8),
                ("좌측 휠 모터", "Left wheel motor", 1.2, 8.0, 25.0, 2.0, 4),
                ("우측 휠 모터", "Right wheel motor", 1.2, 8.0, 25.0, 2.0, 3),
                ("리프트 액추에이터", "Lift actuator", 2.0, 3.0, 12.0, 5.0, 2),
            ],
        },
        Preset {
            settings: st(12.0, 2.0),
            family: Family::Awg,
            runs: vec![
                ("DC-DC → 컴퓨트 보드", "DC-DC → compute board", 0.6, 5.0, 8.0, 1.0, awg_index(18)),
                ("라이다", "Lidar", 1.5, 1.2, 2.0, 0.5, awg_index(22)),
                ("카메라 허브", "Camera hub", 1.1, 1.5, 2.5, 0.5, awg_index(24)),
                ("IMU / 센서 버스", "IMU / sensor bus", 2.5, 0.3, 0.5, 0.2, awg_index(26)),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cu20() -> Settings {
        Settings { supply_v: 48.0, budget_pct: 3.0, material: Material::Copper, temp_c: 20.0 }
    }

    fn run(len: f64, a: f64, rt: bool, size: usize) -> Run {
        Run { name: "r".into(), length_m: len, cont_a: a, peak_a: a, peak_s: 1.0, round_trip: rt, family: Family::Awg, size }
    }

    #[test]
    fn awg_area_formula() {
        assert!((awg_area_mm2(10) - 5.261).abs() < 0.005);
        assert!((awg_area_mm2(0) - 53.48).abs() < 0.05);
        assert!((awg_area_mm2(30) - 0.0507).abs() < 0.0005);
    }

    #[test]
    fn copper_resistance_and_temperature() {
        // 10 AWG copper ≈ 3.277 mΩ/m at 20 °C.
        let r20 = run_resistance(&cu20(), 1.0, false, awg_area_mm2(10));
        assert!((r20 * 1000.0 - 3.277).abs() < 0.01);
        // +50 K → ×(1 + 0.00393·50) = ×1.1965
        let hot = Settings { temp_c: 70.0, ..cu20() };
        let r70 = run_resistance(&hot, 1.0, false, awg_area_mm2(10));
        assert!((r70 / r20 - 1.1965).abs() < 1e-9);
    }

    #[test]
    fn round_trip_doubles_drop() {
        let s = cu20();
        let one = evaluate(&s, &run(1.0, 10.0, false, awg_index(10)));
        let two = evaluate(&s, &run(1.0, 10.0, true, awg_index(10)));
        assert!((two.drop_v / one.drop_v - 2.0).abs() < 1e-12);
        // 2 m × 3.277 mΩ/m × 10 A ≈ 65.5 mV
        assert!((two.drop_v - 0.06555).abs() < 0.0003);
        assert!((two.v_load - (48.0 - two.drop_v)).abs() < 1e-12);
    }

    #[test]
    fn i2r_loss() {
        let s = cu20();
        let e = evaluate(&s, &run(5.0, 20.0, true, awg_index(10)));
        assert!((e.loss_w - 400.0 * e.r_ohm).abs() < 1e-12);
        assert!((e.loss_w - 400.0 * 0.032_77).abs() < 0.05);
    }

    #[test]
    fn smallest_passing_gauge_search() {
        let s = Settings { supply_v: 24.0, budget_pct: 3.0, ..cu20() };
        // 3 m round trip at 10 A, budget 0.72 V → R ≤ 72 mΩ → A ≥ 0.01724·6/0.072 = 1.437 mm²
        // → 15 AWG (1.65 mm²) by drop; ampacity of 15 AWG ≈ 13 A also clears 10 A / 0.8.
        let r = run(3.0, 10.0, true, 0);
        let i = smallest_passing(&s, &r).unwrap();
        assert_eq!(r.family.label(i), "15 AWG");
        assert_eq!(evaluate(&s, &Run { size: i, ..r.clone() }).status, Status::Pass);
        assert_ne!(evaluate(&s, &Run { size: i - 1, ..r }).status, Status::Pass);
    }

    #[test]
    fn csv_escaping() {
        assert_eq!(csv_escape("plain"), "plain");
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
        let csv = to_csv(&cu20(), &[Run { name: "L, \"hip\"".into(), ..run(1.0, 1.0, true, 10) }]);
        assert!(csv.lines().nth(1).unwrap().starts_with("\"L, \"\"hip\"\"\",1,2,20 AWG,"));
    }
}
