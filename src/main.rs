use clap::Parser;
use editor::args::Action;
use editor::args::Args;
use editor::config::Config;
use editor::utils::{NC, RED};

fn main() {
    let args = Args::parse();
    if let Err(e) = args.validate() {
        eprintln!("{RED}ERROR{NC}: {e}");
    } else {
        match Config::from_file(&args.config, &args) {
            Ok(config) => {
                let res = match args.action {
                    Action::Install => config.install(&args),
                    Action::Remove => config.remove(&args),
                    Action::Update => todo!(),
                };

                if let Err(e) = res {
                    eprintln!("{RED}ERROR{NC}: {e}");
                }
            }
            Err(e) => eprintln!("{RED}ERROR{NC}: {e}"),
        }
    }
}
