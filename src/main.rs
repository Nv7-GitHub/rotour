use std::path::PathBuf;

use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand};
use config::config_command;
pub use config::Config;

mod connection;
pub use config::read_config;
use connection::transmit;
pub use connection::{Command, CommandType, ConfigCommand};
mod planner;

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

        #[arg(long)]
        friction: Option<f32>,

        #[arg(long)]
        dowel_off: Option<f32>,
    },
    #[command(about = "Run a self-test on the robot.")]
    SelfTest,
    Run {
        file: PathBuf,
    },
}

fn run_path(path: PathBuf, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let res = planner::plan(path, config)?;
    transmit(res.config, res.commands)?;
    Ok(())
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
            friction,
            dowel_off,
        } => config_command(
            ticks_per_cm,
            kp_turn,
            kp_hold,
            kp_straight,
            kp_velocity,
            turn_accel_time,
            straight_accel_time,
            friction,
            dowel_off,
        ),
        Commands::SelfTest => connection::self_test(),
        Commands::Run { file } => run_path(file, config),
    } {
        Cli::command().error(ErrorKind::Io, v.to_string()).exit();
    }
}
