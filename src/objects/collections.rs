use indradb::Vertex;
use log::error;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::db::Db;

use super::{implement_object_ext, load_object_config};
use super::super::db::IdPath;
use super::super::VertexTypes;
use super::object::{Object, ObjectExt};

pub struct Providers<'a> {
    object: Object<'a>,
}


impl<'a> Providers<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut providers collection object");
        let o = db.create_object_and_init(VertexTypes::Providers, &mut path, "", 0);
        db.add_object_properties(&o, &config.eut, PropertyType::Base);
        let cfg = load_object_config(VertexTypes::get_name_by_object(&o), &config.eut.module, &config);

        Box::new(Providers {
            object: Object {
                db,
                id: o.id,
                id_path: IdPath::new(path, VertexTypes::Project.name(), label, pop),
                vertex: o,
                module_cfg: cfg,
            },
        })
    }
}

implement_object_ext!(Providers);