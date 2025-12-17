use std::time::Duration;

use bitflags::bitflags;
use futures::{SinkExt, StreamExt};
use int_enum::IntEnum;
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
    status: [LimitEventStatus; 4],
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct LimitEventStatus: u8 {
        const VOLTAGE_LIMIT = 1;
        const CURRENT_LIMIT = 1 << 1;
        const OVER_VOLTAGE_TRIP = 1 << 2;
        const OVER_CURRENT_TRIP = 1 << 3;
        const TEMPERATURE_TRIP = 1 << 4;
        const FAULT_TRIP = 1 << 6;
    }
}

pub struct VRange {
    pub voltage: f32,
    pub current: f32,
}

#[repr(u8)]
#[derive(IntEnum, PartialEq, Debug, Clone)]
pub enum VoltageTracking {
    NoTracking = 0,
    CH0_1 = 1,
    CH2_3 = 2,
    CH0_1AndCH2_3 = 3,
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
    let mut candidates: Vec<(usize, &VRange)> = VRANGES[ch as usize]
        .iter()
        .enumerate()
        .skip(1)
        .filter_map(|(i, vrange)| {
            if voltage <= vrange.voltage && current <= vrange.current {
                Some((i, vrange))
            } else {
                None
            }
        })
        .collect();
    candidates.sort_by_key(|(_, vrange)| vrange.voltage as u8);
    if candidates.is_empty() {
        None
    } else {
        Some(candidates[0].0 as u8)
    }
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

        Ok(Mx100qp {
            protocol,
            status: [LimitEventStatus::empty(); 4],
        })
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

    pub async fn set_voltage_tracking(
        &mut self,
        config: VoltageTracking,
    ) -> Result<(), std::io::Error> {
        self.protocol
            .send(format!("CONFIG {}", u8::from(config)))
            .await
    }

    pub async fn get_voltage_tracking(&mut self) -> Result<VoltageTracking, std::io::Error> {
        self.protocol.send("CONFIG?".to_string()).await?;
        Ok(self
            .protocol
            .next()
            .await
            .unwrap()?
            .parse::<u8>()
            .unwrap()
            .try_into()
            .unwrap())
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
        self.protocol.send("TRIPRST".to_string()).await?;
        self.status = [LimitEventStatus::empty(); 4];
        Ok(())
    }

    pub async fn read_channels(&mut self) -> Result<Vec<Channel>, std::io::Error> {
        let mut mychannels = Vec::<Channel>::new();

        let voltage_tracking = self.get_voltage_tracking().await?;

        for i in 1..=4u8 {
            // read status
            self.protocol.send(format!("LSR{i}?")).await?;
            let status = LimitEventStatus::from_bits(
                self.protocol.next().await.unwrap()?.parse::<u8>().unwrap(),
            )
            .unwrap();

            self.status[(i - 1) as usize] |= status;
            let status = self.status[(i - 1) as usize];

            self.protocol.send(format!("OP{i}?")).await?;
            let enabled = self.protocol.next().await.unwrap()? == "1";

            self.protocol.send(format!("VRANGE{i}?")).await?;
            let vrange = self.protocol.next().await.unwrap()?.parse().unwrap();

            // XXX: power supply is not responding OVP2? if CONFIG == 3
            let ovp = if voltage_tracking == VoltageTracking::CH0_1AndCH2_3 && i == 2 {
                None
            } else {
                self.protocol.send(format!("OVP{i}?")).await?;
                match self
                    .protocol
                    .next()
                    .await
                    .unwrap()?
                    .split_once(' ')
                    .unwrap()
                    .1
                {
                    "OFF" => None,
                    val => Some(val.parse().unwrap()),
                }
            };

            let ocp = if voltage_tracking == VoltageTracking::CH0_1AndCH2_3 && i == 2 {
                None
            } else {
                self.protocol.send(format!("OCP{i}?")).await?;
                match self
                    .protocol
                    .next()
                    .await
                    .unwrap()?
                    .split_once(' ')
                    .unwrap()
                    .1
                {
                    "OFF" => None,
                    val => Some(val.parse().unwrap()),
                }
            };

            mychannels.push(Channel {
                enabled,
                vrange,
                index: i - 1,
                overvoltage_trip: ovp,
                overcurrent_trip: ocp,
                status,
                voltage: read_unit(&mut self.protocol, i, 'V').await?,
                current: read_unit(&mut self.protocol, i, 'I').await?,
                voltage_tracking: VoltageTrackingState::from_channel_and_config(
                    i - 1,
                    &voltage_tracking,
                ),
            });
        }
        Ok(mychannels)
    }
}

fn find_usb(config: &PowerSupplyConfig) -> Option<String> {
    let ports: Vec<serialport::SerialPortInfo> =
        serialport::available_ports().expect("No ports found!");
    ports.iter().find_map(|p| {
        if let serialport::SerialPortType::UsbPort(usbinfo) = &p.port_type
            && config.pid == usbinfo.pid
            && config.vid == usbinfo.vid
            && config.serial_number == usbinfo.serial_number
        {
            return Some(p.port_name.clone());
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
pub enum VoltageTrackingState {
    None,
    Master,
    Slave,
}

impl VoltageTrackingState {
    fn from_channel_and_config(ch: u8, config: &VoltageTracking) -> VoltageTrackingState {
        match (config, ch) {
            (VoltageTracking::CH0_1, 0) | (VoltageTracking::CH0_1AndCH2_3, 0) => {
                VoltageTrackingState::Master
            }
            (VoltageTracking::CH0_1, 1) | (VoltageTracking::CH0_1AndCH2_3, 1) => {
                VoltageTrackingState::Slave
            }
            (VoltageTracking::CH2_3, 2) | (VoltageTracking::CH0_1AndCH2_3, 2) => {
                VoltageTrackingState::Master
            }
            (VoltageTracking::CH2_3, 3) | (VoltageTracking::CH0_1AndCH2_3, 3) => {
                VoltageTrackingState::Slave
            }
            _ => VoltageTrackingState::None,
        }
    }
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
    pub status: LimitEventStatus,
    pub voltage_tracking: VoltageTrackingState,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_vrange() {
        assert_eq!(auto_vrange(0, 0.0, 0.0), Some(2));
        assert_eq!(auto_vrange(0, 1.0, 2.0), Some(2));
        assert_eq!(auto_vrange(0, 35.0, 0.1), Some(1));
        assert_eq!(auto_vrange(0, 35.0, 3.0), Some(1));
        assert_eq!(auto_vrange(0, 35.0, 5.0), Some(3));
        assert_eq!(auto_vrange(0, 35.0, 6.0), Some(3));
        assert_eq!(auto_vrange(0, 70.0, 0.1), None);
    }
}
