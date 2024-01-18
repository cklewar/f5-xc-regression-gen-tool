use indradb::NamedProperty;
use log::error;
use uuid::Uuid;

use crate::db::Db;
use crate::PropertyType;

use super::super::db::IdPath;
use super::super::VertexTypes;

#[derive(Debug)]
pub struct Project {
    id: Uuid,
    id_path: IdPath,
}

impl Project {
    pub fn new(db: &Db, path: &mut Vec<String>, label: &str, pop: usize) -> Project {
        error!("Generating new project object");
        let o = db.create_object_with_gv(VertexTypes::Project, path, label, 0);

        Project {
            id: o.id,
            id_path: IdPath::new(path, VertexTypes::Project.name(), label, pop),
        }
    }

    pub fn init() {
        //let project = self.db.create_object_and_init(VertexTypes::Project, &mut id_path, &self.config.project.name, 0);
        //self.db.add_object_properties(&project, &self.config.project, PropertyType::Base);

    }

    pub fn get_base_properties(&self, db: &Db) -> NamedProperty {
        let o_p = db.get_object_with_properties(&self.id).props;
        let p = o_p.get(PropertyType::Base.index()).unwrap();
        p.clone()
    }
}