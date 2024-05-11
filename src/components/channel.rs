use crate::{
    components::{
        editable_text::EditableTextComponent,
        power_supply::{ChannelSelection, PowerSupplyAction},
    },
    config::ChannelConfig,
    mx100qp::Channel,
};
use dioxus::prelude::*;

#[component]
pub fn ChannelComponent(channel: Channel, config: ChannelConfig) -> Element {
    let power_supply_action = use_coroutine_handle::<PowerSupplyAction>();
    let card_class = if channel.enabled { "success" } else { "danger" };

    rsx! {
        div {
            class: "card flex-fill",
            div {
                class: "card-header d-flex gap-1 text-bg-{card_class}",
                div {
                    class: "flex-grow-1",
                    EditableTextComponent {
                        onsubmit: move |new_name: String| {
                            power_supply_action.send(PowerSupplyAction::RenameChannel(channel.index, new_name));
                        },
                        text: config.name,
                    }
                }

                div {
                    class: "btn-group",
                    button {
                        class: "btn btn-sm btn-success",
                        onclick: move |_| power_supply_action.send(PowerSupplyAction::On(ChannelSelection::Channel(channel.index))),
                        "ON"
                    }
                    button {
                        class: "btn btn-sm btn-danger",
                        onclick: move |_| power_supply_action.send(PowerSupplyAction::Off(ChannelSelection::Channel(channel.index))),
                        "OFF"
                    }
                }
            }
            div {
                class: "card-body",
                if channel.current.set != config.current {
                    div {
                        class: "text-danger",
                        "Configured current limit {config.current:.3} A differs to device {channel.current.set:.3} A"
                    }
                }

                if channel.voltage.set != config.voltage {
                    div {
                        class: "text-danger",
                        "Configured Voltage limit {config.voltage:.3} V differs to device {channel.voltage.set:.3} V"
                    }
                }

                div {class: "text-end", "{channel.voltage.current:.3} V"}
                div {class: "text-end", "{channel.current.current:.3} A"}
                form {
                    class: "input-group input-group-sm mb-1",
                    onsubmit: move |event| {
                        log::info!("Submitted! {event:?}");
                        power_supply_action.send(
                            PowerSupplyAction::SetVoltage(channel.index, event.data.values()["value"].as_value().parse().unwrap())
                        );
                    },
                    input {
                        class: "form-control form-control-sm",
                        r#type: "number",
                        step: 0.001,
                        name: "value",
                        value: "{config.voltage}",
                    }
                    span {
                        class: "input-group-text text-center",
                        width: "30px",
                        "V"
                    }
                    button {
                        class: "btn btn-sm btn-outline-secondary",
                        "Set"
                    }
                }
                form {
                    class: "input-group input-group-sm",
                    onsubmit: move |event| {
                        power_supply_action.send(
                            PowerSupplyAction::SetCurrent(channel.index, event.data.values()["value"].as_value().parse().unwrap())
                        );
                    },
                    input {
                        class: "form-control form-control-sm",
                        r#type: "number",
                        step: 0.001,
                        name: "value",
                        value: "{config.current}",
                    }
                    span {
                        class: "input-group-text  text-center",
                        width: "30px",
                        "A"
                    }
                    button {
                        class: "btn btn-sm btn-outline-secondary",
                        "Set"
                    }
                }
            }
        }
    }
}
