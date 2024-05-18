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

    let mut errors = Vec::new();
    if channel.voltage.set != config.voltage {
        errors.push(format!("{:.3} V is set!", channel.voltage.set));
    }

    if channel.current.set != config.current {
        errors.push(format!("{:.3} A is set!", channel.current.set));
    }

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
                if !errors.is_empty() {
                    div {
                        class: "alert alert-danger mt-1 p-0 mb-0",
                        ul {
                            class: "mt-1 mb-1",
                            for error in errors {
                                li {
                                    {error}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
