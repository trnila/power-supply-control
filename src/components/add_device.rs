use dioxus::prelude::*;
use dioxus::{
    core_macro::rsx,
    dioxus_core::Element,
    hooks::{use_context, use_signal},
    signals::Signal,
};
use serialport::UsbPortInfo;

use crate::config::{AppConfig, ChannelConfig, MultiOn, PowerSupplyConfig};

fn format_usb_port(port: &UsbPortInfo) -> String {
    [
        Some(format!("{:04x}:{:04x}", port.vid, port.pid)),
        port.manufacturer.clone(),
        port.product.clone(),
        port.serial_number.clone(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<String>>()
    .join(", ")
}

pub fn AddDeviceComponent() -> Element {
    let mut appconfig = use_context::<Signal<AppConfig>>();

    let scan_usb = move || -> Vec<UsbPortInfo> {
        serialport::available_ports()
            .unwrap()
            .into_iter()
            .filter_map(|p| match p.port_type {
                serialport::SerialPortType::UsbPort(usbinfo) => Some(usbinfo),
                _ => None,
            })
            .filter(|p| {
                for supply in appconfig.read().data.power_supplies.iter() {
                    if supply.vid == p.vid
                        && supply.pid == p.pid
                        && supply.serial_number == p.serial_number
                    {
                        return false;
                    }
                }
                true
            })
            .collect()
    };

    let mut ports = use_signal(scan_usb);

    rsx! {
        form {
            class: "input-group",
            onsubmit: move |evt| {
                let index:usize = evt.data.values()["index"].as_value().parse().unwrap();
                let port = &ports.read()[index];
                appconfig.write().data.power_supplies.push(PowerSupplyConfig{
                    vid: port.vid,
                    pid: port.pid,
                    serial_number: port.serial_number.clone(),
                    id: port.serial_number.clone().unwrap(),
                    name: "Power Supply MX100QP".to_string(),
                    voltage_tracking: 0,
                    channels: (1..=4).map(|ch| {
                        ChannelConfig{
                            name: format!("Channel {ch}"),
                            voltage: 0.0,
                            current: 0.0,
                            vrange: 1,
                            auto_vrange: true,
                            overcurrent_trip: None,
                            overvoltage_trip: None,
                            multi_on: MultiOn {
                                enabled: true,
                                delay_ms: 0,
                            },
                        }
                    }).collect(),
                });
                appconfig.write().save();
            },

            button {
                class: "btn btn-sm btn-secondary",
                prevent_default: "onclick",
                onclick: move |_| *ports.write() = scan_usb(),
                "Rescan USB"
            }
            select {
                class: "form-control form-control-sm",
                name: "index",
                for (i, port) in ports.read().iter().enumerate() {
                    option {
                        value: format!("{i}"),
                        "{format_usb_port(port)}"
                    }
                }
            }
            button {
                class: "btn btn-sm btn-success",
                "Add"
            }
        }
    }
}
