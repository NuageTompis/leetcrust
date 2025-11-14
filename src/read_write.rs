use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::Path,
};

use colored::Colorize;

use crate::{
    create::{NO_PREMIUM_ERR, UNEXPECTED_ERR_HEADER},
    logger::Log,
    LOGGER,
};

const ENV_PATH: &str = ".env";
const SLUGS_PATH: &str = "resources/slugs_and_ids.txt";
const SOLUTION_MOD_PATH: &str = "./src/solutions/mod.rs";
const TEST_FUNCTION_MOLD_PATH: &str = "resources/test_function_mold.txt";
const TEST_FMODULE_MOLD_PATH: &str = "resources/test_module_mold.txt";

/// Possible results of local read
#[derive(Debug)]
pub enum LocalReadResult<T> {
    Found(T),
    FileMissing,
    LineMissing,
    LineCorrupted,
    Io(io::Error),
}

pub fn try_update_env_variable(variable: &str, value: &str) -> Result<(), io::Error> {
    let new_pair = format!("{}={}", variable, value);
    let content = fs::read_to_string(ENV_PATH);
    match content {
        Ok(content) => {
            let mut new_lines = vec![new_pair];
            for line in content.lines() {
                if !line.starts_with(variable) {
                    new_lines.push(line.into());
                }
            }
            fs::write(ENV_PATH, new_lines.join("\n"))?;
        }
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                fs::write(ENV_PATH, new_pair)?;
            } else {
                return Err(e);
            }
        }
    };
    Ok(())
}

/// Reads the `.env` file and searches for the given boolean variable (0 or 1)
///
/// Defaults to `false` if no unexpected error occurs
pub fn try_reading_boolean_env_variable(
    variable_name: &str,
    suggested_command: &str,
    explanation: &str,
) -> Result<bool, ()> {
    let variable_read_res: LocalReadResult<String> = try_read_variable(variable_name, '=');
    let value = match variable_read_res {
        LocalReadResult::Found(value) => {
            let parse_res = value.parse::<u8>();
            match parse_res {
                Ok(val) => match val {
                    0 => false,
                    1 => true,
                    _ => {
                        println!(
                            "{} reading {} bool from .env, it should be 0 or 1",
                            UNEXPECTED_ERR_HEADER.red().bold(),
                            variable_name
                        );
                        return Err(());
                    }
                },
                Err(_) => {
                    println!(
                        "{} parsing {} bool from .env, it should be 0 or 1",
                        UNEXPECTED_ERR_HEADER.red().bold(),
                        variable_name
                    );
                    return Err(());
                }
            }
        }
        LocalReadResult::FileMissing => {
            println!(
                "There is no .env file, please use {} to create it",
                suggested_command
            );
            false
        }
        LocalReadResult::LineMissing => {
            LOGGER.warning(&format!("{} by running {}", explanation, suggested_command));
            false
        }
        LocalReadResult::LineCorrupted => {
            println!(
                "The .env line with the {} variable seems to be corrupted. Please run {}",
                variable_name, suggested_command
            );
            false
        }
        LocalReadResult::Io(e) => {
            println!(
                "{} trying to read {} variable in .env: {e}",
                UNEXPECTED_ERR_HEADER.red().bold(),
                variable_name
            );
            false
        }
    };

    Ok(value)
}

pub fn try_read_test_function_mold() -> Result<String, io::Error> {
    fs::read_to_string(TEST_FUNCTION_MOLD_PATH)
}
pub fn try_read_test_module_mold() -> Result<String, io::Error> {
    fs::read_to_string(TEST_FMODULE_MOLD_PATH)
}

