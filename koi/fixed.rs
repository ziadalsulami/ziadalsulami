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
    /// Splash droplets as (sideways, upward) speed in pixels per frame.
    drops: Vec<(f64, f64)>,
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
    const RIPPLE_COUNT: usize = 5;
    let ripples: Vec<Ripple> = (0..RIPPLE_COUNT)
        .map(|k| Ripple {
            x: rand(24.0, 232.0) as i32,
            y: rand(24.0, 232.0) as i32,
            // spread the start times so ripples overlap in time but never all at once
            start: (k * n / RIPPLE_COUNT + rand(0.0, 6.0) as usize) % n,
            life: rand(22.0, 40.0) as usize,
            max_radius: rand(9.0, 30.0),
            drops: (0..rand(3.0, 6.0) as usize)
                .map(|_| (rand(-1.6, 1.6), rand(1.2, 2.6)))
                .collect(),
        })
        .collect();

    let ripple_color = [127u8, 183, 230, 255];
    let faint_color = ripple_color;
    let drop_color = ripple_color;
    let frames: Vec<KoiFrame> = (0..n)
        .map(|i| {
            let mut px = base[i].clone();
            for ripple in &ripples {
                let age = (i + n - ripple.start) % n;
                if age >= ripple.life {
                    continue;
                }
                // Ring 0 is the main ripple. Rings 1 and 2 start a few frames later,
                // smaller and fainter, like the inner waves of a real ripple.
                for ring in 0..3usize {
                    let delay = ring * 5;
                    if age < delay {
                        continue;
                    }
                    let ring_age = age - delay;
                    let ring_life = ripple.life - delay.min(ripple.life - 4);
                    if ring_age >= ring_life {
                        continue;
                    }
                    let t = ring_age as f64 / ring_life as f64;
                    let reach = ripple.max_radius * (1.0 - 0.3 * ring as f64);
                    // fast at first, then slowing as it spreads
                    let radius = 1.0 + reach * (1.0 - (1.0 - t) * (1.0 - t));
                    // main ring: solid, then thinning; inner rings: always dithered
                    let density = if ring > 0 {
                        if t < 0.5 { 1 } else { 2 }
                    } else if t < 0.5 {
                        0
                    } else if t < 0.75 {
                        1
                    } else {
                        2
                    };
                    let color = if ring == 0 { ripple_color } else { faint_color };
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
                                px[k..k + 4].copy_from_slice(&color);
                            }
                        }
                    }
                }
                // Splash: droplets fly up and out from the center, then fall back.
                const SPLASH_FRAMES: usize = 10;
                if age < SPLASH_FRAMES {
                    let a = age as f64;
                    for &(vx, vy) in &ripple.drops {
                        let x = ripple.x + (vx * a).round() as i32;
                        let y = ripple.y - (vy * a - 0.22 * a * a).round() as i32;
                        // droplets shrink from 2x2 to a single pixel near the end
                        let size = if age < SPLASH_FRAMES - 3 { 2 } else { 1 };
                        for oy in 0..size {
                            for ox in 0..size {
                                let (x, y) = (x + ox, y + oy);
                                if x < 0 || y < 0 || x >= side as i32 || y >= side as i32 {
                                    continue;
                                }
                                let k = (y as usize * side + x as usize) * 4;
                                if px[k + 3] == 0 {
                                    px[k..k + 4].copy_from_slice(&drop_color);
                                }
                            }
                        }
                    }
                }
            }
            KoiFrame::from_rgba_pixels(px.chunks(4).map(|c| [c[0], c[1], c[2], c[3]]).collect())
        })
        .collect();
    std::fs::write("koi-white.gif", encode_gif(&frames).unwrap()).unwrap();
}
