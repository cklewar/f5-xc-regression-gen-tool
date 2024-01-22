use indradb::Vertex;
use log::error;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::db::Db;

use super::load_object_config;
use super::super::db::IdPath;
use super::super::VertexTypes;

#[derive(Debug)]
pub struct Eut {
    id: Uuid,
    object: Vertex,
    id_path: IdPath,
    module_cfg: Value,
}

impl Eut {
    pub fn init(db: &Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Eut {
        error!("Initialize new eut object");
        let o = db.create_object_and_init(VertexTypes::Eut, &mut path, "", 0);
        db.add_object_properties(&o, &config.eut, PropertyType::Base);
        let cfg = load_object_config(VertexTypes::get_name_by_object(&o), &config.eut.module, &config);

        Eut {
            id: o.id,
            id_path: IdPath::new(path, VertexTypes::Project.name(), label, pop),
            object: o,
            module_cfg: cfg,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_object(&self) -> Vertex {
        self.object.to_owned()
    }

    pub fn get_id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn get_module_cfg(&self) -> Map<String, Value> {
        self.module_cfg.as_object().unwrap().to_owned()
    }

    pub fn get_base_properties(&self, db: &Db) -> Map<String, Value> {
        let p = db.get_object_with_properties(&self.id).props;
        p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().to_owned()
    }

    pub fn get_module_properties(&self, db: &Db) -> Map<String, Value> {
        let p = db.get_object_properties(&self.object).unwrap().props;
        p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().to_owned()
    }

    pub fn insert_module_properties(&self, db: &Db, key: String, value: Value) {
        let mut p = self.get_module_properties(db).clone();
        p.insert(key, value);
        db.add_object_properties(&self.object, &p, PropertyType::Module);
    }
}