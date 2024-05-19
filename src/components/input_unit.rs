use dioxus::prelude::*;

fn parse_input(s: &str) -> Option<f32> {
    if !s.is_empty() {
        if let Ok(val) = s.parse() {
            return Some(val);
        }
    }
    None
}

#[component]
pub fn InputUnitComponent(
    value: Option<f32>,
    unit: String,
    required: bool,
    onsubmit: EventHandler<Option<f32>>,
    prepend: Option<String>,
) -> Element {
    let value = match value {
        Some(val) => format!("{val}"),
        None => "".to_string(),
    };

    rsx! {
        form {
            class: "input-group input-group-sm mb-1",
            onsubmit: move |evt| {
                onsubmit(parse_input(&evt.data.values()["value"].as_value()));
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
                required,
                step: 0.001,
                name: "value",
                value: value,
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
