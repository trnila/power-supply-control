use std::time::Duration;

use crate::components::channel_delay::ChannelDelayComponent;
use crate::config::AppConfig;
use crate::config::MultiOn;
use crate::mx100qp::auto_vrange;
use crate::mx100qp::read_channels;
use crate::mx100qp::Channel;
use crate::mx100qp::MultiChannelOn;
use crate::mx100qp::Mx100qp;
use dioxus::prelude::*;
use futures::StreamExt;
use log::{error, info};

use crate::components::channel::ChannelComponent;
use crate::components::editable_text::EditableTextComponent;

#[derive(Debug)]
pub enum ChannelSelection {
    AllChannels,
    Channel(u8),
}

#[derive(Debug)]
pub enum PowerSupplyAction {
    On(ChannelSelection),
    Off(ChannelSelection),
    SetVoltage(u8, f32),
    SetCurrent(u8, f32),
    RenameChannel(u8, String),
    SetMultiChannel(u8, MultiChannelOn),
    SetVRange(u8, u8),
    SetAutoVRange(u8, bool),
    SetVoltageTracking(u8),
    SetOvervoltageTrip(u8, Option<f32>),
    SetOvercurrentTrip(u8, Option<f32>),
    Reconfigure,
}

struct PowerSupply {
    name: String,
    channels: Vec<Channel>,
    connected: bool,
}

