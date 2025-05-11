use colored::Colorize;
use rand::rng;
use rand::seq::SliceRandom;

use std::{
    collections::HashSet,
    fs::{self},
    path::Path,
};

use crate::fetch::Json1;
use crate::{
    fetch::{FetchContentErr, QuestionDataQueryBody, GRAPHQL_URL},
    read_write::SLUGS_PATH,
};

const METADATA_DIR: &str = "resources/metadata/";

pub async fn handle_fetch_metadata_command(target: usize) {
    let mut problems = get_problems_list();
    let mut fetched_pbs = get_fetched_problems();
    let mut premium_pbs: HashSet<u16> = HashSet::new();

    let mut fetched_amt = fetched_pbs.len();
    if fetched_amt >= target {
        println!(
            "We already fetched {} problem metadata ! If you want to fetch {} more, type {}.",
            fetched_amt,
            target,
            fetched_amt + target
        );
        return;
    }

    problems.shuffle(&mut rng());

    for (k, pb) in problems.iter().enumerate() {
        if pb.is_premium {
            println!("Problem {} is premium-only", pb.id);
            premium_pbs.insert(pb.id);
        } else if fetched_pbs.contains(&pb.id) {
            println!("Problem {} was already fetched", pb.id);
        } else {
            println!(
                "Fetching problem {} ({}th problem considered) ...",
                pb.id,
                k + 1
            );
            let res = try_fetch_metadata(&pb.slug).await;
            match res {
                Ok(content) => {
                    let file_path = format!("{}{}.json", METADATA_DIR, pb.id);
                    let res = fs::write(file_path, content);
                    match res {
                        Ok(_) => {
                            fetched_amt += 1;
                            println!(
                                "{} created metadata file of problem {} ({}/{})",
                                "Successfully".cyan().bold(),
                                pb.id,
                                fetched_amt,
                                target
                            );
                            if fetched_amt == target {
                                break;
                            }
                            fetched_pbs.insert(pb.id);
                        }
                        Err(e) => {
                            println!("Error creating metadata file of problem {}: {}", pb.id, e);
                        }
                    }
                }
                Err(e) => {
                    println!("Error fetching metadata for problem {}: {:?}", pb.id, e);
                }
            }
        }
    }

    println!("Finished");
}

fn get_problems_list() -> Vec<Problem> {
    let content = fs::read_to_string(SLUGS_PATH);
    let content = match content {
        Ok(content) => content,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    println!(
                        "No slugs and ids file found, please create it by running 'cargo create 1'"
                    );
                }
                e => {
                    println!("Error reading slugs and ids file: {e}");
                }
            }
            panic!();
        }
    };

    let mut problems = Vec::new();
    for line in content.lines() {
        problems.push(Problem::new(line));
    }

    problems
}

fn get_fetched_problems() -> HashSet<u16> {
    let dir = Path::new(METADATA_DIR);

    let mut res = HashSet::new();

    for entry_result in fs::read_dir(dir).unwrap() {
        let entry = entry_result.unwrap();
        let path = entry.path();

        let path = path.to_str().unwrap();
        let parts: Vec<&str> = path.split('/').collect();
        let filename_full = *parts.last().unwrap();
        let parts: Vec<&str> = filename_full.split('.').collect();
        let file_id = parts[0].parse::<u16>().unwrap();

        res.insert(file_id);
    }

    res
}

#[derive(Debug, PartialEq)]
struct Problem {
    id: u16,
    slug: String,
    is_premium: bool,
}

impl Problem {
    fn new(line: &str) -> Self {
        let fields: Vec<&str> = line.split(',').collect();
        let id: u16 = fields[0].parse().unwrap();
        let slug = fields[1].into();
        let is_premium = match fields[2] {
            "0" => false,
            "1" => true,
            _ => panic!(),
        };
        Self {
            id,
            slug,
            is_premium,
        }
    }
}

pub async fn try_fetch_metadata(slug: &str) -> Result<String, FetchContentErr> {
    let body = QuestionDataQueryBody::new(slug);
    let response: Json1 = reqwest::Client::new()
        .post(GRAPHQL_URL)
        .json(&body)
        .send()
        .await
        .map_err(FetchContentErr::ReqwestErr)?
        .json()
        .await
        .map_err(FetchContentErr::ReqwestErr)?;

    Ok(response.data.question.meta_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_problems_list() {
        let res = get_problems_list();
        let expected_last = Problem {
            id: 1,
            slug: "two-sum".into(),
            is_premium: false,
        };
        let last = res.last().unwrap();
        assert_eq!(last, &expected_last);
    }

    #[test]
    fn test_parse_problem() {
        let res = Problem::new("3540,minimum-time-to-visit-all-houses,1");
        let expected = Problem {
            id: 3540,
            slug: "minimum-time-to-visit-all-houses".into(),
            is_premium: true,
        };
        assert_eq!(res, expected);
    }

    #[test]
    #[should_panic]
    fn test_parse_problem_panic() {
        let _ = Problem::new("3540,minimum-time-to-visit-all-houses,2");
    }
}
