use indradb::Vertex;
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext, load_object_config};
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
        error!("CFG: {:?}", module_cfg);

        Box::new(Dashboard {
            object: Object {
                db,
                id: o.id,
                id_path: IdPath::new(path, VertexTypes::Dashboard.name(), label, pop),
                vertex: o,
                module_cfg,
            },
        })
    }
}

implement_object_ext!(Dashboard);