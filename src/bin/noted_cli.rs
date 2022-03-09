use noted::cli::Cli;
use std::{env::{self, ArgsOs}, io};

#[cfg(not(tarpaulin_include))]
fn main() -> Result<(), io::Error> {
    invoke(env::args_os())
}

fn invoke(args: ArgsOs) -> Result<(), io::Error>
{
    let cli = Cli::parse(args).unwrap_or_else(|e| e.exit());
    simple_logger::SimpleLogger::new()
        .with_level(cli.verbosity)
        .init()
        .unwrap();

    log::trace!("Trace information active");
    log::debug!("Debug information active");
    log::info!("Executing command: {}", cli.command);
    match cli.command.invoke(None) {
        Ok(file) => {
            if let Some(filepath) = file {
                log::debug!("Open file: {:?}", filepath);
                open::that(filepath)
            } else {
                Ok(())
            }
        }
        Err(err) => Err(err),
    }
}
