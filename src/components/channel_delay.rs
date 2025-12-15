use dioxus::prelude::*;

use crate::{components::power_supply::PowerSupplyAction, mx100qp::MultiChannelOn};

#[component]
pub fn ChannelDelayComponent(channel: u8, enabled: bool, delay_ms: u32) -> Element {
    let power_supply_action = use_coroutine_handle::<PowerSupplyAction>();
    let mut delay_ms = use_signal(|| delay_ms);

    rsx! {
        span { class: "input-group-text", class: if !enabled { "disabled" },
            strong { "CH{channel+1}" }
            input {
                class: "form-check-input ms-1",
                r#type: "checkbox",
                checked: enabled,
                autocomplete: "off",
                onchange: move |evt| {
                    power_supply_action
                        .send(
                            PowerSupplyAction::SetMultiChannel(
                                channel,
                                if evt.data.value().parse().unwrap() {
                                    MultiChannelOn::Delay(*delay_ms.read())
                                } else {
                                    MultiChannelOn::Disabled
                                },
                            ),
                        );
                },
            }
        }
        input {
            r#type: "number",
            class: "form-control no-number-arrows text-end border-end-0 pe-0",
            style: "width: 50px",
            value: "{delay_ms}",
            disabled: !enabled,
            min: 0,
            autocomplete: "off",
            oninput: move |evt| {
                delay_ms.set(evt.value().parse().unwrap_or(0));
            },
            onchange: move |evt| {
                delay_ms.set(evt.value().parse().unwrap_or(0));
                power_supply_action
                    .send(
                        PowerSupplyAction::SetMultiChannel(
                            channel,
                            MultiChannelOn::Delay(*delay_ms.read()),
                        ),
                    );
            },
        }
        span { class: "input-group-text ps-1", class: if !enabled { "disabled" }, "ms" }
    }
}
