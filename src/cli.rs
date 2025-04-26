use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub username: String,
    #[arg(short, long)]
    pub password: String, // TODO we should really prefer an environment variable or GUI.
    #[arg(short, long)]
    pub domain: Option<String>,
    #[arg(short = 'P', long, default_value_t = 3389)]
    pub port: u16,
    pub host: String,
}
