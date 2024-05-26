use std::time::Duration;

use futures::{SinkExt, StreamExt};
use log::{debug, error};
use tokio::time::error::Elapsed;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Framed};

use crate::{config::PowerSupplyConfig, line_codec::LineCodec};

#[derive(Debug)]
pub struct NotMatchingId {
    pub configured: String,
    pub device: String,
}

#[derive(Debug)]
pub enum OpenError {
    NoDeviceFound,
    OpenError(tokio_serial::Error),
    IOError(std::io::Error),
    IdNotMatch(NotMatchingId),
    ProtocolError,
    Timeout,
}

impl From<tokio_serial::Error> for OpenError {
    fn from(err: tokio_serial::Error) -> Self {
        OpenError::OpenError(err)
    }
}

impl From<std::io::Error> for OpenError {
    fn from(err: std::io::Error) -> Self {
        OpenError::IOError(err)
    }
}

impl From<Elapsed> for OpenError {
    fn from(_: Elapsed) -> Self {
        OpenError::Timeout
    }
}

#[derive(Debug)]
pub enum MultiChannelOn {
    Disabled,
    Delay(u32),
}

pub struct Mx100qp {
    pub protocol: Framed<SerialStream, LineCodec>,
}

pub struct VRange {
    pub voltage: f32,
    pub current: f32,
}

impl std::fmt::Display for VRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}V/{}A", self.voltage, self.current)
    }
}

const RANGE_A: [VRange; 4] = [
    VRange {
        voltage: 0.0,
        current: 0.0,
    },
    VRange {
        voltage: 35.0,
        current: 3.0,
    },
    VRange {
        voltage: 16.0,
        current: 6.0,
    },
    VRange {
        voltage: 35.0,
        current: 6.0,
    },
];

const RANGE_B: [VRange; 4] = [
    VRange {
        voltage: 0.0,
        current: 0.0,
    },
    VRange {
        voltage: 35.0,
        current: 3.0,
    },
    VRange {
        voltage: 70.0,
        current: 1.5,
    },
    VRange {
        voltage: 70.0,
        current: 3.0,
    },
];

pub const VRANGES: [[VRange; 4]; 4] = [RANGE_A, RANGE_A, RANGE_B, RANGE_B];

pub fn auto_vrange(ch: u8, voltage: f32, current: f32) -> Option<u8> {
    for (index, vrange) in VRANGES[ch as usize].iter().enumerate() {
        if voltage < vrange.voltage && current < vrange.current {
            return Some(index as u8);
        }
    }
    None
}

impl Mx100qp {
    pub async fn open(config: &PowerSupplyConfig) -> Result<Self, OpenError> {
        let port_path = find_usb(config).ok_or(OpenError::NoDeviceFound)?;
        let port = tokio_serial::new(port_path, 9600).open_native_async()?;
        let mut protocol = LineCodec.framed(port);

        let line_result =
            tokio::time::timeout(Duration::from_millis(5000), wait_first_line(&mut protocol))
                .await??;
        let device_id = line_result
            .split_terminator(',')
            .nth(2)
            .ok_or(OpenError::ProtocolError)?
            .trim();

        if device_id != config.id {
            error!("Id does not match! {} != {}", device_id, config.id);
            tokio::time::sleep(Duration::from_millis(1000)).await;
            return Err(OpenError::IdNotMatch(NotMatchingId {
                configured: config.id.clone(),
                device: device_id.to_string(),
            }));
        }

        Ok(Mx100qp { protocol })
    }

    pub async fn set_voltage(&mut self, ch: u8, new_voltage: f32) -> Result<(), std::io::Error> {
        self.protocol
            .send(format!("V{} {new_voltage}", ch + 1))
            .await
    }

    pub async fn set_current(&mut self, ch: u8, new_current: f32) -> Result<(), std::io::Error> {
        self.protocol
            .send(format!("I{} {new_current}", ch + 1))
            .await
    }

    pub async fn channel_on(&mut self, ch: u8) -> Result<(), std::io::Error> {
        self.protocol.send(format!("OP{} 1", ch + 1)).await
    }

    pub async fn channel_off(&mut self, ch: u8) -> Result<(), std::io::Error> {
        self.protocol.send(format!("OP{} 0", ch + 1)).await
    }

    pub async fn all_channel_on(&mut self) -> Result<(), std::io::Error> {
        self.protocol.send("OPALL 1".to_string()).await
    }

    pub async fn all_channel_off(&mut self) -> Result<(), std::io::Error> {
        self.protocol.send("OPALL 0".to_string()).await
    }

    pub async fn multichannel_on_setup(
        &mut self,
        ch: u8,
        behaviour: MultiChannelOn,
    ) -> Result<(), std::io::Error> {
        let action = match behaviour {
            MultiChannelOn::Disabled => "NEVER",
            MultiChannelOn::Delay(0) => "QUICK",
            MultiChannelOn::Delay(_) => "DELAY",
        };

        self.protocol
            .send(format!("ONACTION{} {action}", ch + 1))
            .await?;

        if let MultiChannelOn::Delay(delay) = behaviour {
            self.protocol
                .send(format!("ONDELAY{} {delay}", ch + 1))
                .await?;
        }

        Ok(())
    }

