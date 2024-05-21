use crate::{
    components::{
        editable_text::EditableTextComponent,
        input_unit::InputUnitComponent,
        power_supply::{ChannelSelection, PowerSupplyAction},
    },
    config::ChannelConfig,
    mx100qp::{Channel, VRANGES},
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

    if channel.overvoltage_trip != config.overvoltage_trip {
        errors.push(match channel.overvoltage_trip {
            None => "Overvoltage is disabled".to_string(),
            Some(set) => format!("Overvoltage trip is set to {set:.3} V"),
        });
    }

    if channel.overcurrent_trip != config.overcurrent_trip {
        errors.push(match channel.overcurrent_trip {
            None => "Overcurrent is disabled".to_string(),
            Some(set) => format!("Overcurrent trip is set to {set:.3} A"),
        });
    }

    if channel.vrange != config.vrange {
        errors.push("Different VRange is set!".to_string());
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
                if channel.enabled {
                    div {class: "text-end", "{channel.voltage.current:.3} V"}
                    div {class: "text-end", "{channel.current.current:.3} A"}
                } else {
                    div {class: "text-end text-muted", "{channel.voltage.set:.3} V"}
                    div {class: "text-end text-muted", "{channel.current.set:.3} A"}
                }

                InputUnitComponent{
                    value: Some(config.voltage),
                    unit: "V",
                    required: true,
                    onsubmit: move |new_voltage| {
                        if let Some(new_voltage) = new_voltage {
                            power_supply_action.send(
                                PowerSupplyAction::SetVoltage(channel.index, new_voltage)
                            );
                        }
                    },
                }
                InputUnitComponent{
                    value: Some(config.current),
                    unit: "A",
                    required: true,
                    onsubmit: move |new_current| {
                        if let Some(new_current) = new_current {
                            power_supply_action.send(
                                PowerSupplyAction::SetCurrent(channel.index, new_current)
                            );
                        }
                    },
                }
                InputUnitComponent{
                    value: config.overvoltage_trip,
                    unit: "V",
                    prepend: "Overvoltage trip",
                    required: false,
                    onsubmit: move |new_voltage| {
                        power_supply_action.send(
                            PowerSupplyAction::SetOvervoltageTrip(channel.index, new_voltage)
                        );
                    },
                }

                InputUnitComponent{
                    value: config.overcurrent_trip,
                    unit: "A",
                    prepend: "Overcurrent trip",
                    required: false,
                    onsubmit: move |new_current| {
                        power_supply_action.send(
                            PowerSupplyAction::SetOvercurrentTrip(channel.index, new_current)
                        );
                    },
                }

                div {
                    class: "input-group input-group-sm",
                    span {
                        class: "input-group-text form-switch",
                        "Auto VRANGE"

                        input {
                            r#type: "checkbox",
                            class: "form-check-input ms-1",
                            checked: config.auto_vrange,
                            onchange: move |evt| {
                                power_supply_action.send(PowerSupplyAction::SetAutoVRange(channel.index, evt.data.value().parse::<bool>().unwrap()));
                            }
                        }
                    }

                    select {
                        class: "form-control form-control-sm",
                        disabled: config.auto_vrange,
                        onchange: move |evt| {
                            power_supply_action.send(PowerSupplyAction::SetVRange(channel.index, evt.data.value().parse().unwrap()));
                        },
                        for (i, range) in VRANGES[channel.index as usize].iter().enumerate() {
                            option {
                                selected: config.vrange as usize == i,
                                value: "{i}",
                                "{range}"
                            }
                        }
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
