use indradb::Vertex;
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::constants::KEY_NAME;
use crate::db::Db;

use super::implement_object_ext;
use super::object::{Object, ObjectExt};
use super::super::db::IdPath;
use super::super::VertexTypes;

pub struct EutProvider<'a> {
    pub(crate) object: Object<'a>,
}

pub struct RteProvider<'a> {
    pub(crate) object: Object<'a>,
}

pub struct DashboardProvider<'a> {
    pub(crate) object: Object<'a>,
}

impl<'a> EutProvider<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut provider object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::EutProvider, &mut path, label, pop);
        db.add_object_properties(&o, &json!({KEY_NAME: label}), PropertyType::Base);

        Box::new(EutProvider {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }
}

impl<'a> DashboardProvider<'a> {
    pub fn init(db: &'a Db, config: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new dashboard provider object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::DashboardProvider, &mut path, label, pop);
        db.add_object_properties(&o, &json!({KEY_NAME: label}), PropertyType::Base);

        let provider = Box::new(DashboardProvider {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: config.clone(),
            },
        });

        provider.add_module_properties(config.clone());
        provider
    }
}

/*impl<'a> RteProvider<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new rte provider object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::RteProvider, &mut path, label, pop);
        db.add_object_properties(&o, &json!({KEY_NAME: label}), PropertyType::Base);

        Box::new(RteProvider {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }
}*/

implement_object_ext!(EutProvider, RteProvider, DashboardProvider);