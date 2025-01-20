use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::BufReader};

pub const MODIFIERS: [&str; 5] = ["brucybonus", "double", "doublepoints", "fail", "comp"];

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[derive(Debug)]
enum Point {
    Value(i32),
    Rings(HashMap<String, i32>),
    Legacy(Vec<Tier>),
}

#[derive(Serialize, Deserialize, Debug)]
struct Tier {
    #[serde(rename = "5")]
    point_5: i32,
    #[serde(rename = "6")]
    point_6: i32,
    level: i32,
}

lazy_static! {
    static ref POINTS_MAP: HashMap<String, Point> = {
        let points_input = File::open("points.json").expect("Cannot find points.json");
        serde_json::from_reader(BufReader::new(points_input))
            .expect("Cannot load point values from points.json")
    };
}

lazy_static! {
    pub static ref BOSSES: Vec<String> = POINTS_MAP.keys().map(String::clone).collect();
}

lazy_static! {
    static ref PRIOS: Vec<String> = {
        let prios_input = File::open("prios.json").expect("Cannot find prios.json");
        serde_json::from_reader(BufReader::new(prios_input))
            .expect("Cannot load bosses from prios.json")
    };
}

trait RemFirstAndLast {
    fn rem_first_and_last(&self) -> String;
}

impl RemFirstAndLast for String {
    fn rem_first_and_last(&self) -> String {
        let mut chars = self.chars();
        chars.next();
        chars.next_back();
        chars.collect()
    }
}

pub fn get_points(boss: &str) -> Option<i32> {
    static RINGS_CAPTURE_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new("^rings(?<num>[1-4])x(?<star>[5-6])$").expect("Invalid rings regex")
    });

    static RINGS_MATCH_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new("rings([1-4])x([5-6])").expect("Invalid rings regex"));

    static LEGACY_CAPTURE_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^legacy(?<level>\d+)\.(?<star>[5-6])$").expect("Invalid legacy regex")
    });

    static LEGACY_MATCH_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"legacy(\d+)\.([5-6])").expect("Invalid legacy regex"));

    static ROOT_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^root\d*$").expect("Invalid roots regex"));

    static BOSSES_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(&format!(
            r"^(?<boss>{})(\((?<modifier>{})\))?$",
            {
                let mut temp = BOSSES
                    .iter()
                    .filter_map(|b| match POINTS_MAP.get(b).unwrap() {
                        Point::Value(_) => Some(b.replace('.', r"\.")),
                        _ => None,
                    })
                    .collect::<Vec<String>>();

                temp.push(RINGS_MATCH_RE.to_string());
                temp.push(LEGACY_MATCH_RE.to_string());
                temp.push(ROOT_RE.to_string().rem_first_and_last());
                temp.join("|")
            },
            { MODIFIERS.join("|") }
        ))
        .expect("Failed to create regex")
    });

    let caps = BOSSES_RE.captures(boss)?;

    let mut points = 0;
    let mut double = false;
    let mut half = false;

    let stripped_boss = &caps["boss"];
    if let Some(modifier) = &caps.name("modifier") {
        match modifier.as_str() {
            "brucybonus" => points += 5,
            "doublepoints" => double = true,
            "double" => double = true,
            "fail" => half = true,
            "comp" => {
                let mut is_prio = false;
                for prio in PRIOS.iter() {
                    if boss.contains(prio) {
                        is_prio = true;
                        break;
                    }
                }
                if !is_prio {
                    return None;
                }
            }
            _ => (),
        }
    }

    match POINTS_MAP.get(stripped_boss) {
        Some(point) => match point {
            Point::Value(val) => points += val,
            Point::Rings(_) => return None,
            Point::Legacy(_) => return None,
        },
        None => {
            if let Some(caps) = RINGS_CAPTURE_RE.captures(stripped_boss) {
                let num: i32 = caps["num"].parse::<i32>().unwrap();
                let star = &caps["star"];

                if let Point::Rings(ring) = POINTS_MAP
                    .get("rings")
                    .expect("Cannot find legacy in points.json")
                {
                    points += ring[star] * num;
                }
            } else if let Some(caps) = LEGACY_CAPTURE_RE.captures(stripped_boss) {
                let level: i32 = caps["level"].parse().unwrap();
                let star = &caps["star"];

                if let Point::Legacy(tiers) = POINTS_MAP
                    .get("legacy")
                    .expect("Cannot find legacy in points.json")
                {
                    for tier in tiers {
                        if level >= tier.level {
                            points += if star == "5" {
                                tier.point_5
                            } else {
                                tier.point_6
                            };
                            break;
                        }
                    }
                }
            } else if ROOT_RE.is_match(stripped_boss) {
                points += 4
            } else {
                return None;
            }
        }
    };

    if double {
        points *= 2
    }

    if half {
        points = (points + 1) / 2;
    }

    Some(points)
}
