use indradb::Vertex;
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::implement_object_ext;
use super::super::db::IdPath;
use super::super::VertexTypes;

pub struct Ci<'a> {
    object: Object<'a>,

}

impl<'a> Ci<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new ci object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Ci, &mut path, label, pop);
        db.add_object_properties(&o, &config.ci, PropertyType::Base);

        Box::new(Ci {
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

implement_object_ext!(Ci);