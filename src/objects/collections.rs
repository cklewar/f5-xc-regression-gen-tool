use clap::parser::ValueSource::DefaultValue;
use indradb::Vertex;
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::db::Db;

use super::{implement_object_ext};
use super::super::db::IdPath;
use super::super::VertexTypes;
use super::object::{Object, ObjectExt};

pub struct Features<'a> {
    object: Object<'a>,
}

pub struct Providers<'a> {
    object: Object<'a>,
}

impl<'a> Features<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut features collection object");
        let o = db.create_object_and_init(VertexTypes::Features, &mut path, "", 0);
        db.add_object_properties(&o, &config.features, PropertyType::Base);

        Box::new(Features {
            object: Object {
                db,
                id: o.id,
                id_path: IdPath::new(path, VertexTypes::Features.name(), label, pop),
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }
}

impl<'a> Providers<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut providers collection object");
        let o = db.create_object_and_init(VertexTypes::Providers, &mut path, "", 0);
        db.add_object_properties(&o, &json!({"": ""}), PropertyType::Base);

        Box::new(Providers {
            object: Object {
                db,
                id: o.id,
                id_path: IdPath::new(path, VertexTypes::Providers.name(), label, pop),
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }
}

implement_object_ext!(Providers, Features);