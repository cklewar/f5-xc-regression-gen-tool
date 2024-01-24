use indradb::Vertex;
use log::error;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::db::Db;

use super::load_object_config;
use super::super::db::IdPath;
use super::super::VertexTypes;
use super::object::{Object, ObjectExt};

pub struct EutProvider<'a> {
    object: Object<'a>,
}

pub struct RteProvider<'a> {
    object: Object<'a>,
}

impl<'a> EutProvider<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut provider object");
        let o = db.create_object_and_init(VertexTypes::EutProvider, &mut path, "", 0);
        db.add_object_properties(&o, &config.eut, PropertyType::Base);
        let cfg = load_object_config(VertexTypes::get_name_by_object(&o), &config.eut.module, &config);

        Box::new(EutProvider {
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

impl<'a> RteProvider<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new rte provider object");
        let o = db.create_object_and_init(VertexTypes::RteProvider, &mut path, "", 0);
        db.add_object_properties(&o, &config.eut, PropertyType::Base);
        let cfg = load_object_config(VertexTypes::get_name_by_object(&o), &config.eut.module, &config);

        Box::new(RteProvider {
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

macro_rules! implement_object_ext {
    ($($structure:ident),* ) => {
        $(
            impl ObjectExt for $structure<'_> {
                fn get_id(&self) -> Uuid { self.object.get_id() }
                fn fn_a(&self) -> String { self.object.fn_a() }
                fn fn_b(&self) -> String { self.object.fn_b() }
                fn get_object(&self) -> Vertex { self.object.get_object() }
                fn get_id_path(&self) -> &IdPath { self.object.get_id_path() }
                fn get_module_cfg(&self) -> Map<String, Value> { self.object.get_module_cfg() }
                fn get_base_properties(&self) -> Map<String, Value> { self.object.get_base_properties() }
                fn get_module_properties(&self) -> Map<String, Value> { self.object.get_module_properties() }
                fn insert_module_properties(&self, key: String, value: Value) {
                    self.object.insert_module_properties(key, value)
                }
            }
        )*
    };
}

implement_object_ext!(EutProvider, RteProvider);