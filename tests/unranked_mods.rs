use rosu_pp::Beatmap;
use rosu_pp_older::{
    osu_2014_july, osu_2014_may, osu_2015_april, osu_2015_february, osu_2018, osu_2019,
};

const MAP: &str = "osu file format v14

[General]
Mode: 0

[Difficulty]
HPDrainRate:5
CircleSize:4
OverallDifficulty:8
ApproachRate:9
SliderMultiplier:1.4
SliderTickRate:1

[TimingPoints]
0,400,4,2,0,100,1,0

[HitObjects]
100,100,1000,1,0,0:0:0:0:
400,100,1400,1,0,0:0:0:0:
400,300,1800,1,0,0:0:0:0:
100,300,2200,1,0,0:0:0:0:
256,192,2600,2,0,L|456:192,1,200
";

const RELAX: u32 = 1 << 7;
const AUTOPLAY: u32 = 1 << 11;
const AUTOPILOT: u32 = 1 << 13;

fn pp_of_all(map: &Beatmap, mods: u32) -> [f64; 6] {
    [
        osu_2014_may::OsuPP::new(map).mods(mods).calculate().pp,
        osu_2014_july::OsuPP::new(map).mods(mods).calculate().pp,
        osu_2015_february::OsuPP::new(map).mods(mods).calculate().pp,
        osu_2015_april::OsuPP::new(map).mods(mods).calculate().pp,
        osu_2018::OsuPP::new(map).mods(mods).calculate().pp,
        osu_2019::OsuPP::new(map).mods(mods).calculate().pp,
    ]
}

#[test]
fn ranked_mods_give_pp() {
    let map = Beatmap::from_bytes(MAP.as_bytes()).unwrap();
    assert!(pp_of_all(&map, 0).iter().all(|pp| *pp > 0.0));
}

#[test]
fn relax_autopilot_and_autoplay_give_no_pp() {
    let map = Beatmap::from_bytes(MAP.as_bytes()).unwrap();
    for mods in [RELAX, AUTOPLAY, AUTOPILOT, RELAX | 8] {
        assert_eq!(pp_of_all(&map, mods), [0.0; 6], "mods {mods}");
    }
}
