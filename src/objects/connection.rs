use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{EdgeTypes, PropertyType, RegressionConfig};
use crate::constants::{KEY_ID_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext};
use super::super::db::IdPath;
use super::super::VertexTypes;

#[typetag::serialize(tag = "type")]
pub trait ConnectionExt<'a>: ObjectExt {}

#[typetag::serialize(tag = "type")]
pub trait ConnectionSourceExt<'a>: ObjectExt {}

#[typetag::serialize(tag = "type")]
pub trait ConnectionDestinationExt<'a>: ObjectExt {}

#[derive(serde::Serialize)]
pub struct Connection<'a> {
    object: Object<'a>,
}

#[derive(serde::Serialize)]
pub struct ConnectionSource<'a> {
    object: Object<'a>,
}

#[derive(serde::Serialize)]
pub struct ConnectionDestination<'a> {
    object: Object<'a>,
}

impl<'a> Connection<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new connection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Connection, &mut path, label, pop);
        db.add_object_property(&o, base_cfg, PropertyType::Base);

        Box::new(Connection {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load(db: &'a Db, object: &VertexProperties, _config: &RegressionConfig) -> Box<(dyn ConnectionExt<'a> + 'a)> {
        error!("Loading connection object");
        let arr = object.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());

        Box::new(Connection {
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

impl<'a> ConnectionSource<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new connection source object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::ConnectionSrc, &mut path, label, pop);
        db.add_object_property(&o, base_cfg, PropertyType::Base);

        Box::new(ConnectionSource {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load(db: &'a Db, object: &Vertex, _config: &RegressionConfig) -> Box<(dyn ConnectionSourceExt<'a> + 'a)> {
        error!("Loading connection source object");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasConnectionSrc).unwrap();
        let object_p = db.get_object_properties(&object).unwrap();
        let arr = object_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());

        Box::new(ConnectionSource {
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

impl<'a> ConnectionDestination<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new connection destination object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::ConnectionDst, &mut path, label, pop);
        db.add_object_property(&o, base_cfg, PropertyType::Base);

        Box::new(ConnectionDestination {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load(db: &'a Db, object: &VertexProperties, _config: &RegressionConfig) -> Box<(dyn ConnectionDestinationExt<'a> + 'a)> {
        error!("Loading connection destination object");
        let o = db.get_object_neighbour_with_properties_out(&object.vertex.id, EdgeTypes::HasConnectionDst).unwrap();
        let arr = object.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());

        Box::new(ConnectionDestination {
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
impl ConnectionExt<'_> for Connection<'_> {}

#[typetag::serialize]
impl ConnectionSourceExt<'_> for ConnectionSource<'_> {}

#[typetag::serialize]
impl ConnectionDestinationExt<'_> for ConnectionDestination<'_> {}

implement_object_ext!(Connection, ConnectionSource, ConnectionDestination);