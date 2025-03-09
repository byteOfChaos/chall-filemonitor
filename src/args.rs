use clap::Parser;

pub const DEFAULT_PATH: &str = "~/inbox";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, default_value_t = String::from(DEFAULT_PATH))]
    pub directory: String,
}
