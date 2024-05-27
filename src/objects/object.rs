use indradb::{Vertex, VertexProperties};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::db::{Db, IdPath};
use crate::PropertyType;

#[derive(serde::Serialize)]
pub struct Object<'a> {
    #[serde(skip_serializing)]
    pub(crate) db: &'a Db,
    pub(crate) id: Uuid,
    #[serde(skip_serializing)]
    pub(crate) vertex: Vertex,
    pub(crate) id_path: IdPath,
    pub(crate) module_cfg: Value,
}

pub trait ObjectExt {
    fn get_id(&self) -> Uuid;
    fn get_object(&self) -> Vertex;
    fn get_id_path(&self) -> &IdPath;
    fn get_module_cfg(&self) -> Map<String, Value>;
    fn get_base_properties(&self) -> Map<String, Value>;
    fn get_module_properties(&self) -> Map<String, Value>;
    fn get_object_with_properties(&self) -> VertexProperties;
    fn add_base_properties(&self, value: Value);
    fn add_module_properties(&self, value: Value);
    fn insert_base_property(&self, key: String, value: Value);
    fn insert_module_property(&self, key: String, value: Value);
}

impl ObjectExt for Object<'_> {
    fn get_id(&self) -> Uuid { self.vertex.id }

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

    fn get_object_with_properties(&self) -> VertexProperties {
        self.db.get_object_with_properties(&self.vertex.id)
    }

    fn add_base_properties(&self, value: Value) {
        self.db.add_object_property(&self.vertex, &value, PropertyType::Base);
    }

    fn add_module_properties(&self, value: Value) {
        self.db.add_object_property(&self.vertex, &value, PropertyType::Module);
    }

    fn insert_base_property(&self, key: String, value: Value) {
        let mut p = self.get_module_properties().clone();
        p.insert(key, value);
        self.db.add_object_property(&self.vertex, &p, PropertyType::Base);
    }

    fn insert_module_property(&self, key: String, value: Value) {
        let mut p = self.get_module_properties().clone();
        p.insert(key, value);
        self.db.add_object_property(&self.vertex, &p, PropertyType::Module);
    }
}