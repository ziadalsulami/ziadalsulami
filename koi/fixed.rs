use std::collections::BTreeMap;
use chrono::NaiveDate;
use git_koi::behavior::Behavior;
use git_koi::coat::KoiBody;
use git_koi::history::LanguageColor;
use git_koi::koi::{KoiDevelopment, RecentActivity, StreakDevelopment};
use git_koi::motion::frame_count;
use git_koi::output::encode_gif;
use git_koi::render::{render_swim_frame, KoiFrame};

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
    // Both koi on one canvas, half a lap apart, swimming the same circle.
    let b = Behavior::Cruise;
    let n = frame_count(b);
    let _ = n;
    let side = 256usize;
    let base: Vec<Vec<u8>> = (0..n)
        .map(|i| render_swim_frame(&development, KoiBody::White, b, i).as_rgba_bytes().to_vec())
        .collect();
    // Where each frame's koi is: the centroid of its opaque pixels.
    let centers: Vec<(f64, f64)> = base
        .iter()
        .map(|px| {
            let (mut sx, mut sy, mut c) = (0.0, 0.0, 0.0);
            for (k, p) in px.chunks(4).enumerate() {
                if p[3] != 0 {
                    sx += (k % side) as f64;
                    sy += (k / side) as f64;
                    c += 1.0;
                }
            }
            (sx / c, sy / c)
        })
        .collect();
    // Ripples spawn under the koi every SPAWN_EVERY frames and widen for LIFE frames.
    // Indexing the spawn frame modulo the loop keeps the animation seamless.
    const SPAWN_EVERY: usize = 7;
    const LIFE: usize = 28;
    let ripple = [127u8, 183, 230, 255];
    let frames: Vec<KoiFrame> = (0..n as usize)
        .map(|i| {
            let mut px = base[i].clone();
            let mut age = 0;
            while age < LIFE {
                let (cx, cy) = centers[(i + n as usize - age % n as usize) % n as usize];
                let radius = 10.0 + age as f64 * 2.2;
                // Past two thirds of its life, a ripple thins to a dither.
                let sparse = age * 3 > LIFE * 2;
                let r = radius.ceil() as i32 + 1;
                for dy in -r..=r {
                    for dx in -r..=r {
                        let (x, y) = (cx.round() as i32 + dx, cy.round() as i32 + dy);
                        if x < 0 || y < 0 || x >= side as i32 || y >= side as i32 {
                            continue;
                        }
                        let dist = ((dx * dx + dy * dy) as f64).sqrt();
                        if (dist - radius).abs() > 0.55 || (sparse && (x + y) % 2 == 0) {
                            continue;
                        }
                        let k = (y as usize * side + x as usize) * 4;
                        if px[k + 3] == 0 {
                            px[k..k + 4].copy_from_slice(&ripple);
                        }
                    }
                }
                age += SPAWN_EVERY;
            }
            KoiFrame::from_rgba_pixels(px.chunks(4).map(|c| [c[0], c[1], c[2], c[3]]).collect())
        })
        .collect();
    std::fs::write("koi-white.gif", encode_gif(&frames).unwrap()).unwrap();
}
