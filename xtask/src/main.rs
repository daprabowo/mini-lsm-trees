use clap::{Parser, Subcommand};
use console::style;
use duct::cmd;

#[derive(Subcommand, Debug)]
enum Action {
    /// Check,
    Check,
    /// Run CI jobs.
    Ci,
    /// Install necessary tools for development.
    InstallTools,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

fn fmt() -> anyhow::Result<()> {
    println!("{}", style("cargo fmt").bold());
    cmd!("cargo", "fmt").run()?;
    Ok(())
}

fn check_fmt() -> anyhow::Result<()> {
    println!("{}", style("cargo fmt --check").bold());
    cmd!("cargo", "fmt", "--check").run()?;
    Ok(())
}

fn check() -> anyhow::Result<()> {
    println!("{}", style("cargo check").bold());
    cmd!("cargo", "check").run()?;
    Ok(())
}

fn test() -> anyhow::Result<()> {
    println!("{}", style("cargo test").bold());
    cmd!("cargo", "test").run()?;
    Ok(())
}

fn clippy() -> anyhow::Result<()> {
    println!("{}", style("cargo clippy").bold());
    cmd!("cargo", "clippy").run()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.action {
        Action::Check => {
            fmt()?;
            check()?;
            test()?;
            clippy()?;
        }
        Action::Ci => {
            check_fmt()?;
            check()?;
            test()?;
            clippy()?;
        }
        Action::InstallTools => {
            println!("{}", style("cargo install cargo-nextest").bold());
            cmd!("cargo", "install", "cargo-nextest", "--locked").run()?;
            println!("{}", style("cargo install cargo-semver-checks").bold());
            cmd!("cargo", "install", "cargo-semver-checks", "--locked").run()?;
        }
    }

    Ok(())
}
