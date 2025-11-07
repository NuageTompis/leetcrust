use std::fs;

use clipboard::{ClipboardContext, ClipboardProvider};
use colored::Colorize;

use crate::{
    args::ClipCommand,
    create::{
        SOLUTION_STRUCT_PATTERN, UNEXPECTED_ERR_HEADER, USE_LISTNODE_PATTERN, USE_TREENODE_PATTERN,
    },
    logger::Log,
    read_write, LOGGER,
};

const PATTERNS_TO_REMOVE: [&str; 4] = [
    "#[cfg(test)]",
    SOLUTION_STRUCT_PATTERN,
    USE_TREENODE_PATTERN,
    USE_LISTNODE_PATTERN,
];

pub async fn handle_clip_command(clip: ClipCommand) -> Result<(), ()> {
    LOGGER.change_verbosity(clip.verbose);

    LOGGER.log("Trying to find slug locally...");
    // the premium value doesn't matter here
    let slug_option = read_write::try_read_slug_locally(clip.problem_id, true)?;
    let slug = slug_option.ok_or(())?;

    let filename = format!("s{}_{}", clip.problem_id, slug).replace('-', "_");
    let file_path = format!("./src/solutions/{}.rs", filename);
    let read_file_res = fs::read_to_string(&file_path);

    LOGGER.log("Reading solution file...");
    let mut content = match read_file_res {
        Ok(content) => Ok(content),
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    println!("You're trying to clip the content of your solution for problem {} but it seems you haven't solved it yet !", clip.problem_id);
                }
                e => {
                    println!(
                        "{} reading content of solution file for problem {}: {}",
                        UNEXPECTED_ERR_HEADER.red().bold(),
                        clip.problem_id,
                        e
                    )
                }
            }
            Err(())
        }
    }?;

    for pattern in PATTERNS_TO_REMOVE {
        content = content.replace(pattern, "");
    }

    content = remove_test_module(content);
    content = content.trim().to_owned();

    LOGGER.log("Trying to set solution to your clipboard...");
    let ctx_res = ClipboardProvider::new();
    let mut ctx: ClipboardContext = match ctx_res {
        Ok(ctx) => ctx,
        Err(e) => {
            println!(
                "{} creating clipboard context: {}",
                UNEXPECTED_ERR_HEADER.red().bold(),
                e
            );
            return Err(());
        }
    };

    let setting_clipboard_res = ctx.set_contents(content);

    if let Err(e) = setting_clipboard_res {
        println!("Error setting solution to clipboard: {}", e);
        return Err(());
    }

    let _ = ctx.get_contents(); // not reading clipboard content after setting it doesn't seem to work in ubuntu, so this should be kept as-is

    LOGGER.success("Added solution to clipboard!");

    Ok(())
}

fn remove_test_module(mut input: String) -> String {
    let test_module_pattern = "mod tests {";
    if let Some(start) = input.find(test_module_pattern) {
        let mut brace_count = 0;
        let mut in_line_comment = false;
        let mut in_block_comment = false;
        let mut chars = input[start..].char_indices().peekable();
        let mut end = None;

        while let Some((i, c)) = chars.next() {
            let true_ndx = start + i;
            let next_char = chars.peek().map(|&(_ndx, c)| c);

            // check start of line comment
            if !in_block_comment && !in_line_comment && c == '/' && next_char == Some('/') {
                in_line_comment = true;
                continue;
            }

            // check start of block comment
            if !in_block_comment && !in_line_comment && c == '/' && next_char == Some('*') {
                in_block_comment = true;
                chars.next();
                continue;
            }

            // check end of block comment
            if in_block_comment && c == '*' && next_char == Some('/') {
                in_block_comment = false;
                chars.next();
                continue;
            }

            // check end of line comment
            if in_line_comment && c == '\n' {
                in_line_comment = false;
            }

            if !in_line_comment && !in_block_comment {
                if c == '{' {
                    brace_count += 1;
                } else if c == '}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end = Some(true_ndx);
                        break;
                    }
                }
            }
        }

        if let Some(end_idx) = end {
            input.replace_range(start..=end_idx, "");
        }
    }

    input
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_test_module() {
        let file_content = r#"fn greet() {
    println!("hello");
}

const XD = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert!(true);
    }

    // this bracket -> } should be ignored
    /* this one too } */
    /* and
    this
    *one } */
}"#
        .into();
        let res = remove_test_module(file_content);
        let expected = r#"fn greet() {
    println!("hello");
}

const XD = 4;

#[cfg(test)]
"#;
        assert_eq!(&res, expected);
    }
}
