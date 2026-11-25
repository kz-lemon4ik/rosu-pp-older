use std::collections::HashMap;
use std::path::PathBuf;

use rosu_pp::Beatmap;
use rosu_pp_older::{osu_2015_april, osu_2018};

struct Score {
    id: u32,
    combo: u32,
    n300: u32,
    n100: u32,
    n50: u32,
    misses: u32,
    mods: u32,
    pp: f64,
}

fn mods(text: &str) -> u32 {
    text.split('|')
        .map(str::trim)
        .map(|name| match name {
            "nomod" => 0,
            "nf" => 1,
            "ez" => 2,
            "td" => 4,
            "hd" => 8,
            "hr" => 16,
            "dt" => 64,
            "ht" => 256,
            "nc" => 512,
            "fl" => 1024,
            "so" => 4096,
            other => panic!("unknown mod {other}"),
        })
        .fold(0, |all, bit| all | bit)
}

fn suite() -> Option<(Vec<Score>, PathBuf)> {
    let list = std::env::var_os("OPPAI_SUITE_LIST")?;
    let maps = PathBuf::from(std::env::var_os("OPPAI_SUITE_MAPS")?);
    let text = std::fs::read_to_string(list).expect("suite list");
    let scores = text
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('{'))
        .filter_map(|line| {
            let inner = line
                .trim_start_matches('{')
                .trim_end_matches(',')
                .trim_end_matches('}');
            let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
            if parts.len() != 8 {
                return None;
            }
            Some(Score {
                id: parts[0].parse().ok()?,
                combo: parts[1].parse().ok()?,
                n300: parts[2].parse().ok()?,
                n100: parts[3].parse().ok()?,
                n50: parts[4].parse().ok()?,
                misses: parts[5].parse().ok()?,
                mods: mods(parts[6]),
                pp: parts[7].parse().ok()?,
            })
        })
        .collect();
    Some((scores, maps))
}

#[derive(Clone, Copy)]
enum System {
    April2015,
    May2018,
}

fn errors(selected: impl Fn(&Score) -> bool) -> Vec<(f64, f64, f64, u32, u32)> {
    errors_in(System::May2018, selected)
}

fn errors_in(system: System, selected: impl Fn(&Score) -> bool) -> Vec<(f64, f64, f64, u32, u32)> {
    let Some((scores, maps)) = suite() else {
        return Vec::new();
    };
    let mut beatmaps = HashMap::new();
    let mut difficulty = HashMap::new();
    let mut out = Vec::new();
    for score in scores.iter().filter(|score| selected(score)) {
        let map = beatmaps.entry(score.id).or_insert_with(|| {
            Beatmap::from_path(maps.join(format!("{}.osu", score.id))).expect("suite map")
        });
        let key = (score.id, score.mods & (2 | 16 | 64 | 256));
        let pp = match system {
            System::May2018 => {
                let attributes = difficulty
                    .entry(key)
                    .or_insert_with(|| {
                        osu_2018::OsuPP::new(map)
                            .mods(score.mods)
                            .calculate()
                            .difficulty
                    })
                    .clone();
                osu_2018::OsuPP::new(map)
                    .attributes(attributes)
                    .mods(score.mods)
                    .combo(score.combo)
                    .n300(score.n300)
                    .n100(score.n100)
                    .n50(score.n50)
                    .misses(score.misses)
                    .calculate()
                    .pp
            }
            System::April2015 => {
                osu_2015_april::OsuPP::new(map)
                    .touch_device_penalty(true)
                    .mods(score.mods)
                    .combo(score.combo)
                    .n300(score.n300)
                    .n100(score.n100)
                    .n50(score.n50)
                    .misses(score.misses)
                    .calculate()
                    .pp
            }
        };
        out.push((
            (pp - score.pp) / score.pp,
            pp,
            score.pp,
            score.id,
            score.mods,
        ));
    }
    out
}

fn report(name: &str, found: &[(f64, f64, f64, u32, u32)]) -> (f64, f64, f64) {
    let share = |limit: f64| {
        found.iter().filter(|e| e.0.abs() <= limit).count() as f64 / found.len().max(1) as f64
    };
    let worst = found.iter().map(|e| e.0.abs()).fold(0.0, f64::max);
    let mean = found.iter().map(|e| e.0).sum::<f64>() / found.len().max(1) as f64;
    println!(
        "{name}: {} scores | within 0.01%: {:.1}% | within 0.1%: {:.1}% | within 1%: {:.1}% | mean {:+.3}% | worst {:.2}%",
        found.len(),
        share(0.0001) * 100.0,
        share(0.001) * 100.0,
        share(0.01) * 100.0,
        mean * 100.0,
        worst * 100.0
    );
    let mut sorted: Vec<_> = found.to_vec();
    sorted.sort_by(|a, b| b.0.abs().total_cmp(&a.0.abs()));
    for e in sorted.iter().take(5) {
        println!(
            "  map {} mods {} computed {:.3} live {:.3} ({:+.2}%)",
            e.3,
            e.4,
            e.1,
            e.2,
            e.0 * 100.0
        );
    }
    (share(0.001), worst, mean)
}

#[test]
#[ignore = "needs the 2018 oppai-ng test suite in OPPAI_SUITE_LIST and OPPAI_SUITE_MAPS"]
fn osu_2018_matches_the_live_pp_of_may_2018() {
    let all = errors(|_| true);
    if all.is_empty() {
        return;
    }
    let (within, worst, _) = report("all", &all);
    report("touch device", &errors(|score| score.mods & 4 != 0));
    for (name, bits) in [
        ("nomod", 0u32),
        ("hd", 8),
        ("hr", 16),
        ("dt", 64),
        ("fl", 1024),
    ] {
        let group: Vec<_> = all
            .iter()
            .copied()
            .filter(|e| {
                if bits == 0 {
                    e.4 & (8 | 16 | 64 | 1024) == 0
                } else {
                    e.4 & bits != 0
                }
            })
            .collect();
        let within =
            group.iter().filter(|e| e.0.abs() <= 0.001).count() as f64 / group.len().max(1) as f64;
        let mean = group.iter().map(|e| e.0).sum::<f64>() / group.len().max(1) as f64;
        println!(
            "  {name}: {} scores, within 0.1% {:.1}%, mean {:+.3}%",
            group.len(),
            within * 100.0,
            mean * 100.0
        );
    }
    assert!(within >= 0.99, "only {:.1}% within 0.1%", within * 100.0);
    assert!(worst <= 0.005, "worst error {:.2}%", worst * 100.0);
}

#[test]
#[ignore = "needs the 2018 oppai-ng test suite in OPPAI_SUITE_LIST and OPPAI_SUITE_MAPS"]
fn osu_2015_april_matches_the_live_pp_of_may_2018_without_hidden() {
    let found = errors_in(System::April2015, |score| score.mods & 8 == 0);
    if found.is_empty() {
        return;
    }
    let (within, worst, _) = report("2015 system, no HD", &found);
    assert!(within >= 0.99, "only {:.1}% within 0.1%", within * 100.0);
    assert!(worst <= 0.005, "worst error {:.2}%", worst * 100.0);
}
