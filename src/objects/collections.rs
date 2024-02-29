use indradb::Vertex;
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::db::Db;

use super::implement_object_ext;
use super::object::{Object, ObjectExt};
use super::super::db::IdPath;
use super::super::VertexTypes;

pub struct Features<'a> {
    object: Object<'a>,
}

pub struct Providers<'a> {
    object: Object<'a>,
}

pub struct Sites<'a> {
    object: Object<'a>,
}

pub struct Rtes<'a> {
    object: Object<'a>,
}

pub struct Applications<'a> {
    object: Object<'a>,
}

impl<'a> Features<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut features collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Features, &mut path, label, pop);
        db.add_object_properties(&o, &config.features, PropertyType::Base);

        Box::new(Features {
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

impl<'a> Providers<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new providers collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Providers, &mut path, label, pop);
        db.add_object_properties(&o, &json!({"": ""}), PropertyType::Base);

        Box::new(Providers {
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

impl<'a> Rtes<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new rtes collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Rtes, &mut path, label, pop);
        db.add_object_properties(&o, &config.rte, PropertyType::Base);

        Box::new(Rtes {
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

impl<'a> Sites<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut sites collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Sites, &mut path, label, pop);
        db.add_object_properties(&o, &config, PropertyType::Base);

        Box::new(Sites {
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

impl<'a> Applications<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut applications collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Applications, &mut path, label, pop);
        db.add_object_properties(&o, &config.applications, PropertyType::Base);

        Box::new(Applications {
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

implement_object_ext!(Features, Providers, Rtes, Sites, Applications);