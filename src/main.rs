use exitfailure::ExitFailure;
use structopt::StructOpt;

mod scrivener;
use scrivener::args::Args;

fn main() -> Result<(), ExitFailure> {
    const PROGRAM_NAME: &str = "scrivener";

    let args = Args::from_args();

    args.execute(PROGRAM_NAME)?;

    Ok(())
}
