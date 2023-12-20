use clap::Parser;
use editor::args::Action;
use editor::args::Args;
use editor::config::Config;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::from_file(&args.config)?;

    println!("{:#?}", args);
    println!("{:#?}", config);

    match args.action {
        Action::Install => config.install(&args),
        Action::Remove => config.remove(&args),
        Action::Update => todo!(),
    }
}
