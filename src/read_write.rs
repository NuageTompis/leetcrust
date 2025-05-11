use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::Path,
};

const ENV_PATH: &str = ".env";
pub const SLUGS_PATH: &str = "resources/slugs_and_ids.txt";
const SOLUTION_MOD_PATH: &str = "./src/solutions/mod.rs";

/// Possible results of local read (`LocalReadResult`)
#[derive(Debug)]
pub enum LRR<T> {
    Found(T),
    FileMissing,
    LineMissing,
    LineCorrupted,
    UnexpectedError,
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

pub fn try_append_solution_module(filename: &str) -> Result<(), io::Error> {
    let mod_file = Path::new(SOLUTION_MOD_PATH);

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(mod_file)
        .or_else(|_| File::create(mod_file))
        .unwrap();

    let content = format!("mod {};\n", filename);
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

pub fn try_read_variable<T: ReadVarRes>(variable: &str, separator: char) -> LRR<T> {
    match fs::read_to_string(T::file_path()) {
        Ok(content) => {
            for line in content.lines() {
                let ndx = line.find(separator);
                if let Some(ndx) = ndx {
                    if &line[..ndx] == variable {
                        let parts: Vec<&str> = line.split(separator).collect();
                        if let Some(parsed) = T::parse(&parts) {
                            return LRR::Found(parsed);
                        } else {
                            return LRR::LineCorrupted;
                        }
                    }
                }
            }
            LRR::LineMissing
        }
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                LRR::FileMissing
            } else {
                LRR::UnexpectedError
            }
        }
    }
}
