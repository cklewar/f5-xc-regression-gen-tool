use indradb::Vertex;
use log::error;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::db::Db;
use crate::objects::provider::RteProvider;

use super::{EutProvider, load_object_config};
use super::super::db::IdPath;
use super::super::VertexTypes;
use super::object::{Object, ObjectExt};


pub struct EutProviders<'a> {
    object: Object<'a>,
}

pub struct RteProviders<'a> {
    object: Object<'a>,
}

/*impl<'a> EutProviders<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut provider object");
        let o = db.create_object_and_init(VertexTypes::EutProvider, &mut path, "", 0);
        db.add_object_properties(&o, &config.eut, PropertyType::Base);
        let cfg = load_object_config(VertexTypes::get_name_by_object(&o), &config.eut.module, &config);

        Box::new(EutProviders {
            object: Object {
                db,
                id: o.id,
                id_path: IdPath::new(path, VertexTypes::Project.name(), label, pop),
                vertex: o,
                module_cfg: Default::default(),
            },
        })
    }
}

impl<'a> RteProviders<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new rte provider object");
        let o = db.create_object_and_init(VertexTypes::RteProvider, &mut path, "", 0);
        db.add_object_properties(&o, &config.eut, PropertyType::Base);
        let cfg = load_object_config(VertexTypes::get_name_by_object(&o), &config.eut.module, &config);

        Box::new(RteProviders {
            object: Object {
                db,
                id: o.id,
                id_path: IdPath::new(path, VertexTypes::Project.name(), label, pop),
                vertex: o,
                module_cfg: cfg,
            },
        })
    }
}*/

//implement_object_ext!(EutProvider, RteProvider);