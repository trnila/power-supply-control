use dioxus::prelude::*;

#[component]
pub fn ModalComponent(
    show: bool,
    header: String,
    children: Element,
    on_ok: EventHandler,
    confirm: String,
    on_cancel: EventHandler,
) -> Element {
    rsx! {
        div {
            class: "modal fade show",
            class: if show {"d-block"},
            div {
                class: "modal-dialog",
                div {
                    class: "modal-content",
                    div {
                        class: "modal-header",
                        h1 {
                            class: "modal-title fs-5",
                            {header}
                        }
                    }

                    div {
                        class: "modal-body",
                        {children}
                    }

                    div {
                        class: "modal-footer",
                        button {
                            class: "btn btn-secondary",
                            onclick: move |_| on_cancel(()),
                            "Cancel"
                        }
                        button {
                            class: "btn btn-danger",
                            onclick: move |_| on_ok(()),
                            {confirm}
                        }
                    }
                }
            }
        }

        if show {
            div {
                class: "modal-backdrop fade show",
            }
        }
    }
}
