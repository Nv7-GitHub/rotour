use super::{Command, CommandType, Config, ConfigCommand};
use std::{
    f32::{consts::PI, EPSILON},
    io::BufRead,
    path::PathBuf,
};

pub struct PlanningResult {
    pub commands: Vec<Command>,
    pub config: ConfigCommand,
}

enum Token {
    Up(f32),
    Down(f32),
    Left(f32),
    Right(f32),
}

impl Token {
    fn target_angle(&self) -> f32 {
        match self {
            Token::Up(_) => PI / 2.0,
            Token::Down(_) => -PI / 2.0,
            Token::Left(_) => PI,
            Token::Right(_) => 0.0,
        }
    }
}

pub fn plan(path: PathBuf, config: Config) -> Result<PlanningResult, Box<dyn std::error::Error>> {
    let mut time: f32 = 0.0;

    // Parse
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut tokens = Vec::new();
    for (ind, line) in reader.lines().enumerate() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 || parts[0].starts_with("#") {
            continue;
        }

        let ticks = parts[1].parse::<f32>()?;
        let tok = match parts[0].to_lowercase().as_str() {
            "up" => Ok(Token::Up(ticks)),
            "down" => Ok(Token::Down(ticks)),
            "left" => Ok(Token::Left(ticks)),
            "right" => Ok(Token::Right(ticks)),
            "time" => {
                time = ticks;
                continue;
            }
            _ => Err(""),
        }?;
        tokens.push(tok);
    }

    // Plan
    let mut commands = Vec::new();
    let mut xfin = 0.0;
    let mut yfin = 0.0;

    // Calculate final position
    for tok in tokens.iter() {
        match tok {
            Token::Up(dy) => {
                yfin += dy;
            }
            Token::Down(dy) => {
                yfin -= dy;
            }
            Token::Left(dx) => {
                xfin -= dx;
            }
            Token::Right(dx) => {
                xfin += dx;
            }
        }
    }

    let mut angle = tokens[0].target_angle(); // 0 is pointing east
    let mut x = 0.0;
    let mut y = 0.0;

    for tok in tokens {
        let mut dist;
        match tok {
            Token::Up(dy) => {
                dist = dy;
                y += dy;
            }
            Token::Down(dy) => {
                dist = dy;
                y -= dy;
            }
            Token::Left(dx) => {
                dist = dx;
                x -= dx;
            }
            Token::Right(dx) => {
                dist = dx;
                x += dx;
            }
        }

        let target_ang = tok.target_angle();
        let dang = (target_ang - angle) % PI; // Go backwards instead of doing 180deg turn
        angle += dang;

        // Backwards driving
        if ((angle - target_ang) - PI).abs() < EPSILON {
            dist = -dist;
        }

        if dang.abs() > EPSILON {
            println!("Turn: {}", dang.to_degrees());
            commands.push(Command {
                command_type: CommandType::Turn as u8,
                ticks: dang,
            });
        }

        println!("Drive: {}", dist);
        commands.push(Command {
            command_type: CommandType::Move as u8,
            ticks: dist * config.ticks_per_cm as f32,
        });
    }

    Ok(PlanningResult {
        commands,
        config: ConfigCommand {
            kp_turn: config.kp_turn,
            kp_hold: config.kp_hold,
            kp_straight: config.kp_straight,
            kp_velocity: config.kp_velocity,
            turn_accel_time: config.turn_accel_time,
            straight_accel_time: config.straight_accel_time,
            velocity: 0.0, // TODO: Calculate velocity in ticks per cm
            time: time as f32,
        },
    })
}
