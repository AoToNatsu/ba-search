use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Archive using WGET from a range
    Archive {
        /// Link to process with wget
        #[arg(short, long)]
        link: String,
        /// Start of range
        #[arg(short, long)]
        start: Option<usize>,
        /// End of range
        #[arg(short, long)]
        end: usize,
        /// Increment (positive or minus)
        #[arg(short, long)]
        increment: Option<usize>,
        /// Format string
        #[arg(short, long)]
        format: Option<String>,
        /// Omit the details output
        #[arg(short, long)]
        quiet: bool,
    },
    /// Search BA Wiki string from current directory
    Search {
        /// Input string
        #[arg(short, long)]
        input: String,
        /// Count instead of print
        #[arg(short, long)]
        count: bool,
        /// When searching for a pattern, ignore case differences
        #[arg(short = 'I', long)]
        ignore_case: bool,
        /// Include chapter summaries
        #[arg(short, long)]
        summary: bool,
        /// Require that all matches of the pattern be surrounded by word boundaries
        #[arg(short, long)]
        word_regexp: bool,
        /// Outline matches with asterisks
        #[arg(short, long)]
        outline: bool,
        /// If enabled, disable line numbers
        #[arg(short, long)]
        numbered: bool,
        /// Ignore Student lines
        #[arg(long)]
        student: bool,
        /// Ignore Sensei lines
        #[arg(long)]
        sensei: bool,
        /// Ignore Story Info lines
        #[arg(long)]
        info: bool,
        /// Ignore Episode Description lines
        #[arg(long)]
        description: bool,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.cmd {
        Cmd::Archive {
            link,
            start,
            end,
            increment,
            format,
            quiet,
        } => commands::archive::main()
            .link(link)
            .maybe_start(start)
            .end(end)
            .maybe_increment(increment)
            .maybe_format(format)
            .quiet(quiet)
            .call()?,
        Cmd::Search {
            input,
            count,
            ignore_case,
            summary,
            word_regexp,
            outline,
            numbered,
            student,
            sensei,
            info,
            description,
        } => commands::search::main()
            .input(input)
            .count(count)
            .ignore_case(ignore_case)
            .summary(summary)
            .word_regexp(word_regexp)
            .outline(outline)
            .numbered(numbered)
            .student(student)
            .sensei(sensei)
            .info(info)
            .description(description)
            .call()?,
    }

    Ok(())
}
