use std::sync::Arc;

use crate::{
    args::{FetchCommand, FetchSubcommand},
    create::UNEXPECTED_ERR_HEADER,
    parse_api::ProbMetaData,
};
use colored::Colorize;
use serde::{Deserialize, Serialize};

pub const PROBLEM_LIST_URL: &str = "https://leetcode.com/api/problems/algorithms/";

// todo: find a way to refactor this
#[derive(Debug, Deserialize, Serialize)]
struct Json1 {
    data: Json2,
}

#[derive(Debug, Deserialize, Serialize)]
struct Json2 {
    question: Json3,
}
#[derive(Debug, Deserialize, Serialize)]
struct Json3 {
    #[serde(rename = "codeDefinition")]
    code_definition: String,
    #[serde(rename = "metaData")]
    meta_data: String,
}
#[derive(Debug, Deserialize, Serialize)]
struct Json4 {
    #[serde(rename = "defaultCode")]
    default_code: String,
    value: String,
}
// and this
#[derive(Debug, Deserialize, Serialize)]
struct Json1Bis {
    data: Json2Bis,
}

#[derive(Debug, Deserialize, Serialize)]
struct Json2Bis {
    question: Json3Bis,
}
#[derive(Debug, Deserialize, Serialize)]
struct Json3Bis {
    #[serde(rename = "exampleTestcases")]
    example_testcases: String,
}
//

#[derive(Debug, PartialEq)]
pub struct ProblemContent {
    pub default_code: String,
    pub metadata: ProbMetaData,
}

const QUERY_QUESTION_DATA: &str = "query questionData($titleSlug: String!) { question(titleSlug: $titleSlug) { codeDefinition metaData }}";
const QUERY_EXAMPLE_TESTCASES: &str = "query selectProblem($titleSlug: String!) { question(titleSlug: $titleSlug) { exampleTestcases }}";
const GRAPHQL_URL: &str = "https://leetcode.com/graphql";

