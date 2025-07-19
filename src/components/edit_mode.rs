use dioxus::prelude::*;

use crate::components::modal::ModalComponent;

#[derive(Clone, Copy, PartialEq)]
pub struct EditMode(pub bool);

#[component]
pub fn EditModeComponent() -> Element {
    let mut show = use_signal(|| false);
    let mut edit_mode = use_context::<Signal<EditMode>>();

    rsx! {
        div {
            class: "form-check form-switch",
            input {
                class: "form-check-input",
                style: "cursor: pointer",
                r#type: "checkbox",
                id: "edit-mode-switch",
                checked: edit_mode.read().0,
                onclick: move |evt| {
                    evt.prevent_default();
                    if !edit_mode.read().0 {
                        *show.write() = true;
                    } else {
                        *edit_mode.write() = EditMode(false);
                    }
                },
            }
            label {
                class: "form-check-label",
                style: "cursor: pointer",
                r#for: "edit-mode-switch",
                "Edit mode",
            }
        }

        ModalComponent {
            show: show(),
            header: "Enable editing?",
            on_ok: move |_| {
                *edit_mode.write() = EditMode(true);
                *show.write() = false;
            },
            on_cancel: move |_| {
                *show.write() = false;
            },
            confirm: "I know what I am doing, enable edit mode",
            "Enabling edit mode may " strong {"damage"} " connected hardware if wrong parameters are configured."
        }
    }
}
