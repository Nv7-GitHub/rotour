use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub ticks_per_cm: f32,

    pub kp_move: f32,
    pub kp_hold: f32,
    pub kp_straight: f32,
    pub kp_velocity: f32,
    pub imu_weight: f32,
    pub backlash: i32,

    pub turn_accel_time: f32,
    pub straight_accel_time: f32,
    pub friction: f32,
    pub dowel_off: f32,
    pub reverse: bool,
}

// Read the config file
pub fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir().ok_or("Failed to get config directory")?;
    let config_path = config_dir.join("rotour/config.toml");

    if !config_path.exists() {
        // Create a default config if the file doesn't exist
        let default_config = Config {
            ticks_per_cm: 84.6,
            kp_move: 1.5,
            kp_hold: 0.01,
            kp_straight: 2.5,
            kp_velocity: 0.000003,
            turn_accel_time: 0.25,
            straight_accel_time: 0.5,
            friction: 0.12,
            dowel_off: 6.562, // CM
            imu_weight: 0.6,
            backlash: 20,
            reverse: false,
        };

        let config_str = toml::to_string(&default_config)?;
        fs::create_dir_all(config_path.parent().unwrap())?;
        fs::write(&config_path, config_str)?;
    }

    // Read the config file
    let config_str = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_str)?;

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir().ok_or("Failed to get config directory")?;
    let config_path = config_dir.join("rotour/config.toml");

    let config_str = toml::to_string(config)?;
    fs::create_dir_all(config_path.parent().unwrap())?;
    fs::write(&config_path, config_str)?;

    Ok(())
}

pub fn config_command(
    ticks_per_cm: Option<f32>,
    kp_move: Option<f32>,
    kp_hold: Option<f32>,
    kp_straight: Option<f32>,
    kp_velocity: Option<f32>,
    turn_accel_time: Option<f32>,
    straight_accel_time: Option<f32>,
    friction: Option<f32>,
    dowel_off: Option<f32>,
    reverse: Option<bool>,
    imu_weight: Option<f32>,
    backlash: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = read_config()?;
    if let Some(v) = ticks_per_cm {
        config.ticks_per_cm = v;
    }
    if let Some(v) = kp_move {
        config.kp_move = v;
    }
    if let Some(v) = kp_hold {
        config.kp_hold = v;
    }
    if let Some(v) = kp_straight {
        config.kp_straight = v;
    }
    if let Some(v) = kp_velocity {
        config.kp_velocity = v;
    }
    if let Some(v) = turn_accel_time {
        config.turn_accel_time = v;
    }
    if let Some(v) = straight_accel_time {
        config.straight_accel_time = v;
    }
    if let Some(v) = friction {
        config.friction = v;
    }
    if let Some(v) = dowel_off {
        config.dowel_off = v;
    }
    if let Some(v) = reverse {
        config.reverse = v;
    }
    if let Some(v) = imu_weight {
        config.imu_weight = v;
    }
    if let Some(v) = backlash {
        config.backlash = v;
    }

    save_config(&config)?;

    // Print the new config
    println!("ticks_per_cm: {}", config.ticks_per_cm);
    println!("kp_move: {}", config.kp_move);
    println!("kp_hold: {}", config.kp_hold);
    println!("kp_straight: {}", config.kp_straight);
    println!("kp_velocity: {}\n", config.kp_velocity);
    println!("turn_accel_time: {}", config.turn_accel_time);
    println!("straight_accel_time: {}", config.straight_accel_time);
    println!("friction: {}", config.friction);
    println!("dowel_off: {}", config.dowel_off);
    println!("reverse: {}", config.reverse);
    println!("imu_weight: {}", config.imu_weight);
    println!("backlash: {}", config.backlash);

    Ok(())
}
