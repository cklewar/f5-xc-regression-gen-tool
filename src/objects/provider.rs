use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{EdgeTypes, PropertyType, RegressionConfig};
use crate::constants::{KEY_ID_PATH, KEY_NAME};
use crate::db::Db;

use super::{implement_object_ext};
use super::object::{Object, ObjectExt};
use super::super::db::IdPath;
use super::super::VertexTypes;

pub struct EutProvider<'a> {
    pub(crate) object: Object<'a>,
}

pub struct RteProvider<'a> {
    pub(crate) object: Object<'a>,
}

pub struct DashboardProvider<'a> {
    pub(crate) object: Object<'a>,
}

impl<'a> EutProvider<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut provider object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::EutProvider, &mut path, label, pop);
        db.add_object_property(&o, &json!({KEY_NAME: label}), PropertyType::Base);

        Box::new(EutProvider {
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

impl<'a> DashboardProvider<'a> {
    pub fn init(db: &'a Db, config: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new dashboard provider object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::DashboardProvider, &mut path, label, pop);
        db.add_object_property(&o, &json!({KEY_NAME: label}), PropertyType::Base);

        let provider = Box::new(DashboardProvider {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: config.clone(),
            },
        });

        provider.add_module_properties(config.clone());
        provider
    }

    pub fn load(db: &'a Db, object: &Vertex, _config: &RegressionConfig) -> Vec<Box<(dyn ObjectExt + 'a)>> {
        let objects = db.get_object_neighbours_with_properties_out(&object.id, EdgeTypes::UsesProvider);
        let mut providers: Vec<Box<(dyn ObjectExt + 'a)>> = vec![];

        for obj in objects {
            let as_arr = obj.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
            let id_path = IdPath::load_from_array(as_arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());
            let module_props = obj.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
            let provider = Box::new(DashboardProvider {
                object: Object {
                    db,
                    id: obj.vertex.id,
                    id_path,
                    vertex: obj.vertex,
                    module_cfg: Value::Object(module_props),
                },
            });
            providers.push(provider);
        }
        providers
    }
}

impl<'a> RteProvider<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new rte provider object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::RteProvider, &mut path, label, pop);
        db.add_object_property(&o, &json!({KEY_NAME: label}), PropertyType::Base);

        Box::new(RteProvider {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load(db: &'a Db, object: &Vertex, _config: &RegressionConfig) -> Box<(dyn ObjectExt + 'a)> {
        let obj = db.get_object_with_properties(&object.id);
        let as_arr = obj.props.get(PropertyType::Base.index())
            .unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(as_arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());

        Box::new(RteProvider {
            object: Object {
                db,
                id: obj.vertex.id,
                id_path,
                vertex: obj.vertex,
                module_cfg: json!(null),
            },
        })
    }
}

implement_object_ext!(EutProvider, RteProvider, DashboardProvider);