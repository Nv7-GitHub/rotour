use std::io::Read;
use std::mem;
use std::slice;
use std::time::Duration;

use serialport::{SerialPort, SerialPortType};

pub fn connect() -> Result<Box<dyn SerialPort + 'static>, Box<serialport::Error>> {
    let res = serialport::available_ports().expect("Failed to fetch Serial ports");
    let mut name: Option<String> = None;
    for port in res {
        if let SerialPortType::UsbPort(info) = port.port_type {
            if let Some(manu) = info.manufacturer {
                if manu == "STMicroelectronics" {
                    name = Some(port.port_name);
                    break;
                }
            }
        }
    }
    if name.is_none() {
        return Err(Box::new(serialport::Error {
            kind: serialport::ErrorKind::NoDevice,
            description: "Tektite-R is not connected.".to_string(),
        }));
    }

    let mut v = serialport::new(name.unwrap(), 9600)
        .timeout(Duration::from_secs(30))
        .flow_control(serialport::FlowControl::Software)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()?;
    v.write_data_terminal_ready(true)?;

    return Ok(v);
}

#[derive(Copy, Clone)]
pub enum CommandType {
    SelfTest,
    Transmit,
    TurnMove,
}

#[repr(C, packed)]
#[derive(Clone)]
pub struct Command {
    pub command_type: u8,
    pub turn: f32, // always turn so that you move forwards
    pub ticks: i32,
    pub tw_off: f32, // tw_off*track_width is added to ticks
}

#[repr(C, packed)]
pub struct ConfigCommand {
    pub kp_turn: f32,
    pub kp_hold: f32,
    pub kp_straight: f32,
    pub kp_velocity: f32,

    pub turn_accel_time: f32,
    pub straight_accel_time: f32,

    pub velocity: f32,
    pub velocity_twoff: f32,
    // Velocity = (velocity+velocity_twoff*track_width)/time
    pub time: f32,
    pub vtime: f32,
}

pub fn self_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut port = connect()?;
    port.clear(serialport::ClearBuffer::Input)?;
    send_command(
        Command {
            command_type: CommandType::SelfTest as u8,
            turn: 0.0,
            ticks: 0,
            tw_off: 0.0,
        },
        &mut port,
    )?;

    Ok(())
}

fn send_command(command: Command, port: &mut Box<dyn SerialPort>) -> Result<(), std::io::Error> {
    let data = unsafe {
        slice::from_raw_parts(
            &command as *const Command as *const u8,
            mem::size_of::<Command>(),
        )
    };
    port.write_all(data)?;
    port.flush()?;

    Ok(())
}

pub fn transmit(cfg: ConfigCommand, moves: Vec<Command>) -> Result<(), Box<dyn std::error::Error>> {
    // Write config
    let mut port = connect()?;
    port.clear(serialport::ClearBuffer::Input)?;
    let data = unsafe {
        slice::from_raw_parts(
            &cfg as *const ConfigCommand as *const u8,
            mem::size_of::<ConfigCommand>(),
        )
    };
    port.write_all(data)?;
    port.flush()?;
    port.read_exact(&mut [0; 1])
        .expect("Failed to read from Serial port"); // Wait for ack

    // Transmit moves
    send_command(
        Command {
            command_type: CommandType::Transmit as u8,
            turn: 0.0,
            ticks: moves.len() as i32,
            tw_off: 0.0,
        },
        &mut port,
    )?;
    for m in moves {
        port.read_exact(&mut [0; 1])
            .expect("Failed to read from Serial port"); // Wait for ack
        send_command(m, &mut port)?;
    }

    Ok(())
}
