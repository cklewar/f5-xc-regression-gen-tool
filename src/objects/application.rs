use indradb::Vertex;
use log::error;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{EdgeTypes, PropertyType, RegressionConfig};
use crate::constants::{KEY_ID_PATH, KEY_MODULE};
use crate::db::Db;
use crate::objects::dashboard::DashboardExt;
use crate::objects::object::{Object, ObjectExt};

use super::{Dashboard, implement_object_ext, load_object_config};
use super::super::db::IdPath;
use super::super::VertexTypes;

pub struct Application<'a> {
    object: Object<'a>,
}

impl<'a> Application<'a>  {
    pub fn init(db: &'a Db, config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new application object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Application, &mut path, label, pop);
        db.add_object_properties(&o, base_cfg, PropertyType::Base);
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o), label, &config);
        db.add_object_properties(&o, &module_cfg, PropertyType::Module);

        Box::new(Application {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg,
            },
        })
    }

    pub fn load(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Box<(dyn DashboardExt<'a> + 'a)> {
        error!("Loading application object");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::ProvidesApplication).unwrap();
        let arr = o.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());
        let module = o.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o.vertex), module, &config);

        let application = Box::new(Application {
            object: Object {
                db,
                id: o.vertex.id,
                id_path,
                vertex: o.vertex,
                module_cfg,
            },
        });

        application
    }
}

implement_object_ext!(Application);