use dioxus::prelude::*;

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
                prevent_default: "onclick",
                onclick: move |_| {
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

        div {
            class: "modal fade show",
            class: if *show.read() {"d-block"},
            div {
                class: "modal-dialog",
                div {
                    class: "modal-content",
                    div {
                        class: "modal-header",
                        h1 {
                            class: "modal-title fs-5",
                            "Enable editing?"
                        }
                    }

                    div {
                        class: "modal-body",
                        "Enabling edit mode may "
                        strong {"damage"}
                        " connected hardware if wrong parameters are configured."
                    }

                    div {
                        class: "modal-footer",
                        button {
                            class: "btn btn-secondary",
                            onclick: move |_| *show.write() = false,
                            "Cancel"
                        }
                        button {
                            class: "btn btn-danger",
                            onclick: move |_| {
                                *edit_mode.write() = EditMode(true);
                                *show.write() = false;
                            },
                            "I know what I am doing, enable edit mode"
                        }
                    }
                }
            }
        }

        if *show.read() {
            div {
                class: "modal-backdrop fade show",
            }
        }
    }
}
