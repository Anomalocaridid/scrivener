//! Argument parsing logic

use failure::Error;
use structopt::StructOpt;

mod commands;
use crate::scrivener::notes::Index;
use commands::Command;

/// A struct that contains the arguments passed by the user.
#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(subcommand)]
    cmd: Command,
}

impl Args {
    /// Executes logic based on the command that the user entered.
    pub fn execute(&self, program_name: &str) -> Result<(), Error> {
        let mut index = Index::load(program_name)?;

        self.cmd.execute(&mut index)?;

        index.store(program_name)?;

        Ok(())
    }
}

//TODO: Add tests
