use clap::Parser;

#[derive(Parser)]
#[command(name = "openduckrust", about = "openduckrust CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Check API health
    Health,
    /// Login to the platform
    Login { #[arg(short, long)] email: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Health => { println!("TODO: ping API health endpoint"); }
        Commands::Login { email } => { println!("TODO: authenticate {email}"); }
    }
}
