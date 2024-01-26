use indradb::Vertex;
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{PropertyType};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext};
use super::super::db::IdPath;
use super::super::VertexTypes;

pub struct Test<'a> {
    object: Object<'a>,
}

impl<'a> Test<'a> {
    pub fn init(db: &'a Db, config: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new test object");
        let o = db.create_object_and_init(VertexTypes::Test, &mut path, "", pop);
        db.add_object_properties(&o, &config, PropertyType::Base);

        Box::new(Test {
            object: Object {
                db,
                id: o.id,
                id_path: IdPath::new(path, VertexTypes::Test.name(), label, pop),
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }
}

implement_object_ext!(Test);