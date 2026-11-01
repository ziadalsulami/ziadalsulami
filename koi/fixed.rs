use std::collections::BTreeMap;
use chrono::NaiveDate;
use git_koi::behavior::Behavior;
use git_koi::coat::KoiBody;
use git_koi::history::LanguageColor;
use git_koi::koi::{KoiDevelopment, RecentActivity, StreakDevelopment};
use git_koi::motion::frame_count;
use git_koi::output::encode_gif;
use git_koi::render::render_swim_frame;

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
    for (name, body) in [("white", KoiBody::White), ("asagi", KoiBody::Asagi)] {
        let b = Behavior::Cruise;
        let frames: Vec<_> = (0..frame_count(b))
            .map(|i| render_swim_frame(&development, body, b, i))
            .collect();
        std::fs::write(format!("koi-{name}.gif"), encode_gif(&frames).unwrap()).unwrap();
    }
}
