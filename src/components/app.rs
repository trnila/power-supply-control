use crate::{
    components::{add_device::AddDeviceComponent, power_supply::PowerSupplyComponent},
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

    rsx! {
        style { {include_str!("../../assets/bootstrap.css")} },
        style { {include_str!("../../assets/main.css")} },

        for config in *config.read().data.power_supplies {
            PowerSupplyComponent {id: config.id.clone()}
        }

        AddDeviceComponent{}
    }
}
