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

    /// Configure your information
    Config(ConfigCommand),

    /// Fetch something from leetcode's api
    #[clap(alias = "f")]
    Fetch(FetchCommand),

    /// Finds all types defined by leetcode (among the fetched metadata files)
    #[clap(alias = "t")]
    FindTypes,
}

#[derive(Args, Debug)]
pub struct CreateCommand {
    /// The problem's id
    pub problem_id: u16,
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

    /// Fetch problems randomly, keeping only their metadata and storing them in directory resources/metadata
    Metadata(MetadataCommand),

    /// Not implemented yet
    Unimplemented,
}

#[derive(Args, Debug)]
pub struct MetadataCommand {
    /// The amount of metadata that should be fetched before terminating (including previous calls)
    pub target: usize,
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
