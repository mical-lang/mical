mod flags;

use anyhow::Result;

fn main() -> Result<()> {
    let flags = {
        let cli_args = std::env::args_os().skip(1).collect::<Vec<_>>();
        flags::Xtask::from_vec(cli_args)?
    };
    println!("{:?}", flags);
    Ok(())
}
