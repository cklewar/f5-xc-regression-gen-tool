use indradb::Vertex;
use serde_json::{Map, Value};
use uuid::Uuid;
use crate::db::{Db, IdPath};
use crate::PropertyType;

pub struct Object<'a> {
    pub(crate) db: &'a Db,
    pub(crate) id: Uuid,
    pub(crate) vertex: Vertex,
    pub(crate) id_path: IdPath,
    pub(crate) module_cfg: Value,
}

pub trait ObjectExt {
    fn get_id(&self) -> Uuid;
    fn fn_a(&self) -> String;
    fn fn_b(&self) -> String;
    fn get_object(&self) -> Vertex;
    fn get_id_path(&self) -> &IdPath;
    fn get_module_cfg(&self) -> Map<String, Value>;
    fn get_base_properties(&self) -> Map<String, Value>;
    fn get_module_properties(&self) -> Map<String, Value>;
    fn insert_module_properties(&self, key: String, value: Value);
}

impl ObjectExt for Object<'_> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn fn_a(&self) -> String {
        "fn_a".to_string()
    }

    fn fn_b(&self) -> String {
        "fn_b".to_string()
    }

    fn get_object(&self) -> Vertex {
        self.vertex.to_owned()
    }

    fn get_id_path(&self) -> &IdPath {
        &self.id_path
    }

    fn get_module_cfg(&self) -> Map<String, Value> {
        self.module_cfg.as_object().unwrap().to_owned()
    }

    fn get_base_properties(&self) -> Map<String, Value> {
        let p = self.db.get_object_with_properties(&self.id).props;
        p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().to_owned()
    }

    fn get_module_properties(&self) -> Map<String, Value> {
        let p = self.db.get_object_properties(&self.vertex).unwrap().props;
        p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().to_owned()
    }

    fn insert_module_properties(&self, key: String, value: Value) {
        let mut p = self.get_module_properties().clone();
        p.insert(key, value);
        self.db.add_object_properties(&self.vertex, &p, PropertyType::Module);
    }
}