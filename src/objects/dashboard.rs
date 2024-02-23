use indradb::Vertex;
use log::error;
use serde_json::{json, Map, to_value, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::constants::{KEY_CI, KEY_NAME, KEY_PROVIDER, KEY_RELEASE, KEY_SCRIPTS, KEY_SCRIPTS_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{dashboard, EutProvider, implement_object_ext, load_object_config};
use super::super::db::IdPath;
use super::super::VertexTypes;

pub struct Dashboard<'a> {
    object: Object<'a>,
}

impl<'a> Dashboard<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new dashboard object");
        let o = db.create_object_and_init(VertexTypes::Dashboard, &mut path, "", pop);
        db.add_object_properties(&o, &config.dashboard, PropertyType::Base);
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o), &config.dashboard.module, &config);

        let dashboard = Box::new(Dashboard {
            object: Object {
                db,
                id: o.id,
                id_path: IdPath::new(path, VertexTypes::Dashboard.name(), label, pop),
                vertex: o,
                module_cfg: module_cfg.clone(),
            },
        });

        for (k, v) in module_cfg.as_object().unwrap() {
            match k {
                k if k == KEY_SCRIPTS_PATH => {
                    dashboard.add_module_properties(to_value(json!({k: v.clone()}).as_object().unwrap().clone()).unwrap());
                }
                k if k == KEY_RELEASE => {
                    dashboard.add_module_properties(to_value(json!({k: v.clone()}).as_object().unwrap().clone()).unwrap());
                }
                k if k == KEY_NAME => {
                    dashboard.add_module_properties(to_value(json!({k: v.clone()}).as_object().unwrap().clone()).unwrap());
                }
                k if k == KEY_CI => {
                    dashboard.add_module_properties(to_value(json!({k: v.clone()}).as_object().unwrap().clone()).unwrap());
                }
                k if k == KEY_SCRIPTS => {
                    dashboard.add_module_properties(to_value(json!({k: v.clone()}).as_object().unwrap().clone()).unwrap());
                }
                _ => {}
            }
        }

        dashboard
    }
}

implement_object_ext!(Dashboard);