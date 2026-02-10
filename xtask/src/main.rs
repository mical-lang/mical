mod flags;

mod utils;
use utils::*;

mod commands;

use anyhow::Result;
use xshell::Shell;

fn main() -> Result<()> {
    let flags = {
        let cli_args = std::env::args_os().skip(1).collect::<Vec<_>>();
        flags::Xtask::from_vec(cli_args)?
    };
    let sh = Shell::new()?;
    match flags.subcommand {
        flags::XtaskCmd::Codegen(cmd) => cmd.run(&sh),
    }
}
