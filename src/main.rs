use clap::Parser;
use editor::args::Action;
use editor::args::Args;
use editor::config::Config;
use editor::utils::{NC, RED};

fn main() {
    match Args::parse().validate() {
        Err(e) => {
            eprintln!("{RED}ERROR{NC}: {e}");
        }
        Ok(mut args) => match Config::from_file(&args.config.to_owned(), &mut args) {
            Ok(config) => {
                let res = match args.action {
                    Action::Install => config.install(&args),
                    Action::Remove => config.remove(&args),
                    Action::List => config.list(&args),
                    Action::Update => todo!(),
                };

                if let Err(e) = res {
                    if !e.to_string().is_empty() {
                        eprintln!("{RED}ERROR{NC}: {e}");
                    }
                }
            }
            Err(e) => eprintln!("{RED}ERROR{NC}: {e}"),
        },
    }
}
