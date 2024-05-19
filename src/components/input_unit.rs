use dioxus::prelude::*;

#[component]
pub fn InputUnitComponent(
    value: f32,
    unit: String,
    onsubmit: EventHandler<f32>,
    prepend: Option<String>,
) -> Element {
    rsx! {
        form {
            class: "input-group input-group-sm mb-1",
            onsubmit: move |evt| {
                onsubmit(evt.data.values()["value"].as_value().parse().unwrap());
            },
            if let Some(prepend) = prepend {
                span {
                    class: "input-group-text",
                    "{prepend}"
                }
            }
            input {
                class: "form-control form-control-sm text-end",
                r#type: "number",
                step: 0.001,
                name: "value",
                value: "{value}",
            }
            span {
                class: "input-group-text",
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