    pub async fn set_vrange(&mut self, ch: u8, vrange: u8) -> Result<(), std::io::Error> {
        self.protocol
            .send(format!("VRANGE{} {vrange}", ch + 1))
            .await
    }

    pub async fn set_voltage_tracking(&mut self, config: u8) -> Result<(), std::io::Error> {
        self.protocol.send(format!("CONFIG {config}")).await
    }

    pub async fn get_voltage_tracking(&mut self) -> Result<u8, std::io::Error> {
        self.protocol.send("CONFIG?".to_string()).await?;
        Ok(self.protocol.next().await.unwrap()?.parse().unwrap())
    }

    pub async fn set_overvoltage_trip(
        &mut self,
        ch: u8,
        voltage: Option<f32>,
    ) -> Result<(), std::io::Error> {
        self.set_trip(ch, voltage, 'V').await
    }

    pub async fn set_overcurrent_trip(
        &mut self,
        ch: u8,
        current: Option<f32>,
    ) -> Result<(), std::io::Error> {
        self.set_trip(ch, current, 'C').await
    }

    pub async fn set_trip(
        &mut self,
        ch: u8,
        threshold: Option<f32>,
        unit: char,
    ) -> Result<(), std::io::Error> {
        if let Some(threshold) = threshold {
            self.protocol
                .send(format!("O{unit}P{} {threshold}", ch + 1))
                .await?;
        }

        let action = match threshold {
            None => "OFF",
            Some(_) => "ON",
        };
        self.protocol
            .send(format!("O{unit}P{} {action}", ch + 1))
            .await
    }

    pub async fn trip_reset(&mut self) -> Result<(), std::io::Error> {
        self.protocol.send("TRIPRST".to_string()).await
    }
}

fn find_usb(config: &PowerSupplyConfig) -> Option<String> {
    let ports: Vec<serialport::SerialPortInfo> =
        serialport::available_ports().expect("No ports found!");
    ports.iter().find_map(|p| {
        if let serialport::SerialPortType::UsbPort(usbinfo) = &p.port_type {
            if config.pid == usbinfo.pid
                && config.vid == usbinfo.vid
                && config.serial_number == usbinfo.serial_number
            {
                return Some(p.port_name.clone());
            }
        }
        None
    })
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Unit {
    pub current: f32,
    pub set: f32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Channel {
    pub index: u8,
    pub vrange: u8,
    pub enabled: bool,
    pub current: Unit,
    pub voltage: Unit,
    pub overvoltage_trip: Option<f32>,
    pub overcurrent_trip: Option<f32>,
}

pub async fn read_channels(
    reader: &mut Framed<SerialStream, LineCodec>,
) -> Result<Vec<Channel>, std::io::Error> {
    let mut mychannels = Vec::<Channel>::new();
    for i in 1..=4 {
        reader.send(format!("OP{i}?")).await?;
        let enabled = reader.next().await.unwrap()? == "1";

        reader.send(format!("VRANGE{i}?")).await?;
        let vrange = reader.next().await.unwrap()?.parse().unwrap();

        reader.send(format!("OVP{}?", i)).await?;
        let ovp = match reader.next().await.unwrap()?.split_once(' ').unwrap().1 {
            "OFF" => None,
            val => Some(val.parse().unwrap()),
        };

        reader.send(format!("OCP{}?", i)).await?;
        let ocp = match reader.next().await.unwrap()?.split_once(' ').unwrap().1 {
            "OFF" => None,
            val => Some(val.parse().unwrap()),
        };

        mychannels.push(Channel {
            enabled,
            vrange,
            index: i - 1,
            overvoltage_trip: ovp,
            overcurrent_trip: ocp,
            voltage: read_unit(reader, i, 'V').await?,
            current: read_unit(reader, i, 'I').await?,
        });
    }
    Ok(mychannels)
}

async fn read_unit(
    reader: &mut Framed<SerialStream, LineCodec>,
    channel: u8,
    unit: char,
) -> Result<Unit, std::io::Error> {
    reader.send(format!("{unit}{channel}O?")).await?;
    let mut current = reader.next().await.unwrap()?;
    current.truncate(current.len() - 1);

    reader.send(format!("{unit}{channel}?")).await?;
    let set = reader.next().await.unwrap()?;
    let mut parts = set.split_terminator(' ');
    assert!((parts.next()).unwrap() == format!("{unit}{channel}"));
    let set = parts.next().unwrap();

    Ok(Unit {
        current: current.parse().unwrap(),
        set: set.parse().unwrap(),
    })
}

async fn wait_first_line(
    protocol: &mut Framed<SerialStream, LineCodec>,
) -> Result<std::string::String, std::io::Error> {
    loop {
        protocol.send("*IDN?".to_string()).await?;
        match tokio::time::timeout(Duration::from_millis(100), protocol.next()).await {
            Ok(v) => match v {
                Some(line) => return line,
                None => continue,
            },
            Err(err) => debug!("{err}"),
        };
    }
}
