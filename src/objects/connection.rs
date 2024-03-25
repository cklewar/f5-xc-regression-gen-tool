use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext};
use super::super::db::IdPath;
use super::super::VertexTypes;

pub struct Connection<'a> {
    object: Object<'a>,
}

pub struct ConnectionSource<'a> {
    object: Object<'a>,
}

pub struct ConnectionDestination<'a> {
    object: Object<'a>,
}

impl<'a> Connection<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new connection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Connection, &mut path, label, pop);
        db.add_object_properties(&o, base_cfg, PropertyType::Base);

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
}

impl<'a> ConnectionSource<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new connection source object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::ConnectionSrc, &mut path, label, pop);
        db.add_object_properties(&o, base_cfg, PropertyType::Base);

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
}

impl<'a> ConnectionDestination<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new connection destination object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::ConnectionDst, &mut path, label, pop);
        db.add_object_properties(&o, base_cfg, PropertyType::Base);

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
}

implement_object_ext!(Connection, ConnectionSource, ConnectionDestination);