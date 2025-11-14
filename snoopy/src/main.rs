// if have this cli tool, and it has some minimal logic that loads the bpf program, attaches it
// then loops and processes messages from it, currently its in a "run" module because I want to
// keep main clean, what would a better name than "run"?
use crate::cli::Cli;

mod bpf;
mod cli;
mod netlink;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::args()?;
    match args.command {
        cli::Command::Run(args) => {
            env_logger::init();

            bpf::attach(args.source_interface, args.target_interface).await?;
        }
    }

    Ok(())
}
