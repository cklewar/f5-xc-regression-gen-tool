use indradb::{NamedProperty, Vertex};
use log::error;
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig};
use crate::db::Db;

use super::super::db::IdPath;
use super::super::VertexTypes;

#[derive(Debug)]
pub struct Ci {
    id: Uuid,
    object: Vertex,
    id_path: IdPath,
}

impl Ci {
    pub fn init(db: &Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Ci {
        error!("Initialize new ci object");
        let o = db.create_object_and_init(VertexTypes::Ci, &mut path, "", 0);
        db.add_object_properties(&o, &config.ci, PropertyType::Base);

        Ci {
            id: o.id,
            id_path: IdPath::new(path, VertexTypes::Ci.name(), label, pop),
            object: o,
        }
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }

    pub fn get_object(&self) -> &Vertex {
        &self.object
    }

    pub fn get_id_path(&self) -> &IdPath {
        &self.id_path
    }

    pub fn get_base_properties(&self, db: &Db) -> NamedProperty {
        let o_p = db.get_object_with_properties(&self.id).props;
        let p = o_p.get(PropertyType::Base.index()).unwrap();
        p.clone()
    }
}