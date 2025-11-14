use anyhow::Error;
use pico_args::Arguments;

#[derive(Clone, Debug)]
pub struct Cli {
    pub command: Command,
}

const HELP: &str = "\
snoopy

USAGE:
  snoopy <SOURCE_INTERFACE> <TARGET_INTERFACE>

FLAGS:
  -h, --help            Prints help information

ARGS:
  <SOURCE_INTERFACE>           Interface where we snoop the arp requests
  <TARGET_INTERFACE>           Interface where we insert the neighbor record
";

impl Cli {
    pub fn args() -> Result<Self, Error> {
        let mut args = Arguments::from_env();

        if args.contains(["-h", "--help"]) {
            print!("{}", HELP);
            std::process::exit(0);
        }

        let args = RunArgs {
            source_interface: args.free_from_str()?,
            target_interface: args.free_from_str()?,
        };

        let command = Command::Run(args);

        Ok(Self { command })
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    Run(RunArgs),
}

#[derive(Clone, Debug)]
pub struct RunArgs {
    pub source_interface: String,
    pub target_interface: String,
}
