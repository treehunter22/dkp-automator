#[macro_use]
extern crate lazy_static;
extern crate google_sheets4 as sheets4;

use autocorrect::Autocorrecter;
use serde_json::from_reader;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, stdout, BufReader, Write};
use std::process;

pub mod autocorrect;
pub mod points;
pub mod sanitise;
pub mod sheets;

fn clear() {
    print!("\x1B[2J");
}

fn build_aliases(names: Vec<String>) -> HashMap<String, String> {
    let aliases_input = File::open("aliases.json").expect("Cannot find aliases.json");

    let mut aliases: HashMap<String, String> = from_reader(BufReader::new(aliases_input))
        .expect("aliases.json does not contain valid json");

    for name in names {
        if name.contains(' ') {
            let tmp: Vec<&str> = name.split_whitespace().collect();
            aliases.insert(tmp[0].to_lowercase(), name.clone());
            aliases.insert(tmp.join("").to_lowercase(), name.clone());
            aliases.insert(
                tmp.join("")
                    .trim_end_matches(char::is_numeric)
                    .to_lowercase(),
                name.clone(),
            );
        } else {
            aliases.insert(name.to_lowercase(), name.clone());
            if name.parse::<i32>().is_err() {
                aliases.insert(
                    name.trim_end_matches(char::is_numeric).to_lowercase(),
                    name.clone(),
                );
            }
        }
    }

    aliases
}

fn input(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = stdout().flush();
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .expect("Failed to read input");
    answer = answer.trim().to_string();
    println!("{}", answer);
    answer
}

#[tokio::main]
async fn main() {
    let Some(lines) = sanitise::get_valid_lines() else {
        return;
    };

    let Some(names) = sheets::get_names_from_sheets().await else {
        return;
    };

    let mut aliases = build_aliases(names);

    let mut dkp_count = HashMap::<String, i32>::new();

    let autocorrector: Autocorrecter = Autocorrecter::new(aliases.keys().cloned().collect());
    let mut discard = HashSet::<String>::new();

    for (points, names) in lines {
        let mut actual_names = Vec::<String>::new();

        'names: for name in names {
            if name == "not" {
                actual_names.push("not".to_string());
                continue;
            }

            if discard.contains(&name) {
                continue;
            }

            if name.len() <= 1 {
                continue;
            }

            if let Some(actual_name) = aliases.get(&name) {
                actual_names.push(actual_name.clone())
            } else {
                let guesses = autocorrector.correct(&name).unwrap();
                clear();
                println!("Error found: {}\n", &name);
                println!("Guess (1): {}", guesses[0]);
                println!("Guess (2): {}", guesses[1]);
                println!("Guess (3): {}", guesses[2]);
                println!("Guess (4): {}", guesses[3]);
                println!("Guess (5): {}", guesses[4]);
                println!("Enter a different name (6)");
                println!("Split into two names (7)");
                println!("Add as new name not already in the spreadsheet (8)");
                println!("Discard (Return)");
                println!("Enter Q to quit");

                let mut corrections = Vec::<String>::new();

                let mut answer: String;

                loop {
                    let mut valid_input = true;

                    answer = input("Select a choice (1-9): ");

                    match answer.as_str() {
                        "q" => process::exit(1),
                        "1" => corrections.push(guesses[0].to_string()),
                        "2" => corrections.push(guesses[1].to_string()),
                        "3" => corrections.push(guesses[2].to_string()),
                        "4" => corrections.push(guesses[3].to_string()),
                        "5" => corrections.push(guesses[4].to_string()),
                        "6" => corrections.push(input("Enter the name: ")),
                        "7" => {
                            corrections.push(input("Enter the first name: "));
                            corrections.push(input("Enter the second name: "));
                        }
                        "8" => {
                            let new_name = input("Enter the name: ");
                            aliases.insert(name.clone(), new_name);
                            actual_names.push(name.clone());
                            continue 'names;
                        }
                        "" => {
                            discard.insert(name.clone());
                        }
                        _ => {
                            println!("Invalid input, please enter a number between 1 and 8, q, or nothing.");
                            valid_input = false;
                        }
                    }

                    if valid_input {
                        break;
                    }
                }

                'outer: for mut correction in corrections {
                    loop {
                        if let Some(actual_name) = aliases.get(&correction).cloned() {
                            if answer != "7" {
                                aliases.insert(name.clone(), actual_name.clone());
                            }

                            actual_names.push(actual_name.clone());
                            break;
                        } else {
                            let correction_guesses = autocorrector.correct(&correction).unwrap();

                            println!("\nThe name {correction} is invalid.\n");
                            println!("Guess (1): {}", correction_guesses[0]);
                            println!("Guess (2): {}", correction_guesses[1]);
                            println!("Enter a different name (3)");
                            println!("Discard (4)");

                            let mut valid_input = false;
                            while !valid_input {
                                valid_input = true;
                                let answer2 = input("Select a choice (1-4): ");

                                match answer2.as_str() {
                                    "1" => correction = correction_guesses[0].to_string(),
                                    "2" => correction = correction_guesses[1].to_string(),
                                    "3" => correction = input("Enter the name: "),
                                    "4" => {
                                        discard.insert(name.clone());
                                        continue 'outer;
                                    }
                                    _ => valid_input = false,
                                }

                                if !valid_input {
                                    println!("Invalid input. Select a number bewteen 1 and 4.\n");
                                }
                            }
                        }
                    }
                }
            }
        }

        if actual_names.len() == 3 {
            if actual_names[1] == "not" {
                dkp_count
                    .entry(actual_names[0].clone())
                    .and_modify(|p| *p += points)
                    .or_insert(points);

                dkp_count
                    .entry(actual_names[2].clone())
                    .and_modify(|p| *p -= points)
                    .or_insert(points);

                continue;
            }
        } else if actual_names.len() == 2 && actual_names[0] == "not" {
            dkp_count
                .entry(actual_names[1].clone())
                .and_modify(|p| *p -= points)
                .or_insert(points);

            continue;
        }

        let cleaned_names: HashSet<String> =
            HashSet::from_iter(actual_names.into_iter().filter(|n| n.as_str() != "not"));
        for name in cleaned_names {
            dkp_count
                .entry(name)
                .and_modify(|p| *p += points)
                .or_insert(points);
        }
    }

    let mut dkp_count: Vec<(String, i32)> = dkp_count.into_iter().collect();

    dkp_count.sort();

    dkp_count.retain(|(_, p)| *p > 0);
    for (n, p) in dkp_count {
        println!("{}, {}", n, p);
    }
}
