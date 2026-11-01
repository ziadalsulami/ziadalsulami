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
    let pair: Vec<_> = (0..n)
        .map(|i| {
            let white = render_swim_frame(&development, KoiBody::White, b, i);
            let asagi = render_swim_frame(&development, KoiBody::Asagi, b, i + n / 2);
            let mut px = white.as_rgba_bytes().to_vec();
            for (dst, src) in px.chunks_mut(4).zip(asagi.as_rgba_bytes().chunks(4)) {
                if src[3] != 0 {
                    dst.copy_from_slice(src);
                }
            }
            KoiFrame::from_rgba_pixels(px.chunks(4).map(|c| [c[0], c[1], c[2], c[3]]).collect())
        })
        .collect();
    std::fs::write("koi-pair.gif", encode_gif(&pair).unwrap()).unwrap();
    for (name, body) in [("white", KoiBody::White), ("asagi", KoiBody::Asagi)] {
        let b = Behavior::Cruise;
        let frames: Vec<_> = (0..frame_count(b))
            .map(|i| render_swim_frame(&development, body, b, i))
            .collect();
        std::fs::write(format!("koi-{name}.gif"), encode_gif(&frames).unwrap()).unwrap();
    }
}
