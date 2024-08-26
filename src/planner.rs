use super::{Command, CommandType, Config, ConfigCommand};
use std::{f32::consts::PI, io::BufRead, path::PathBuf};
const EPSILON: f32 = 1e-4;
const CM_PER_SQUARE: f32 = 50.0;

pub struct PlanningResult {
    pub commands: Vec<Command>,
    pub config: ConfigCommand,
}

#[derive(Debug)]
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

fn mod_floats(a: f32, b: f32) -> f32 {
    a - (a / (b + 2.0 * EPSILON)).round() * b
}

fn plan_token(tok: &Token, angle: &mut f32, config: &Config) -> Command {
    let mut dist = match *tok {
        Token::Up(dy) | Token::Down(dy) => dy,
        Token::Left(dx) | Token::Right(dx) => dx,
    };

    let target_ang = tok.target_angle();
    let mut dang = target_ang - *angle;
    if dang > PI + EPSILON {
        dang -= 2.0 * PI;
    } else if dang < -PI - EPSILON {
        dang += 2.0 * PI;
    }
    dang = mod_floats(dang, PI); // Go backwards instead of doing 180deg turn
    if dang.abs() < EPSILON {
        dang = 0.0;
    }

    *angle += dang;

    // Backwards driving
    if ((*angle - target_ang).abs() - PI).abs() < EPSILON {
        dist = -dist;
    }

    Command {
        command_type: CommandType::TurnMove as u8,
        turn: dang,
        ticks: (dist * config.ticks_per_cm as f32 * CM_PER_SQUARE) as i32,
        tw_off: 0.0,
    }
}

pub fn plan(path: PathBuf, config: Config) -> Result<PlanningResult, Box<dyn std::error::Error>> {
    let mut time: f32 = 0.0;

    // Parse
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut tokens = Vec::new();
    for line in reader.lines() {
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
    if time == 0.0 {
        return Err("No time specified.".into());
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
    for tok in tokens.iter() {
        commands.push(plan_token(tok, &mut angle, &config));
    }

    // Fix ending angle
    let ediff = tokens.last().unwrap().target_angle() - angle;
    if ediff.abs() > EPSILON {
        // Backtrack to last turn
        for (i, cmd) in commands.iter().enumerate().rev() {
            if cmd.turn.abs() > EPSILON {
                commands[i].turn += ediff;
                if commands[i].turn > PI {
                    commands[i].turn -= 2.0 * PI;
                } else if commands[i].turn < -PI {
                    commands[i].turn += 2.0 * PI;
                }
                angle = tokens.last().unwrap().target_angle();

                // Re-calculate all commands after
                commands = commands[..=i].to_vec();

                for (j, tok) in tokens.iter().enumerate().skip(i) {
                    let res = plan_token(tok, &mut angle, &config);
                    if j == i {
                        commands[i].ticks = res.ticks;
                        commands[i].tw_off = res.tw_off;
                    } else {
                        commands.push(res);
                    }
                }
                break;
            }
        }
    }

    // Calculate tw_off, velocity
    let mut velocity = 0.0;
    let mut velocity_twoff = 0.0;
    let mut vtime = time;
    for i in 0..commands.len() - 1 {
        // Calculate velocity
        velocity += commands[i].ticks.abs() as f32;
        velocity_twoff += commands[i].tw_off.abs();
        vtime -= config.straight_accel_time * 2.0;
        if commands[i + 1].turn.abs() > EPSILON {
            velocity_twoff += commands[i + 1].turn.abs();
            vtime -= 2.0 * config.turn_accel_time;
        }
        if i == commands.len() - 2 {
            velocity += commands[i + 1].ticks.abs() as f32;
            velocity_twoff += commands[i + 1].tw_off.abs();
            vtime -= config.straight_accel_time * 2.0;
        }

        // Calculate twoff
        if commands[i + 1].turn.abs() < EPSILON {
            continue;
        }
        commands[i].tw_off -= 0.5;
        commands[i + 1].tw_off -= 0.5;
    }
    if vtime < 0.0 {
        return Err("Not enough time. Try increasing the time or decreasing the distance.".into());
    }

    // Print resulting path
    for cmd in commands.iter() {
        if (cmd.turn).abs() > EPSILON {
            println!("Turn: {} degrees", cmd.turn.to_degrees());
        }
        let two = cmd.tw_off;
        let v = cmd.ticks;
        println!(
            "Move: {} ticks {} {} track width",
            v,
            if two < 0.0 { "-" } else { "+" },
            two.abs()
        );
    }
    println!("Approximate Velocity: {} ticks/second", velocity / time);

    Ok(PlanningResult {
        commands,
        config: ConfigCommand {
            kp_turn: config.kp_turn,
            kp_hold: config.kp_hold,
            kp_straight: config.kp_straight,
            kp_velocity: config.kp_velocity,
            turn_accel_time: config.turn_accel_time,
            straight_accel_time: config.straight_accel_time,
            friction: config.friction,
            dowel_off: config.dowel_off,

            velocity, // TODO: Calculate velocity in ticks per cm
            velocity_twoff,
            time,
            vtime,
        },
    })
}
