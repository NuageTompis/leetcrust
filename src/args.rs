use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct LCArgs {
    #[clap(subcommand)]
    pub arg_type: MainCommand,
}

#[derive(Debug, Subcommand)]
pub enum MainCommand {
    /// Creates a solution file for the given problem, with default code and test cases
    #[clap(alias = "c")]
    Create(CreateCommand),

    /// Reads a solution file and puts the relevant content to your clipboard
    Clip(ClipCommand),

    /// Configure your information
    Config(ConfigCommand),

    /// Fetch something from leetcode's api
    #[clap(alias = "f")]
    Fetch(FetchCommand),
}

#[derive(Args, Debug)]
pub struct ClipCommand {
    /// The problem's id
    pub problem_id: u16,

    /// Flag to enable verbose output
    #[clap(long, short)]
    pub verbose: bool,
}

#[derive(Args, Debug)]
pub struct CreateCommand {
    /// The problem's id
    pub problem_id: u16,

    /// Flag to enable verbose output
    #[clap(long, short)]
    pub verbose: bool,
}

#[derive(Args, Debug)]
pub struct FetchCommand {
    #[clap(subcommand)]
    pub command: FetchSubcommand,
}

#[derive(Args, Debug)]
pub struct ConfigCommand {
    #[clap(subcommand)]
    pub command: ConfigSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum FetchSubcommand {
    /// Fetch each problem's id, slug and whether they're premium-only or not
    Slugs,

    /// Not implemented yet
    Unimplemented,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    /// Set up your leetcode's username
    Username(UsernameCommand),

    /// Set up your leetcode session cookie [not implemented yet]
    Cookie(CookieCommand),

    /// Tell leetcrust if you are a premium leetcode user (0 or 1)
    /// This will be used for better error-handling
    #[clap(verbatim_doc_comment)]
    Premium(PremiumCommand),

    /// Use the #[allow(dead_code)] attribute instead of the #[cfg(test)] one to escape rust's warnings
    AllowDeadCode(AllowDeadCodeCommand),
}

#[derive(Args, Debug)]
pub struct UsernameCommand {
    pub username: String,
}

#[derive(Args, Debug)]
pub struct CookieCommand {
    pub cookie: String,
}

#[derive(Args, Debug)]
pub struct PremiumCommand {
    /// 0 or 1
    pub premium: u8,
}

#[derive(Args, Debug)]
pub struct AllowDeadCodeCommand {
    /// 0 or 1
    pub allow_dead_code: u8,
}

pub trait GivenBooleanValue {
    fn get_name(&self) -> String;
    fn get_value(&self) -> u8;
    fn display_wrong_value(&self) {
        println!(
            "The {} value {} is not valid, it should be either 0 or 1",
            self.get_name(),
            self.get_value()
        )
    }
    fn is_valid(&self) -> bool {
        self.get_value() <= 1
    }
}

impl GivenBooleanValue for PremiumCommand {
    fn get_name(&self) -> String {
        String::from("premium")
    }

    fn get_value(&self) -> u8 {
        self.premium
    }
}

impl GivenBooleanValue for AllowDeadCodeCommand {
    fn get_name(&self) -> String {
        String::from("allow_dead_code")
    }

    fn get_value(&self) -> u8 {
        self.allow_dead_code
    }
}
