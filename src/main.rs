use clap::Parser;
use editor::args::Action;
use editor::args::Args;
use editor::config::Config;
use editor::utils::{NC, RED};

fn main() {
    let args = Args::parse();
    match Config::from_file(&args.config, &args) {
        Ok(config) => {
            // println!("{args:#?}");
            // println!("{config:#?}");
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