#[component]
pub fn PowerSupplyComponent(id: String) -> Element {
    let mut appconfig = use_context::<Signal<AppConfig>>();
    let config = appconfig.write().power_supply(&id).clone();
    let mut state = use_signal(|| PowerSupply {
        name: config.name.clone(),
        channels: Vec::new(),
        connected: false,
    });

    let voltage_tracking = config.voltage_tracking;
    let id1 = id.clone();
    let channels = config.channels.clone();

    let voltage_trackings = ["V1 V2 V3 V4", "V1=V2 V3 V4", "V1 V2 V3=V4", "V1=V2 V3=V4"];

    let sync_task = use_coroutine(|mut rx: UnboundedReceiver<PowerSupplyAction>| async move {
        loop {
            state.write().connected = false;
            let port = Mx100qp::open(&config.clone()).await;

            if let Err(err) = port {
                error!("failed to open port: {:?}", err);
                tokio::time::sleep(Duration::from_millis(1000)).await;
                continue;
            }
            let mut port = port.unwrap();
            state.write().connected = true;

            loop {
                if let Ok(Some(msg)) =
                    tokio::time::timeout(Duration::from_millis(100), rx.next()).await
                {
                    info!("{:?}", msg);
                    let res = match msg {
                        PowerSupplyAction::On(channels) => match channels {
                            ChannelSelection::AllChannels => port.all_channel_on().await,
                            ChannelSelection::Channel(ch) => port.channel_on(ch).await,
                        },
                        PowerSupplyAction::Off(channels) => match channels {
                            ChannelSelection::AllChannels => port.all_channel_off().await,
                            ChannelSelection::Channel(ch) => port.channel_off(ch).await,
                        },
                        PowerSupplyAction::SetVoltage(ch, new_voltage) => {
                            let mut conf = appconfig.write();
                            let channel_conf = &mut conf.power_supply_channel(&id, ch);
                            channel_conf.voltage = new_voltage;

                            if channel_conf.auto_vrange {
                                if let Some(vrange) =
                                    auto_vrange(ch, channel_conf.voltage, channel_conf.current)
                                {
                                    channel_conf.vrange = vrange;
                                    port.set_vrange(ch, channel_conf.vrange).await.unwrap();
                                }
                            }

                            conf.save();
                            port.set_voltage(ch, new_voltage).await
                        }
                        PowerSupplyAction::SetCurrent(ch, new_current) => {
                            let mut conf = appconfig.write();
                            let channel_conf = &mut conf.power_supply_channel(&id, ch);
                            channel_conf.current = new_current;

                            if channel_conf.auto_vrange {
                                if let Some(vrange) =
                                    auto_vrange(ch, channel_conf.voltage, channel_conf.current)
                                {
                                    channel_conf.vrange = vrange;
                                    port.set_vrange(ch, channel_conf.vrange).await.unwrap();
                                }
                            }

                            conf.save();
                            port.set_current(ch, new_current).await
                        }
                        PowerSupplyAction::RenameChannel(ch, new_name) => {
                            appconfig
                                .write()
                                .power_supply_channel(&id, ch)
                                .name
                                .clone_from(&new_name);
                            appconfig.write().save();
                            Ok(())
                        }
                        PowerSupplyAction::SetMultiChannel(channel, behaviour) => {
                            match behaviour {
                                MultiChannelOn::Disabled => {
                                    appconfig
                                        .write()
                                        .power_supply_channel(&id, channel)
                                        .multi_on
                                        .enabled = false;
                                }
                                MultiChannelOn::Delay(delay_ms) => {
                                    appconfig
                                        .write()
                                        .power_supply_channel(&id, channel)
                                        .multi_on = MultiOn {
                                        enabled: true,
                                        delay_ms,
                                    };
                                }
                            };
                            appconfig.write().save();

                            port.multichannel_on_setup(channel, behaviour).await
                        }
                        PowerSupplyAction::SetVRange(channel, vrange) => {
                            appconfig.write().power_supply_channel(&id, channel).vrange = vrange;
                            appconfig.write().save();
                            port.set_vrange(channel, vrange).await
                        }
                        PowerSupplyAction::SetAutoVRange(channel, enable) => {
                            appconfig
                                .write()
                                .power_supply_channel(&id, channel)
                                .auto_vrange = enable;
                            appconfig.write().save();
                            Ok(())
                        }
                        PowerSupplyAction::SetVoltageTracking(config) => {
                            appconfig.write().power_supply(&id).voltage_tracking = config;
                            appconfig.write().save();
                            port.set_voltage_tracking(config).await
                        }
                        PowerSupplyAction::SetOvervoltageTrip(channel, voltage) => {
                            appconfig
                                .write()
                                .power_supply_channel(&id, channel)
                                .overvoltage_trip = voltage;
                            appconfig.write().save();
                            port.set_overvoltage_trip(channel, voltage).await
                        }
                        PowerSupplyAction::SetOvercurrentTrip(channel, current) => {
                            appconfig
                                .write()
                                .power_supply_channel(&id, channel)
                                .overcurrent_trip = current;
                            appconfig.write().save();
                            port.set_overcurrent_trip(channel, current).await
                        }
                        PowerSupplyAction::Reconfigure => {
                            port.all_channel_off().await.unwrap();
                            port.set_voltage_tracking(config.voltage_tracking)
                                .await
                                .unwrap();

                            let power_supply = appconfig.write().power_supply(&id).clone();

                            for (ch, channel_config) in power_supply.channels.iter().enumerate() {
                                let ch = ch as u8;
                                port.set_vrange(ch, channel_config.vrange).await.unwrap();
                                port.set_voltage(ch, channel_config.voltage).await.unwrap();
                                port.set_current(ch, channel_config.current).await.unwrap();
                                port.set_overvoltage_trip(ch, channel_config.overvoltage_trip)
                                    .await
                                    .unwrap();
                                port.set_overcurrent_trip(ch, channel_config.overcurrent_trip)
                                    .await
                                    .unwrap();
                                port.multichannel_on_setup(
                                    ch,
                                    match channel_config.multi_on.enabled {
                                        true => {
                                            MultiChannelOn::Delay(channel_config.multi_on.delay_ms)
                                        }
                                        false => MultiChannelOn::Disabled,
                                    },
                                )
                                .await
                                .unwrap();
                            }

                            Ok(())
                        }
                    };

                    if let Err(err) = res {
                        error!("Error: {}", err);
                        break;
                    }
                }
                match read_channels(&mut port.protocol).await {
                    Ok(new) => state.write().channels = new,
                    Err(_) => break,
                };
            }
        }
    });

    rsx! {
        div {
            class: "card mb-1",
            div {
                class: "card-header d-flex gap-3",
                div {
                    class: "flex-grow-1",
                    EditableTextComponent {
                        onsubmit: move |new_name: String| {
                            state.write().name.clone_from(&new_name);
                            appconfig.write().power_supply(&id1).name.clone_from(&new_name);
                            appconfig.write().save();
                        },
                        text: state.read().name.clone(),
                    }
                }
                if state.read().connected {
                    div {
                        class: "d-flex gap-1",

                        span {
                            cursor: "pointer",
                            onclick: move |_| sync_task.send(PowerSupplyAction::Reconfigure),
                            dangerous_inner_html: iconify::svg!("hugeicons:configuration-01"),
                        }

                    select {
                        class: "form-control form-control-sm w-auto",
                        onchange: move |evt| {
                            sync_task.send(PowerSupplyAction::SetVoltageTracking(evt.data.value().parse().unwrap()))
                        },
                        for (i, label) in voltage_trackings.iter().enumerate() {
                            option {
                                value: "{i}",
                                selected: voltage_tracking == i as u8,
                                {label}
                            }
                        }
                    }

                    div {
                        class: "input-group input-group-sm w-auto",
                        span {
                            class: "input-group-text",
                            "Delayed MultiON"
                        }
                        for (channel, channel_conf) in channels.iter().enumerate() {
                            ChannelDelayComponent{
                                channel: channel as u8,
                                enabled: channel_conf.multi_on.enabled,
                                delay_ms: channel_conf.multi_on.delay_ms
                            }
                        }
                        button {
                            class: "btn btn-sm btn-success",
                            onclick: move |_| sync_task.send(PowerSupplyAction::On(ChannelSelection::AllChannels)),
                            "ON"
                        },
                        button {
                            class: "btn btn-sm btn-danger",
                            onclick: move |_| sync_task.send(PowerSupplyAction::Off(ChannelSelection::AllChannels)),
                            "OFF"
                        },
                    }
                }
                }
            }
            if state.read().connected {
                div {
                    class: "card-body d-flex gap-1",
                    for (i, channel) in state.read().channels.iter().enumerate() {
                        ChannelComponent{channel: channel.clone(), config: channels[i].clone()}
                    }
                }
            } else {
                div {
                    class: "text-center",
                    "Device not connected."
                }
            }
        }
    }
}
