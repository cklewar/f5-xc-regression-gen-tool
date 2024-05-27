use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::constants::{KEY_ID_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext};
use super::super::db::IdPath;
use super::super::VertexTypes;

#[typetag::serialize(tag = "type")]
pub trait ComponentSourceExt<'a>: ObjectExt {}

#[typetag::serialize(tag = "type")]
pub trait ComponentDestinationExt<'a>: ObjectExt {}

#[derive(serde::Serialize)]
pub struct ComponentSource<'a> {
    object: Object<'a>,
}

#[derive(serde::Serialize)]
pub struct ComponentDestination<'a> {
    object: Object<'a>,
}

impl<'a> ComponentSource<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new component source object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::ComponentSrc, &mut path, label, pop);
        db.add_object_property(&o, &base_cfg, PropertyType::Base);

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

    pub fn load(db: &'a Db, object: &VertexProperties, _config: &RegressionConfig) -> Box<(dyn ComponentSourceExt<'a> + 'a)> {
        error!("Loading component source object");
        let arr = object.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());

        Box::new(ComponentSource {
            object: Object {
                db,
                id: object.vertex.id,
                id_path,
                vertex: object.vertex.clone(),
                module_cfg: json!(null),
            },
        })
    }
}

#[typetag::serialize]
impl ComponentSourceExt<'_> for ComponentSource<'_> {}

impl<'a> ComponentDestination<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new component destination object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::ComponentDst, &mut path, label, pop);
        db.add_object_property(&o, &base_cfg, PropertyType::Base);

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

    pub fn load(db: &'a Db, object: &VertexProperties, _config: &RegressionConfig) -> Box<(dyn ComponentDestinationExt<'a> + 'a)> {
        error!("Loading component destination object");
        let arr = object.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());

        Box::new(ComponentDestination {
            object: Object {
                db,
                id: object.vertex.id,
                id_path,
                vertex: object.vertex.clone(),
                module_cfg: json!(null),
            },
        })
    }
}

#[typetag::serialize]
impl ComponentDestinationExt<'_> for ComponentDestination<'_> {}

implement_object_ext!(ComponentSource, ComponentDestination);