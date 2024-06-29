use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand};
use config::config_command;

mod connection;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Configure Tektite-R! Does not require a connection to the robot.")]
    Config {
        #[arg(long)]
        ticks_per_cm: Option<u32>,

        #[arg(long)]
        kp_turn: Option<f32>,

        #[arg(long)]
        kp_hold: Option<f32>,

        #[arg(long)]
        kp_straight: Option<f32>,

        #[arg(long)]
        kp_velocity: Option<f32>,

        #[arg(long)]
        turn_accel_time: Option<f32>,

        #[arg(long)]
        straight_accel_time: Option<f32>,
    },
    #[command(about = "Run a self-test on the robot.")]
    SelfTest,
}

mod config;
fn main() {
    let args = Cli::parse();
    let res = config::read_config();
    if let Err(v) = res {
        Cli::command().error(ErrorKind::Io, v.to_string()).exit();
    }
    let config = res.unwrap();

    if let Err(v) = match args.command {
        Commands::Config {
            ticks_per_cm,
            kp_turn,
            kp_hold,
            kp_straight,
            kp_velocity,
            turn_accel_time,
            straight_accel_time,
        } => config_command(
            ticks_per_cm,
            kp_turn,
            kp_hold,
            kp_straight,
            kp_velocity,
            turn_accel_time,
            straight_accel_time,
        ),
        Commands::SelfTest => connection::self_test(),
    } {
        Cli::command().error(ErrorKind::Io, v.to_string()).exit();
    }
}
