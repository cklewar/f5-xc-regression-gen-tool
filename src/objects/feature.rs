use indradb::Vertex;
use log::error;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext, load_object_config};
use super::super::db::IdPath;
use super::super::VertexTypes;

pub struct Feature<'a> {
    object: Object<'a>,
}

impl<'a> Feature<'a>  {
    pub fn init(db: &'a Db, config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new feature object");
        let o = db.create_object_and_init(VertexTypes::Feature, &mut path, "", 0);
        db.add_object_properties(&o, base_cfg, PropertyType::Base);
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o), label, &config);
        db.add_object_properties(&o, &module_cfg, PropertyType::Module);

        Box::new(Feature {
            object: Object {
                db,
                id: o.id,
                id_path: IdPath::new(path, VertexTypes::Features.name(), label, pop),
                vertex: o,
                module_cfg,
            },
        })
    }
}

implement_object_ext!(Feature);