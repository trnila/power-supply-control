use crate::{
    components::{
        add_device::AddDeviceComponent,
        config_selector::ConfigSelectorComponent,
        edit_mode::{EditMode, EditModeComponent},
        power_supply::PowerSupplyComponent,
    },
    config::AppConfig,
    get_config_dir,
};
use dioxus::prelude::*;

#[component]
pub fn AppComponent() -> Element {
    let config = use_context_provider(move || {
        Signal::new(AppConfig::load_from_file(
            get_config_dir().join("config.json"),
        ))
    });

    let edit_mode = use_context_provider(|| Signal::new(EditMode(false)));

    rsx! {
        style { {include_str!("../../assets/bootstrap.css")} },
        style { {include_str!("../../assets/main.css")} },

        div {
            class: "d-flex p-1",
            div {
                class: "me-auto",
                ConfigSelectorComponent {}
            }
            EditModeComponent {}
        }

        for config in *config.read().data.power_supplies {
            PowerSupplyComponent {id: config.id.clone()}
        }

        if edit_mode.read().0 {
            AddDeviceComponent{}
        }
    }
}
