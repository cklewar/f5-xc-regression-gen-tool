use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{EdgeTypes, PropertyType, RegressionConfig};
use crate::constants::{KEY_ID_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};
use crate::objects::project::ProjectExt;

use super::{implement_object_ext};
use super::super::db::IdPath;
use super::super::VertexTypes;

#[typetag::serialize(tag = "type")]
pub trait CiExt<'a>: ObjectExt {}

#[derive(serde::Serialize)]
pub struct Ci<'a> {
    object: Object<'a>,
}

impl<'a> Ci<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new ci object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Ci, &mut path, label, pop);
        db.add_object_property(&o, &base_cfg, PropertyType::Base);

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

    pub fn load(db: &'a Db, object: &Box<(dyn ProjectExt + 'a)>, _config: &RegressionConfig) -> Box<(dyn CiExt<'a> + 'a)> {
        error!("Loading ci object");
        let o = db.get_object_neighbour_with_properties_out(&object.get_id(), EdgeTypes::HasCi).unwrap();
        let arr = o.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());

        Box::new(Ci {
            object: Object {
                db,
                id: o.vertex.id,
                id_path,
                vertex: o.vertex.clone(),
                module_cfg: json!(null),
            },
        })
    }
}

#[typetag::serialize]
impl CiExt<'_> for Ci<'_> {}

implement_object_ext!(Ci);