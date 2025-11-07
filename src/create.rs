use std::sync::Arc;

use colored::Colorize;

use crate::fetch::{ProblemContent, ProblemJSON};
use crate::logger::Log;
use crate::parse_api::{ProbMetaData, ScalarType};
use crate::test_module::create_test_module::try_create_test_module;
use crate::{fetch, read_write, LOGGER};

use crate::args::CreateCommand;

pub const UNEXPECTED_ERR_HEADER: &str = "Unexpected error while";
const ERR_HEADER: &str = "Error while";
const PREMIUM_COMMAND: &str = "`cargo conf premium (0 or 1)`";
const ALLOW_DEAD_CODE_COMMAND: &str = "`cargo conf allow-dead-code (0 or 1)`";

pub const NO_PREMIUM_ERR: &str = "This problem seems to be premium-only but you are registered as a free-user. Please run `cargo conf premium 1` if you are premium.";

const TEST_COMPILER_CONFIGURATION_ATTRIBUTE: &str = "#[cfg(test)]\n";
const PATTERNS_TO_GIVE_TEST_ATTRIBUTE: [&str; 6] = [
    "impl ",
    "struct ",
    "use std::cell::RefCell;",
    "use std::rc::Rc;",
    "use crate::tree::TreeNode;",
    "use crate::linked_list::ListNode;",
];

pub const SOLUTION_STRUCT_PATTERN: &str = "struct Solution;";
pub const USE_TREENODE_PATTERN: &str = "use crate::tree::TreeNode;";
pub const USE_LISTNODE_PATTERN: &str = "use crate::linked_list::ListNode;";

pub async fn handle_create_command(create: CreateCommand) -> Result<(), ()> {
    LOGGER.change_verbosity(create.verbose);

    let premium = try_checking_if_user_is_premium()?;

    let allow_dead_code = try_checking_if_user_wants_allow_dead_code()?;

    let slug_option = read_write::try_read_slug_locally(create.problem_id, premium)?;

    let slug = if let Some(slug) = slug_option {
        slug
    } else {
        println!("Couldn't find slug locally, trying to fetch it...");
        try_fetch_slug(create.problem_id, premium).await?
    };

    let slug = Arc::from(slug);
    let problem_content = tokio::spawn(fetch::try_fetch_content(Arc::clone(&slug)));
    let example_testcases = tokio::spawn(fetch::try_fetch_example_testcases(Arc::clone(&slug)));

    let (problem_content, example_testcases) = tokio::try_join!(problem_content, example_testcases)
        .map_err(|e| println!("error joining the fetch handles: {e}"))?;

    let problem_content = problem_content.map_err(|e| {
        e.log(create.problem_id);
    })?;
    let example_testcases = example_testcases.map_err(|e| {
        e.log(create.problem_id);
    })?;

    let test_module = try_create_test_module(&example_testcases, &problem_content.metadata)?;

    try_creating_solution_file(
        problem_content,
        &test_module,
        create.problem_id,
        &slug,
        allow_dead_code,
    );

    let url = format!("https://leetcode.com/problems/{}", slug);
    println!("URL: {}", url);

    Ok(())
}

/// Starts by fetching all problems slugs from leetcode's api, then update `slugs_and_ids.txt`, then looks for the given problem's slug
///
/// ### Arguments
///
/// * `problem_id` - The (front-end) id of the problem
/// * `premium` - The boolean describing if the user declared himself as premium or not
async fn try_fetch_slug(problem_id: u16, premium: bool) -> Result<String, ()> {
    let all_result = fetch::try_fetch_problem_list().await;
    let res = handle_problems_fetch_and_find(all_result, problem_id);
    if let Some((slug, prem)) = res {
        if prem && !premium {
            println!("{}", NO_PREMIUM_ERR);
            Err(())
        } else {
            Ok(slug)
        }
    } else {
        println!(
            "We could not find problem {}, please double-check its number.",
            problem_id
        );
        Err(())
    }
}

/// Reads the `.env` file and searches if the user is premium
fn try_checking_if_user_is_premium() -> Result<bool, ()> {
    LOGGER.log("Checking if you're premium...");
    read_write::try_reading_boolean_env_variable(
        "premium",
        PREMIUM_COMMAND,
        "Please specify if you're a premium leetcode user",
    )
}

