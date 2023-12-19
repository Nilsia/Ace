use clap::Parser;
use editor::{args::EditorArg, config::Config};

fn main() -> anyhow::Result<()> {
    let args = EditorArg::parse();
    let config: Config = Config::from_file(args.config)?;
    println!("{:#?}", config);
    Ok(())
}
