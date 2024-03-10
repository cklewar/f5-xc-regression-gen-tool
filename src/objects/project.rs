use std::any::Any;
use std::collections::HashMap;
use indradb::{Vertex};
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{PropertyType, RegressionConfig, RenderContext, Renderer};
use crate::constants::{KEY_ID_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext};
use super::super::db::IdPath;
use super::super::VertexTypes;

#[typetag::serialize(tag = "type")]
pub trait ProjectExt<'a>: ObjectExt + Renderer<'a> + RenderContext {}

#[derive(serde::Serialize)]
pub struct Project<'a> {
    object: Object<'a>,
}

impl<'a> Project<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new project object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Project, &mut path, label, pop);
        db.add_object_properties(&o, &config.project, PropertyType::Base);
        Box::new(Project {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load(db: &'a Db, id: &Uuid, _config: &RegressionConfig) -> Box<(dyn ProjectExt<'a> + 'a)> {
        error!("Loading project object");
        db.get_object_with_properties(&id);
        let project = db.get_object_with_properties(&id);
        let arr = project.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());

        Box::new(Project {
            object: Object {
                db,
                id: project.vertex.id,
                id_path,
                vertex: project.vertex.clone(),
                module_cfg: json!(null),
            },
        })
    }
}

impl Renderer<'_> for Project<'_> {
    fn gen_render_ctx(&self, _config: &RegressionConfig, _ctx: Vec<HashMap<String, Vec<String>>>) -> Box<dyn RenderContext> {
        todo!()
    }

    fn gen_script_render_ctx(&self, _config: &RegressionConfig) -> Vec<HashMap<String, Vec<String>>> {
        todo!()
    }
}

impl RenderContext for Project<'_> {
    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn typetag_name(&self) -> &'static str {
        todo!()
    }
}

#[typetag::serialize]
impl ProjectExt<'_> for Project<'_> {}

implement_object_ext!(Project);