/// Tries finding the slug of a problem using the local file `slugs_and_ids.txt`
///
/// It will return an error if we sould abort (eg the problem is premium-only but the user is not premium), or an Option<String> if we should continue
///
/// ### Arguments
///
/// * `problem_id` - The (front-end) id of the problem
/// * `premium` - The boolean describing if the user declared himself as premium or not
pub fn try_read_slug_locally(problem_id: u16, premium: bool) -> Result<Option<String>, ()> {
    LOGGER.log("Trying to find slug locally...");
    let slug_read_res: LocalReadResult<(String, String)> =
        try_read_variable(&problem_id.to_string(), ',');
    let slug = match slug_read_res {
        LocalReadResult::Found((slug, prem)) => {
            // prem should be either 0 or 1
            let prem = match prem.parse::<u8>() {
                Ok(val) => match val {
                    0 => Some(false),
                    1 => Some(true),
                    _ => None,
                },
                Err(_) => None,
            };
            let prem = match prem {
                Some(prem) => prem,
                None => {
                    println!(
                        "{} parsing premium value locally for problem {}",
                        UNEXPECTED_ERR_HEADER.red().bold(),
                        problem_id
                    );
                    return Err(());
                }
            };
            if prem && !premium {
                println!("{}", NO_PREMIUM_ERR);
                return Err(());
            }
            Some(slug)
        }
        LocalReadResult::FileMissing => {
            println!("File `slugs_and_ids` missing, creating it...");
            None
        }
        LocalReadResult::LineMissing => None,
        LocalReadResult::LineCorrupted => {
            println!("We did find problem {} locally but the line seems to be corrupted. Starting api call...", problem_id);
            None
        }
        LocalReadResult::Io(e) => {
            println!(
                "{} reading slug locally: {e}.",
                UNEXPECTED_ERR_HEADER.red().bold()
            );
            None
        }
    };
    Ok(slug)
}

pub fn try_write_slugs_and_ids(content: String) -> io::Result<()> {
    fs::write(SLUGS_PATH, content)
}

pub fn try_write_solution_template(path: &str, content: &str) -> Result<(), io::Error> {
    if Path::new(path).exists() {
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "File already exists",
        ))
    } else {
        fs::write(path, content)
    }
}

pub fn try_append_solution_module(filename: &str, allow_dead_code: bool) -> Result<(), io::Error> {
    let mod_file = Path::new(SOLUTION_MOD_PATH);

    let mut file = OpenOptions::new()
        .append(true)
        .open(mod_file)
        .or_else(|_| File::create(mod_file))
        .unwrap();

    let mut content = if allow_dead_code {
        String::from("#[allow(dead_code)]\n")
    } else {
        String::new()
    };
    content.push_str(&format!("mod {};\n", filename));
    file.write_all(content.as_bytes())
}

// Trait to define behaviours on object we will be reading (eg slugs, premium value etc..)
pub trait ReadVarRes: Sized {
    fn parse(parts: &[&str]) -> Option<Self>;
    fn file_path() -> &'static str;
}
impl ReadVarRes for String {
    fn parse(parts: &[&str]) -> Option<Self> {
        if parts.len() == 2 {
            Some(parts[1].to_string())
        } else {
            None
        }
    }
    fn file_path() -> &'static str {
        ENV_PATH
    }
}
impl ReadVarRes for (String, String) {
    fn parse(parts: &[&str]) -> Option<Self> {
        if parts.len() == 3 {
            Some((parts[1].to_string(), parts[2].to_string()))
        } else {
            None
        }
    }
    fn file_path() -> &'static str {
        SLUGS_PATH
    }
}

pub fn try_read_variable<T: ReadVarRes>(variable: &str, separator: char) -> LocalReadResult<T> {
    let pattern = format!("{}{}", variable, separator);
    match fs::read_to_string(T::file_path()) {
        Ok(content) => {
            for line in content.lines() {
                if line.starts_with(&pattern) {
                    let parts: Vec<&str> = line.split(separator).collect();
                    if let Some(parsed) = T::parse(&parts) {
                        return LocalReadResult::Found(parsed);
                    } else {
                        return LocalReadResult::LineCorrupted;
                    }
                }
            }
            LocalReadResult::LineMissing
        }
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                LocalReadResult::FileMissing
            } else {
                LocalReadResult::Io(e)
            }
        }
    }
}
