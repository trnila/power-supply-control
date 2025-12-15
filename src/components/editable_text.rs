use dioxus::prelude::*;

#[component]
pub fn EditableTextComponent(
    text: String,
    disabled: bool,
    onsubmit: EventHandler<String>,
) -> Element {
    let mut editing = use_signal(|| false);

    rsx! {
        if *editing.read() {
            form {
                onsubmit: move |event| {
                    onsubmit(event.data.values()["value"].as_value());
                    *editing.write() = false;
                },
                class: "input-group input-group-sm",
                input {
                    name: "value",
                    required: true,
                    class: "form-control form-control-sm",
                    autocomplete: "off",
                    value: text,
                }
                button { class: "btn btn-sm btn-success", "OK" }
            }
        } else {
            span {
                class: "text-nowrap",
                ondoubleclick: move |_| *editing.write() = !disabled,
                "{text}"
            }
            if !disabled {
                span {
                    dangerous_inner_html: iconify::svg!("ic:round-drive-file-rename-outline"),
                    class: "ms-1",
                    cursor: "pointer",
                    onclick: move |_| *editing.write() = true,
                }
            }
        }
    }
}
