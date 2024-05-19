use std::time::Duration;

use futures::{SinkExt, StreamExt};
use log::error;
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

pub struct Mx100qp {
    pub protocol: Framed<SerialStream, LineCodec>,
}

impl Mx100qp {
    pub async fn open(config: &PowerSupplyConfig) -> Result<Self, OpenError> {
        let port_path = find_usb(config).ok_or(OpenError::NoDeviceFound)?;
        let port = tokio_serial::new(port_path, 9600).open_native_async()?;
        let mut protocol = LineCodec.framed(port);

        // TODO: remove because of arduino
        tokio::time::sleep(Duration::from_millis(2000)).await;
        protocol.send("*IDN?".to_string()).await?;
        let line_result = wait_first_line(&mut protocol).await?;
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
    pub enabled: bool,
    pub current: Unit,
    pub voltage: Unit,
}

pub async fn read_channels(
    reader: &mut Framed<SerialStream, LineCodec>,
) -> Result<Vec<Channel>, std::io::Error> {
    let mut mychannels = Vec::<Channel>::new();
    for i in 1..=4 {
        reader.send(format!("OP{i}?")).await?;
        let enabled = reader.next().await.unwrap()? == "1";

        mychannels.push(Channel {
            enabled,
            index: i - 1,
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
        match protocol.next().await {
            Some(line) => return line,
            None => continue,
        };
    }
}