#[derive(Debug, Serialize, Deserialize)]
struct Variables {
    #[serde(rename = "titleSlug")]
    title_slug: String,
}
impl Variables {
    fn new(title_slug: &str) -> Self {
        Variables {
            title_slug: title_slug.into(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
struct ExampleTestcasesQueryBody {
    variables: Variables,
    query: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct QuestionDataQueryBody {
    #[serde(rename = "operationName")]
    operation_name: String,
    variables: Variables,
    query: String,
}
impl QuestionDataQueryBody {
    fn new(title_slug: &str) -> Self {
        QuestionDataQueryBody {
            operation_name: "questionData".into(),
            variables: Variables::new(title_slug),
            query: QUERY_QUESTION_DATA.into(),
        }
    }
}
impl ExampleTestcasesQueryBody {
    fn new(title_slug: &str) -> Self {
        ExampleTestcasesQueryBody {
            variables: Variables::new(title_slug),
            query: QUERY_EXAMPLE_TESTCASES.into(),
        }
    }
}

pub fn handle_fetch_command(fetch: FetchCommand) {
    match fetch.command {
        FetchSubcommand::Slugs => {
            println!("This command is not implemented yet");
        }
        FetchSubcommand::Unimplemented => {
            println!("This command is not implemented yet");
        }
    }
}

pub async fn try_fetch_problem_list() -> Result<Vec<ProblemJSON>, Box<dyn std::error::Error>> {
    let result: ProblemListJSON = reqwest::Client::new()
        .get(PROBLEM_LIST_URL)
        .send()
        .await?
        .json()
        .await?;

    Ok(result.problems)
}

#[derive(Debug)]
pub enum FetchContentErr {
    NotFound,
    ReqwestErr(reqwest::Error),
    ParseError(serde_json::Error),
}

impl FetchContentErr {
    pub fn log(&self, problem_id: u16) {
        match self {
            FetchContentErr::NotFound => {
                println!(
                    "Problem {} doesn't seem to be available in Rust (T_T)",
                    problem_id
                );
            }
            FetchContentErr::ReqwestErr(error) => {
                println!(
                    "{} fetching content for problem {}: {}",
                    UNEXPECTED_ERR_HEADER.red().bold(),
                    problem_id,
                    error
                );
            }
            FetchContentErr::ParseError(error) => {
                println!(
                    "{} fetching content for problem {}: {}",
                    UNEXPECTED_ERR_HEADER.red().bold(),
                    problem_id,
                    error
                );
            }
        }
    }
}

// For unit tests
impl PartialEq for FetchContentErr {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (FetchContentErr::NotFound, FetchContentErr::NotFound)
                | (
                    FetchContentErr::ReqwestErr(_),
                    FetchContentErr::ReqwestErr(_)
                )
                | (
                    FetchContentErr::ParseError(_),
                    FetchContentErr::ParseError(_)
                )
        )
    }
}

pub async fn try_fetch_content(slug: Arc<str>) -> Result<ProblemContent, FetchContentErr> {
    let body = QuestionDataQueryBody::new(&slug);

    let response: Json1 = reqwest::Client::new()
        .post(GRAPHQL_URL)
        .json(&body)
        .send()
        .await
        .map_err(FetchContentErr::ReqwestErr)?
        .json()
        .await
        .map_err(FetchContentErr::ReqwestErr)?;

    let languages = response.data.question.code_definition;

    let parsed: Vec<Json4> =
        serde_json::from_str(&languages).map_err(FetchContentErr::ParseError)?;

    let mut default_code = None;
    for lang in &parsed {
        if lang.value == "rust" {
            default_code = Some(lang.default_code.clone());
        }
    }

    if let Some(default_code) = default_code {
        let parse_metadata_result =
            serde_json::from_str::<ProbMetaData>(&response.data.question.meta_data);
        match parse_metadata_result {
            Ok(metadata) => Ok(ProblemContent {
                default_code,
                metadata,
            }),
            Err(e) => Err(FetchContentErr::ParseError(e)),
        }
    } else {
        Err(FetchContentErr::NotFound)
    }
}

pub async fn try_fetch_example_testcases(slug: Arc<str>) -> Result<String, FetchContentErr> {
    let body = ExampleTestcasesQueryBody::new(&slug);

    let response: Json1Bis = reqwest::Client::new()
        .post(GRAPHQL_URL)
        .json(&body)
        .send()
        .await
        .map_err(FetchContentErr::ReqwestErr)?
        .json()
        .await
        .map_err(FetchContentErr::ReqwestErr)?;

    Ok(response.data.question.example_testcases)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProblemListJSON {
    #[serde(rename = "stat_status_pairs")]
    pub problems: Vec<ProblemJSON>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProblemJSON {
    #[serde(rename = "stat")]
    pub problem_stat: ProblemStatJSON,
    pub paid_only: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProblemStatJSON {
    pub frontend_question_id: u16,
    #[serde(rename = "question__title_slug")]
    pub question_title_slug: String,
}

#[cfg(test)]
mod tests_fetch {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn rust_unavailable() {
        let problem_slug = "find-a-corresponding-node-of-a-binary-tree-in-a-clone-of-that-tree"; // problem_id = 1379
        let content = try_fetch_content(Arc::from(problem_slug)).await;
        assert_eq!(content, Err(FetchContentErr::NotFound));
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_example_testcases() {
        let problem_slug = "find-a-corresponding-node-of-a-binary-tree-in-a-clone-of-that-tree";
        let example_testcases = try_fetch_example_testcases(Arc::from(problem_slug)).await;
        assert_eq!(
            example_testcases.unwrap(),
            "[7,4,3,null,null,6,19]\n3\n[7]\n7\n[8,null,6,null,5,null,4,null,3,null,2,null,1]\n4"
        );
    }
}
