use crate::points::get_points;
use crate::points::BOSSES;
use crate::points::MODIFIERS;
use chrono::Days;
use chrono::{NaiveDate, NaiveDateTime};
use colored::*;
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
    let lines: Vec<(usize, String)> = timers
        .lines()
        .enumerate()
        .map(|(i, l)| {
            (
                i,
                l.expect("Line not read")
                    .split_whitespace()
                    .fold(String::new(), |acc, e| format!("{} {}", acc, e)),
            )
        })
        .filter(|(_, l)| !l.is_empty())
        .collect();

    // Replace boss aliases/misspellings
    let lines: Vec<(usize, String)> = lines
        .into_iter()
        .map(|(i, line)| {
            (
                i,
                bosses_renames
                    .iter()
                    .fold(line, |acc, (original, replacement)| {
                        acc.replace(original, replacement)
                    }),
            )
        })
        .collect();

    let args: Vec<String> = env::args().collect();

    let program_name = match env::consts::OS {
        "windows" => r".\dkp-automator.exe",
        _ => r"./dkp-automator",
    };

    let usage_string = format!("Usage:

{}       Calculates dkp for all lines in timers.txt
{}   Calculates dkp for a 7-day period from the date given
                        Start date must be in the format \"day short-month year\" e.g. \"2 Jun 2024\"
                        You can also include a 24-hour time, e.g. \"2 Jun 2024 18:00\"",
                        format!("{program_name} all").bold(),
                        format!("{program_name} <start>").bold()
            );

    if args.len() != 2 {
        println!(
            "
Incorrect number of arguments.

{usage_string}
"
        );
        process::exit(1);
    }

    let arg = &args[1];

    if arg == "all" {
        lines.to_vec()
    } else {
        let start_date = if let Ok(without_time) = NaiveDate::parse_from_str(arg, "%d %b %Y") {
            without_time.and_hms_opt(0, 0, 0).unwrap()
        } else if let Ok(withtime) = NaiveDateTime::parse_from_str(arg, "%d %b %Y %H:%M") {
            withtime
        } else {
            println!(
                "
Invalid argument `{arg}`.

{usage_string}
"
            );
            process::exit(1);
        };

        let end_date = start_date.checked_add_days(Days::new(7)).unwrap();

        let mut start_index = 0;
        let mut end_index = 0;

        for (index, (_, line)) in lines.iter().enumerate().rev() {
            let as_date = match get_date(line) {
                Some(date) => date,
                None => continue,
            };

            if end_index == 0 {
                if as_date <= end_date {
                    end_index = index;
                }
            } else if as_date < start_date {
                start_index = index + 1;
                break;
            }
        }

        lines[start_index..end_index + 1].to_vec()
    }
}

type Line = (usize, String);

fn check_dates(lines: &Vec<Line>) -> Vec<usize> {
    let mut error_lines: Vec<usize> = Vec::new();

    for (index, line) in lines {
        if get_date(line).is_none() {
            error_lines.push(index + 1);
        }
    }

    error_lines
}

fn get_date(line: &str) -> Option<NaiveDateTime> {
    if line.len() < 20 {
        return None;
    }

    let date = &line[..20].trim();
    let fmt = "%d %b %Y at %H:%M";
    match NaiveDateTime::parse_from_str(date, fmt) {
        Ok(date) => Some(date),
        Err(_) => {
            if line.len() < 24 {
                return None;
            }

            let date = &line[..24].trim();
            let fmt = "%b %d, %Y at %I:%M %p";
            match NaiveDateTime::parse_from_str(date, fmt) {
                Ok(date) => Some(date),
                Err(_) => None,
            }
        }
    }
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
    let mut incorrect_use_of_not_lines = Vec::<usize>::new();
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

        let boss = full_line.remove(0);
        if let Some(points) = get_points(&boss) {
            if full_line.contains(&"at".to_string()) {
                error_at_lines.push(index + 1)
            }

            if full_line.contains(&"not".to_string()) {
                if full_line.len() == 3 {
                    if full_line[1] != "not" {
                        incorrect_use_of_not_lines.push(index + 1);
                    }
                } else if full_line.len() == 2 {
                    if full_line[0] != "not" {
                        incorrect_use_of_not_lines.push(index + 1);
                    }
                } else {
                    incorrect_use_of_not_lines.push(index + 1);
                }
            }

            error_single_character_name_lines
                .extend(full_line.iter().filter(|n| n.len() == 1).map(|_| index + 1));

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
    if !incorrect_use_of_not_lines.is_empty() {
        ready = false;
        println!("Incorrect use of 'not' in lines:");
        for line in incorrect_use_of_not_lines {
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
