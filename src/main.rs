use clap::Parser;
use editor::args::Action;
use editor::args::Args;
use editor::config::Config;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut config = Config::from_file(&args.config)?;

    println!("{:#?}", args);
    println!("{:#?}", config);

    match args.action {
        Action::Install => todo!(),
        Action::Remove => todo!(),
        Action::Update => todo!(),
    }
}
