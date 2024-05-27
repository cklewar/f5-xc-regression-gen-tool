use std::any::Any;
use std::collections::HashMap;
use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{FeatureRenderContext, PropertyType, RegressionConfig, render_script, RenderContext,
            Renderer, ScriptFeatureRenderContext};
use crate::constants::{KEY_DATA, KEY_FEATURE, KEY_FILE, KEY_ID_PATH, KEY_MODULE, KEY_NAME, KEY_RELEASE, KEY_SCRIPT, KEY_SCRIPTS, KEY_SCRIPTS_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext, load_object_config};
use super::super::db::IdPath;
use super::super::VertexTypes;

#[typetag::serialize(tag = "type")]
pub trait FeatureExt<'a>: ObjectExt + Renderer<'a> + RenderContext {}

#[derive(serde::Serialize)]
pub struct Feature<'a> {
    object: Object<'a>,
}

impl<'a> Feature<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new feature object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Feature, &mut path, label, pop);
        db.add_object_property(&o, base_cfg, PropertyType::Base);
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o), label, &config);
        db.add_object_property(&o, &module_cfg, PropertyType::Module);

        Box::new(Feature {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg,
            },
        })
    }

    pub fn load(db: &'a Db, object: &VertexProperties, config: &RegressionConfig) -> Box<(dyn FeatureExt<'a> + 'a)> {
        error!("Loading feature object");
        let arr = object.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());
        let module = object.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&object.vertex), module, &config);

        Box::new(Feature {
            object: Object {
                db,
                id: object.vertex.id,
                id_path,
                vertex: object.vertex.clone(),
                module_cfg,
            },
        })
    }
}

#[typetag::serialize]
impl RenderContext for FeatureRenderContext {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[typetag::serialize]
impl RenderContext for Feature<'_> {
    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}

impl Renderer<'_> for Feature<'_> {
    fn gen_render_ctx(&self, config: &RegressionConfig, scripts: Vec<HashMap<String, Vec<String>>>) -> Box<dyn RenderContext> {
        Box::new(FeatureRenderContext {
            job: format!("{}_{}_{}", config.project.module, KEY_FEATURE,
                         self.get_base_properties().get(KEY_NAME).unwrap().as_str().unwrap()).replace('_', "-"),
            eut: config.eut.module.to_string(),
            base: self.get_base_properties(),
            module: self.get_module_properties(),
            project: config.project.clone(),
            scripts: scripts.clone(),
        })
    }

    fn gen_script_render_ctx(&self, config: &RegressionConfig) -> Vec<HashMap<String, Vec<String>>> {
        let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
        let module = self.get_base_properties().get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
        let props_base: Map<String, Value> = self.get_base_properties();
        let props_module: Map<String, Value> = self.get_module_properties();
        let scripts_path = props_module.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();

        for script in props_module.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
            let path = format!("{}/{}/{}/{}/{}", config.root_path,
                               config.features.path,
                               module, scripts_path,
                               script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
            let contents = std::fs::read_to_string(path).expect("panic while opening feature script file");
            let ctx = ScriptFeatureRenderContext {
                eut: config.eut.module.to_string(),
                name: module.to_string(),
                data: props_base.get(KEY_DATA).unwrap().as_str().unwrap().to_string(),
                module: module.to_string(),
                release: props_module.get(KEY_RELEASE).unwrap().as_str().unwrap().to_string(),
                project: config.project.clone(),
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
impl FeatureExt<'_> for Feature<'_> {}

implement_object_ext!(Feature);