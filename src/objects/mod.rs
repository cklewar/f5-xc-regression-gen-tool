use log::{info};
use serde_json::Value;
use serde_json::Value::Null;

pub use ci::Ci;
pub use collections::{Connections, Features, Providers, Rtes, Sites, Applications};
pub use dashboard::Dashboard;
pub use eut::{Eut, EutExt};
pub(crate) use macros::implement_object_ext;
pub use project::Project;
pub use provider::EutProvider;
pub use rte::Rte;
pub use site::Site;
pub use application::Application;
pub use test::Test;
pub use connection::{Connection, ConnectionDestination, ConnectionSource};
pub use feature::Feature;

use crate::constants::*;
use crate::RegressionConfig;

mod project;
mod ci;
mod eut;
mod provider;
pub(crate) mod object;
mod collections;
mod macros;
mod feature;
mod site;
mod rte;
mod test;
mod verification;
mod dashboard;
mod application;
mod collector;
mod connection;

fn load_object_config(_type: &str, module: &str, config: &RegressionConfig) -> Value {
    info!("Loading module <{module}> configuration data...");
    let file: String;
    match _type {
        KEY_EUT => {
            file = format!("{}/{}/{}/{}", config.root_path, config.eut.path, module, CONFIG_FILE_NAME);
        }
        KEY_RTE => {
            file = format!("{}/{}/{}/{}", config.root_path, config.rte.path, module, CONFIG_FILE_NAME);
        }
        KEY_TEST => {
            file = format!("{}/{}/{}/{}", config.root_path, config.tests.path, module, CONFIG_FILE_NAME);
        }
        KEY_FEATURE => {
            file = format!("{}/{}/{}/{}", config.root_path, config.features.path, module, CONFIG_FILE_NAME);
        }
        KEY_PROJECT => {
            file = format!("{}/{}/{}/{}", config.root_path, config.project.path, module, CONFIG_FILE_NAME);
        }
        KEY_SUMMARY => {
            file = format!("{}/{}/{}/{}", config.root_path, config.verifications.path, module, CONFIG_FILE_NAME);
        }
        KEY_DASHBOARD => {
            file = format!("{}/{}/{}/{}", config.root_path, config.dashboard.path, module, CONFIG_FILE_NAME);
        }
        KEY_APPLICATION => {
            file = format!("{}/{}/{}/{}", config.root_path, config.applications.path, module, CONFIG_FILE_NAME);
        }
        KEY_VERIFICATION => {
            file = format!("{}/{}/{}/{}", config.root_path, config.verifications.path, module, CONFIG_FILE_NAME);
        }
        _ => {
            return Null;
        }
    }

    let raw = std::fs::read_to_string(String::from(&file)).unwrap();
    let cfg: Value = serde_json::from_str(&raw).unwrap();
    info!("Loading module <{module}> configuration data -> Done.");
    cfg
}