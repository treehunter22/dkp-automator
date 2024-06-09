use crate::points::get_points;
use crate::points::BOSSES;
use crate::points::MODIFIERS;
use chrono::format::ParseError;
use chrono::{NaiveDate, NaiveDateTime};
use serde_json::from_reader;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

fn pre_process_lines(timers: BufReader<File>) -> Vec<(usize, String)> {
    let bosses_input = File::open("boss_aliases.json").expect("Cannot find boss_aliases.json");
    let bosses = BufReader::new(bosses_input);

    let bosses_renames: Vec<HashMap<String, String>> =
        from_reader(bosses).expect("boss_aliases.json does not contain valid json");
    let bosses_renames: Vec<(String, String)> = bosses_renames
        .into_iter()
        .flat_map(|b| b.into_iter())
        .collect();

    // Tidy lines
    let lines: Vec<String> = timers
        .lines()
        .map(|l| {
            l.expect("Line not read")
                .to_lowercase()
                .replace("  ", " ")
                .trim_end()
                .to_string()
        })
        .collect();

    // Replace boss aliases/misspellings
    let lines: Vec<(usize, String)> = lines
        .into_iter()
        .map(|line| {
            bosses_renames
                .iter()
                .fold(line, |acc, (original, replacement)| {
                    acc.replace(original, replacement)
                })
        })
        .enumerate()
        .collect();

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!(
            "Incorrect number of arguments. Has {}, expected 2",
            args.len() - 1
        );
        process::exit(1);
    }

    let start_date = NaiveDate::parse_from_str(&args[1], "%d %b %Y").expect("Invalid start date");

    let end_date = NaiveDate::parse_from_str(&args[2], "%d %b %Y").expect("Invalid end date");

    let mut start_index = 0;
    let mut end_index = 0;

    for (index, line) in lines.iter().rev() {
        if line.len() < 20 {
            continue;
        }

        let date = &line[..11].trim();
        let fmt = "%d %b %Y";
        let as_date = match NaiveDate::parse_from_str(date, fmt) {
            Ok(date) => date,
            Err(_) => {
                continue;
            }
        };

        if end_index == 0 {
            if as_date <= end_date {
                end_index = *index;
            }
        } else if as_date < start_date {
            start_index = *index + 1;
            break;
        }
    }

    lines[start_index..end_index + 1].to_vec()
}

type Line = (usize, String);

fn check_dates(lines: &Vec<Line>) -> Vec<usize> {
    let mut error_lines: Vec<usize> = Vec::new();

    for (index, line) in lines {
        match check_date(line) {
            Ok(_) => continue,
            Err(_) => error_lines.push(index + 1),
        }
    }

    error_lines
}

fn check_date(line: &str) -> Result<(), ParseError> {
    let date_slice = line[..20].trim();
    let fmt = "%d %b %Y at %H:%M";
    let _ = NaiveDateTime::parse_from_str(date_slice, fmt)?;

    Ok(())
}

fn first_index_of_boss(line: &str, bosses: &Vec<String>) -> usize {
    let mut min = usize::max_value();
    for boss in bosses {
        if let Some(i) = line.find(boss) {
            if i < min {
                min = i;
            }
        }
    }

    if min < usize::max_value() {
        min
    } else {
        0
    }
}

pub fn get_valid_lines() -> Option<Vec<(i32, Vec<String>)>> {
    let timers_input = File::open("timers.txt").expect("Cannot find timers.txt");
    let timers = BufReader::new(timers_input);

    let lines = pre_process_lines(timers);

    let error_date_lines = check_dates(&lines);
    let mut error_boss_lines = Vec::<usize>::new();
    let mut error_at_lines = Vec::<usize>::new();
    let mut error_single_character_name_lines = Vec::<usize>::new();
    let mut general_error_lines = Vec::<usize>::new();

    let boss_lines: Vec<(usize, Vec<String>)> = lines
        .iter()
        .map(|l| {
            (
                l.0,
                l.1[first_index_of_boss(&l.1, &BOSSES)..]
                    .to_string()
                    .split_whitespace()
                    .map(str::to_string)
                    .collect(),
            )
        })
        .collect();

    let mut formatted_lines = Vec::<(i32, Vec<String>)>::new();

    for (index, line) in boss_lines.iter() {
        let mut full_line = line.clone();

        let modifier = if full_line.len() < 2 {
            general_error_lines.push(index + 1);
            continue;
        } else {
            full_line.remove(1)
        };

        let mut is_valid_modifier = false;

        for test in MODIFIERS {
            if modifier == format!("({})", test) {
                is_valid_modifier = true;
            }
        }

        if is_valid_modifier {
            full_line[0].push_str(&modifier)
        } else {
            full_line.insert(1, modifier)
        }

        if full_line.contains(&"at".to_string()) {
            error_at_lines.push(index + 1)
        }

        error_single_character_name_lines
            .extend(full_line.iter().filter(|n| n.len() == 1).map(|_| index + 1));

        let boss = full_line.remove(0);
        if let Some(points) = get_points(&boss) {
            formatted_lines.push((points, full_line));
        } else {
            error_boss_lines.push(index + 1)
        }
    }

    let mut ready = true;

    if !error_date_lines.is_empty() {
        ready = false;
        println!("Cannot read date in lines:");
        for line in error_date_lines {
            println!("{}", line);
        }
    }

    if !error_boss_lines.is_empty() {
        ready = false;
        println!("Cannot read boss in lines:");
        for line in error_boss_lines {
            println!("{}", line);
        }
    }
    if !error_at_lines.is_empty() {
        ready = false;
        println!("Word 'at' in lines:");
        for line in error_at_lines {
            println!("{}", line);
        }
    }
    if !error_single_character_name_lines.is_empty() {
        ready = false;
        println!("Single character name in lines:");
        for line in error_single_character_name_lines {
            println!("{}", line);
        }
    }
    if !general_error_lines.is_empty() {
        ready = false;
        println!("Error at lines:");
        for line in general_error_lines {
            println!("{}", line);
        }
    }

    if ready {
        Some(formatted_lines)
    } else {
        None
    }
}
