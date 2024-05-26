use dioxus::prelude::*;

use crate::components::editable_text::EditableTextComponent;
use crate::config::AppConfig;
use crate::{components::edit_mode::EditMode, get_config_dir};
use std::path::PathBuf;

fn new_config_name() -> PathBuf {
    let mut i = 1;
    loop {
        let path = get_config_dir().join(format!("config{i}.json"));
        if !path.exists() {
            return path;
        }
        i += 1;
    }
}

#[component]
pub fn ConfigSelectorComponent() -> Element {
    let edit_mode = use_context::<Signal<EditMode>>();
    let mut config = use_context::<Signal<AppConfig>>();
    let mut show = use_signal(|| false);

    let mut configs: Vec<(String, String)> = std::fs::read_dir(get_config_dir())
        .unwrap()
        .filter_map(|path| {
            let path = path.unwrap().path();
            if path.extension().unwrap() == "json" {
                let name = path.file_stem().unwrap().to_str().unwrap().to_string();
                Some((name.clone(), name))
            } else {
                None
            }
        })
        .collect();
    configs.sort_by_key(|(a, _)| a.clone().to_lowercase());

    rsx! {
        div {
            class: "input-group input-group-sm w-auto flex-nowrap",

            EditableTextComponent {
                onsubmit: move |new_name: String| {
                    config.write().rename(&new_name);
                },
                disabled: !edit_mode.read().0,
                text: config.read().name(),
            }

            if edit_mode.read().0 {
                div {
                    class: "dropdown",

                    button {
                        class: "ms-2 btn btn-sm btn-secondary dropdown-toggle",
                        onclick: move |_| {
                            let value = !*show.read();
                            *show.write() = value;
                        },
                        "Load config"
                    }

                    ul {
                        class: "dropdown-menu",
                        class: if *show.read() {"show"},
                        li {
                            class: "dropdown-item",
                            style: "cursor: pointer",
                            onclick: move |_| {
                                *config.write() = AppConfig::load_from_file(new_config_name());
                                config.write().save();
                                *show.write() = false;
                            },
                            span {
                                style: "vertical-align: text-bottom",
                                dangerous_inner_html: iconify::svg!("ic:baseline-add"),
                            }
                            "Add new config"
                        }
                        li {
                            hr {
                                class: "dropdown-divider"
                            }
                        }
                        for (name1, name2) in configs {
                            li {
                                class: "dropdown-item flex flex-nowrap",
                                style: "cursor: pointer; display: flex",
                                span {
                                    class: "flex-grow-1",
                                    onclick: move |_| {
                                        *config.write() = AppConfig::load_from_file(get_config_dir().join(format!("{name1}.json")));
                                        *show.write() = false;
                                    },
                                    {name1.clone()}
                                }

                                button {
                                    class: "btn p-0 text-danger flex-align-end",
                                    onclick: move |_| {
                                        std::fs::remove_file(get_config_dir().join(format!("{name2}.json"))).unwrap();
                                        *show.write() = false;
                                    },
                                    dangerous_inner_html: iconify::svg!("ph:trash"),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