/// Reads the `.env` file and searches if the user is premium
fn try_checking_if_user_wants_allow_dead_code() -> Result<bool, ()> {
    LOGGER.log("Checking how you'd like to escape rust's dead code warnings...");
    read_write::try_reading_boolean_env_variable("allow_dead_code", ALLOW_DEAD_CODE_COMMAND, "Please specify if you'd like to escape rust's warnings by using the #[allow(dead_code)] attribute")
}

fn try_creating_solution_file(
    problem_content: ProblemContent,
    test_module: &str,
    problem_id: u16,
    slug: &str,
    allow_dead_code: bool,
) {
    LOGGER.log("Trying to create a solution file...");
    let mut content = apply_modifications_to_solution_file(
        problem_content.default_code,
        problem_content.metadata,
        allow_dead_code,
    );
    content.push_str(test_module);
    let filename = format!("s{}_{}", problem_id, slug).replace('-', "_");
    let file_path = format!("./src/solutions/{}.rs", filename);

    let write_file_res = read_write::try_write_solution_template(&file_path, &content); // we check that it doesn't already exist
    match write_file_res {
        Ok(_) => {
            LOGGER.success(&format!("Created {}", file_path));
            // declares the newly created module file so that it is included in unit tests
            let append_mod_res = read_write::try_append_solution_module(&filename, allow_dead_code);
            if let Err(e) = append_mod_res {
                println!(
                    "{} declaring module of solution file: {}",
                    UNEXPECTED_ERR_HEADER.red().bold(),
                    e
                );
            }
        }
        Err(e) => {
            println!("{} creating solution file: {}", ERR_HEADER, e);
        }
    }
}

fn apply_modifications_to_solution_file(
    content: String,
    metadata: ProbMetaData,
    allow_dead_code: bool,
) -> String {
    let mut content = if let ProbMetaData::Function(_) = metadata {
        format!("{}{}", SOLUTION_STRUCT_PATTERN, content)
    } else {
        content
    };

    if let ProbMetaData::Function(function_metadata) = metadata {
        let has_tree_node = function_metadata
            .params
            .iter()
            .any(|param| param._type.scalar_type == ScalarType::TreeNode);
        if has_tree_node {
            content = format!("{}{}", USE_TREENODE_PATTERN, content)
        }
        let has_list_node = function_metadata
            .params
            .iter()
            .any(|param| param._type.scalar_type == ScalarType::ListNode);
        if has_list_node {
            content = format!("{}{}", USE_LISTNODE_PATTERN, content)
        }
    }

    if !allow_dead_code {
        for pattern in PATTERNS_TO_GIVE_TEST_ATTRIBUTE {
            let concat = format!("{}{}", TEST_COMPILER_CONFIGURATION_ATTRIBUTE, pattern);
            content = content
                .lines()
                .map(|line| {
                    if line.starts_with("// ") {
                        line.to_string()
                    } else {
                        line.replace(pattern, &concat)
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
        }
    }

    content
}

fn handle_problems_fetch_and_find(
    result: Result<Vec<ProblemJSON>, Box<dyn std::error::Error>>,
    problem_id: u16,
) -> Option<(String, bool)> {
    match result {
        Ok(problems) => {
            let mut problem: Option<(String, bool)> = None;
            let new_slugs_and_ids_content = problems
                .iter()
                .map(|prob| {
                    if prob.problem_stat.frontend_question_id == problem_id {
                        problem = Some((
                            prob.problem_stat.question_title_slug.clone(),
                            prob.paid_only,
                        ));
                    }
                    format!(
                        "{},{},{}",
                        prob.problem_stat.frontend_question_id,
                        prob.problem_stat.question_title_slug,
                        match prob.paid_only {
                            true => 1,
                            false => 0,
                        }
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");
            let res = read_write::try_write_slugs_and_ids(new_slugs_and_ids_content);
            if let Err(e) = res {
                println! {"{} writing all problems to slugs_and_ids: {}", UNEXPECTED_ERR_HEADER.red().bold(), e};
            }
            problem
        }
        Err(e) => {
            println!(
                "{} fecthing all problems: {}",
                UNEXPECTED_ERR_HEADER.red().bold(),
                e
            );
            None
        }
    }
}
