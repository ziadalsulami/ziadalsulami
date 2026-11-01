use std::collections::BTreeMap;
use chrono::NaiveDate;
use git_koi::behavior::Behavior;
use git_koi::coat::KoiBody;
use git_koi::history::LanguageColor;
use git_koi::koi::{KoiDevelopment, RecentActivity, StreakDevelopment};
use git_koi::motion::frame_count;
use git_koi::output::encode_gif;
use git_koi::render::{render_swim_frame, KoiFrame};

struct Ripple {
    x: i32,
    y: i32,
    start: usize,
    life: usize,
    max_radius: f64,
}

fn main() {
    // (name, share, color) -- edit these to recolor the fish
    let langs = [
        ("Rust", 0.25, "#dea584"),
        ("Python", 0.25, "#3572A5"),
        ("JavaScript", 0.2, "#f1e05a"),
        ("TypeScript", 0.2, "#3178c6"),
        ("C", 0.1, "#555555"),
    ];
    let development = KoiDevelopment {
        identity_seed: 12345,
        language_shares: langs.iter().map(|l| (l.0.to_string(), l.1)).collect(),
        language_colors: langs
            .iter()
            .map(|l| (l.0.to_string(), Some(LanguageColor::try_from(l.2).unwrap())))
            .collect::<BTreeMap<_, _>>(),
        total_contributions: 100_000,
        observed_on: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        streaks: StreakDevelopment { longest: vec![], recent: None },
        recent_activity: RecentActivity {
            quiet_day_count: 0,
            recent_contributions: 0,
            complete_week_contributions: vec![],
            observation_week_contributions: vec![],
        },
    };
    let b = Behavior::Cruise;
    let n = frame_count(b) as usize;
    let side = 256usize;
    let base: Vec<Vec<u8>> = (0..n)
        .map(|i| render_swim_frame(&development, KoiBody::White, b, i as u32).as_rgba_bytes().to_vec())
        .collect();

    // Ripples appear at random spots, widen to a random size, and fade away. They
    // are independent of the koi. Each has a start frame and wraps modulo the loop,
    // so the animation is seamless. A fixed seed keeps the result the same each run.
    let mut seed: u64 = 0x2545F4914F6CDD1D;
    let mut rand = move |lo: f64, hi: f64| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        lo + (seed % 10_000) as f64 / 10_000.0 * (hi - lo)
    };
    const RIPPLE_COUNT: usize = 9;
    let ripples: Vec<Ripple> = (0..RIPPLE_COUNT)
        .map(|k| Ripple {
            x: rand(24.0, 232.0) as i32,
            y: rand(24.0, 232.0) as i32,
            // spread the start times so ripples overlap in time but never all at once
            start: (k * n / RIPPLE_COUNT + rand(0.0, 6.0) as usize) % n,
            life: rand(22.0, 40.0) as usize,
            max_radius: rand(9.0, 30.0),
        })
        .collect();

    let ripple_color = [127u8, 183, 230, 255];
    let frames: Vec<KoiFrame> = (0..n)
        .map(|i| {
            let mut px = base[i].clone();
            for ripple in &ripples {
                let age = (i + n - ripple.start) % n;
                if age >= ripple.life {
                    continue;
                }
                let t = age as f64 / ripple.life as f64;
                // fast at first, then slowing as it spreads
                let radius = 1.0 + ripple.max_radius * (1.0 - (1.0 - t) * (1.0 - t));
                // the first half stays solid, then it thins to a dither and vanishes
                let density = if t < 0.5 { 0 } else if t < 0.75 { 1 } else { 2 };
                let r = radius.ceil() as i32 + 1;
                for dy in -r..=r {
                    for dx in -r..=r {
                        let (x, y) = (ripple.x + dx, ripple.y + dy);
                        if x < 0 || y < 0 || x >= side as i32 || y >= side as i32 {
                            continue;
                        }
                        let dist = ((dx * dx + dy * dy) as f64).sqrt();
                        if (dist - radius).abs() > 0.55 {
                            continue;
                        }
                        let keep = match density {
                            0 => true,
                            1 => (x + y) % 2 == 0,
                            _ => (x % 2 == 0) && (y % 2 == 0),
                        };
                        if !keep {
                            continue;
                        }
                        let k = (y as usize * side + x as usize) * 4;
                        if px[k + 3] == 0 {
                            px[k..k + 4].copy_from_slice(&ripple_color);
                        }
                    }
                }
            }
            KoiFrame::from_rgba_pixels(px.chunks(4).map(|c| [c[0], c[1], c[2], c[3]]).collect())
        })
        .collect();
    std::fs::write("koi-white.gif", encode_gif(&frames).unwrap()).unwrap();
}
