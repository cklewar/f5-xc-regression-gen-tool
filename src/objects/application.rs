use std::any::Any;
use std::collections::HashMap;
use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{ApplicationRenderContext, EdgeTypes, PropertyType, RegressionConfig, render_script,
            RenderContext, Renderer, ScriptApplicationRenderContext};
use crate::constants::{KEY_APPLICATION, KEY_ARTIFACTS_PATH, KEY_FILE, KEY_ID_PATH, KEY_MODULE,
                       KEY_NAME, KEY_PROVIDER, KEY_REF_ARTIFACTS_PATH, KEY_RELEASE, KEY_SCRIPT,
                       KEY_SCRIPTS, KEY_SCRIPTS_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext, load_object_config, Rte};
use super::super::db::IdPath;
use super::super::VertexTypes;

#[typetag::serialize(tag = "type")]
pub trait ApplicationExt<'a>: ObjectExt + Renderer<'a> + RenderContext {}

#[derive(serde::Serialize)]
pub struct Application<'a> {
    object: Object<'a>,
}

impl<'a> Application<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, parent: &Vertex, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new application object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Application, &mut path, label, pop);
        db.add_object_property(&o, base_cfg, PropertyType::Base);
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o), label, &config);
        db.add_object_property(&o, &module_cfg, PropertyType::Module);
        db.create_relationship(parent, &o);

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

    pub fn load(db: &'a Db, object: &VertexProperties, config: &RegressionConfig) -> Box<(dyn ApplicationExt<'a> + 'a)> {
        error!("Loading application object");
        let arr = object.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());
        let module = object.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&object.vertex), module, &config);

        Box::new(Application {
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
impl RenderContext for ApplicationRenderContext {
    fn as_any(&self) -> &dyn Any { self }
}

#[typetag::serialize]
impl RenderContext for Application<'_> {
    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}

impl Renderer<'_> for Application<'_> {
    fn gen_render_ctx(&self, config: &RegressionConfig, scripts: Vec<HashMap<String, Vec<String>>>) -> Box<dyn RenderContext> {
        let job: String;
        let base_p = self.get_base_properties();
        let p = base_p.get(KEY_PROVIDER).unwrap().as_str().unwrap();

        if p.len() > 0 {
            job = format!("{}_{}_{}_{}", config.project.module, KEY_APPLICATION, self.get_module_properties().get(KEY_NAME).unwrap().as_str().unwrap(), p).replace('_', "-")
        } else {
            job = format!("{}_{}_{}", config.project.module, KEY_APPLICATION, self.get_module_properties().get(KEY_NAME).unwrap().as_str().unwrap()).replace('_', "-")
        }

        Box::new(ApplicationRenderContext {
            job,
            base: self.get_base_properties(),
            module: self.get_module_properties(),
            project: config.project.clone(),
            scripts: scripts.clone(),
        })
    }

    fn gen_script_render_ctx(&self, config: &RegressionConfig) -> Vec<HashMap<String, Vec<String>>> {
        let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
        let module = self.get_base_properties().get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
        let m_props: Map<String, Value> = self.get_module_properties();
        let base_props: Map<String, Value> = self.get_base_properties();
        let scripts_path = m_props.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
        let rte_p = self.object.db.get_object_neighbour_with_properties_out(&self.get_id(), EdgeTypes::RefersRte).unwrap();
        let rte = Rte::load(&self.object.db, &rte_p, &config);
        let rte_p_base = &rte.get_base_properties();

        for script in m_props.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
            let path = format!("{}/{}/{}/{}/{}", config.root_path, config.applications.path, module, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
            let contents = std::fs::read_to_string(path).expect("panic while opening application script file");
            let ctx = ScriptApplicationRenderContext {
                eut: config.eut.module.to_string(),
                rte: rte_p_base.get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                name: module.to_string(),
                refs: base_props.get(KEY_REF_ARTIFACTS_PATH).unwrap().as_object().unwrap().clone(),
                release: m_props.get(KEY_RELEASE).unwrap().as_str().unwrap().to_string(),
                provider: base_props.get(KEY_PROVIDER).unwrap().as_str().unwrap().to_string(),
                project: config.project.clone(),
                artifacts_path: base_props.get(KEY_ARTIFACTS_PATH).unwrap().as_str().unwrap().to_string(),
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
impl ApplicationExt<'_> for Application<'_> {}

implement_object_ext!(Application);