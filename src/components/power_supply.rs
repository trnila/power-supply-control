use std::time::Duration;

use crate::config::AppConfig;
use crate::mx100qp::read_channels;
use crate::mx100qp::Channel;
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

    let id1 = id.clone();
    let channels = config.channels.clone();

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
                            appconfig.write().power_supply_channel(&id, ch).voltage = new_voltage;
                            appconfig.write().save();
                            port.set_voltage(ch, new_voltage).await
                        }
                        PowerSupplyAction::SetCurrent(ch, new_current) => {
                            appconfig.write().power_supply_channel(&id, ch).current = new_current;
                            appconfig.write().save();
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
                        class: "btn-group",
                        button {
                            disabled: true,
                            opacity: "1.0",
                            class: "btn btn-sm",
                            "Multi"
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
