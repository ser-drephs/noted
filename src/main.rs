use std::env;

mod noted;

use noted::CommandHandler;

fn main() -> std::io::Result<()> {
    CommandHandler::execute(env::args().collect())
}
