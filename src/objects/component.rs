use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::implement_object_ext;
use super::super::db::IdPath;
use super::super::VertexTypes;

pub struct ComponentSource<'a> {
    object: Object<'a>,
}

pub struct ComponentDestination<'a> {
    object: Object<'a>,
}

impl<'a> ComponentSource<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new component source object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::ComponentSrc, &mut path, label, pop);
        db.add_object_properties(&o, &base_cfg, PropertyType::Base);

        Box::new(ComponentSource {
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

impl<'a> ComponentDestination<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new component destination object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::ComponentDst, &mut path, label, pop);
        db.add_object_properties(&o, &base_cfg, PropertyType::Base);

        Box::new(ComponentDestination {
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

implement_object_ext!(ComponentSource, ComponentDestination);