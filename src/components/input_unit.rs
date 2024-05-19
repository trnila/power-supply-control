use dioxus::prelude::*;

#[component]
pub fn InputUnitComponent(value: f32, unit: String, onsubmit: EventHandler<f32>) -> Element {
    rsx! {
        form {
            class: "input-group input-group-sm mb-1",
            onsubmit: move |evt| {
                onsubmit(evt.data.values()["value"].as_value().parse().unwrap());
            },
            input {
                class: "form-control form-control-sm",
                r#type: "number",
                step: 0.001,
                name: "value",
                value: "{value}",
            }
            span {
                class: "input-group-text text-center",
                width: "30px",
                "{unit}"
            }
            button {
                class: "btn btn-sm btn-outline-secondary",
                "Set"
            }
        }
    }
}
