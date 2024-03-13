use std::any::Any;
use std::collections::HashMap;
use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{ProjectRenderContext, PropertyType, RegressionConfig, render_script, RenderContext,
            Renderer, ScriptProjectRenderContext};
use crate::constants::{KEY_FILE, KEY_ID_PATH, KEY_MODULE, KEY_PROJECT, KEY_RELEASE, KEY_SCRIPT,
                       KEY_SCRIPTS, KEY_SCRIPTS_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext, load_object_config};
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
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o), &config.project.module, &config);
        db.add_object_properties(&o, &module_cfg, PropertyType::Module);

        Box::new(Project {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg,
            },
        })
    }

    pub fn load(db: &'a Db, id: &Uuid, config: &RegressionConfig) -> Box<(dyn ProjectExt<'a> + 'a)> {
        error!("Loading project object");
        let o = db.get_object_with_properties(&id);
        let p_base = o.props.get(PropertyType::Base.index()).unwrap();
        let arr = o.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());
        let module = p_base.value.get(KEY_MODULE).unwrap().as_str().unwrap();
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o.vertex), module, &config);

        Box::new(Project {
            object: Object {
                db,
                id: o.vertex.id,
                id_path,
                vertex: o.vertex.clone(),
                module_cfg,
            },
        })
    }
}

impl Renderer<'_> for Project<'_> {
    fn gen_render_ctx(&self, config: &RegressionConfig, scripts: Vec<HashMap<String, Vec<String>>>) -> Box<dyn RenderContext> {
        Box::new(ProjectRenderContext {
            job: format!("{}_{}", config.project.module, KEY_PROJECT).replace('_', "-"),
            base: self.get_base_properties(),
            module: self.get_module_properties(),
            scripts: scripts.clone(),
        })
    }

    fn gen_script_render_ctx(&self, config: &RegressionConfig) -> Vec<HashMap<String, Vec<String>>> {
        let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
        let module = self.get_base_properties().get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
        let m_props = self.get_module_properties();
        let scripts_path = m_props.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();

        for script in m_props.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
            let path = format!("{}/{}/{}/{}/{}", config.root_path, config.project.path, module, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
            let contents = std::fs::read_to_string(path).expect("panic while opening project script file");
            let ctx = ScriptProjectRenderContext {
                project: config.project.clone(),
                release: m_props.get(KEY_RELEASE).unwrap().to_string(),
            };

            let mut commands: Vec<String> = Vec::new();
            for command in render_script(&ctx, &contents).lines() {
                commands.push(format!("{:indent$}{}", "", command, indent = 0));
            }

            let data: HashMap<String, Vec<String>> = [
                (script.as_object().unwrap().get(KEY_SCRIPT).unwrap().as_str().unwrap().to_string(), commands),
            ].into_iter().collect();
            scripts.push(data);
        }

        scripts
    }
}

#[typetag::serialize]
impl RenderContext for ProjectRenderContext {
    fn as_any(&self) -> &dyn Any { self }
}

#[typetag::serialize]
impl RenderContext for Project<'_> {
    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}

#[typetag::serialize]
impl ProjectExt<'_> for Project<'_> {}

implement_object_ext!(Project);