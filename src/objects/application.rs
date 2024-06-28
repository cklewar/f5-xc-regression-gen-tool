use std::any::Any;
use std::collections::HashMap;
use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{ApplicationRenderContext, EdgeTypes, PropertyType, RegressionConfig,
            render_script, RenderContext, Renderer, ScriptApplicationRenderContext};
use crate::constants::{KEY_APPLICATION, KEY_APPLICATIONS, KEY_ARTIFACTS_PATH, KEY_DATA, KEY_FILE, KEY_ID_PATH, KEY_MODULE, KEY_NAME, KEY_PROVIDER, KEY_REF_ARTIFACTS_PATH, KEY_RELEASE, KEY_SCRIPT, KEY_SCRIPTS, KEY_SCRIPTS_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext, load_object_config};
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
        let (o, id_path) = db.create_object_and_init(VertexTypes::Application,
                                                     &mut path,
                                                     base_cfg.get(KEY_NAME).unwrap().as_str().unwrap(),
                                                     pop);
        db.create_relationship(parent, &o);
        let applications = db.get_object_neighbour_in_out_id(&o.id,
                                                             EdgeTypes::ProvidesApplication,
                                                             VertexTypes::Applications).unwrap();
        let eut = db.get_object_neighbour_in_out_id(&applications.id,
                                                    EdgeTypes::HasApplications,
                                                    VertexTypes::Eut).unwrap();
        let eut_p = db.get_object_properties(&eut).unwrap();
        let eut_name = eut_p.props.get(PropertyType::Base.index()).
            unwrap().value.as_object().
            unwrap().get(KEY_MODULE).
            unwrap().as_str().
            unwrap().to_string();
        let a_name = base_cfg.get(KEY_NAME).unwrap().as_str().unwrap().to_string();
        let a_module = base_cfg.get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
        let a_provider = base_cfg.get(KEY_PROVIDER).unwrap().as_str().unwrap().to_string();
        let artifacts_path = format!("{}/{}/{}/{}/{}/{}/{}",
                                     config.applications.artifacts_dir,
                                     eut_name, KEY_APPLICATIONS.to_string(),
                                     a_module,
                                     a_provider,
                                     a_name,
                                     config.applications.artifacts_file);
        let mut _base_cfg = base_cfg.as_object().unwrap().clone();
        _base_cfg.insert(KEY_ARTIFACTS_PATH.to_string(), json!(artifacts_path));
        db.add_object_property(&o, &_base_cfg, PropertyType::Base);
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o), label, &config);
        db.add_object_property(&o, &module_cfg, PropertyType::Module);

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
        let provider: String = self.get_base_properties().get(KEY_PROVIDER).unwrap().as_str().unwrap().to_string();

        if provider.len() > 0 {
            job = format!("{}_{}_{}_{}_{}", config.project.module, KEY_APPLICATION, self.get_module_properties().get(KEY_NAME).unwrap().as_str().unwrap(), provider, self.get_base_properties().get(KEY_NAME).unwrap().as_str().unwrap()).replace('_', "-")
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

        for script in m_props.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
            let path = format!("{}/{}/{}/{}/{}/{}",
                               config.root_path,
                               config.applications.path,
                               module,
                               scripts_path,
                               base_props.get(KEY_PROVIDER).unwrap().as_str().unwrap(),
                               script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
            let contents = std::fs::read_to_string(path).expect("panic while opening application script file");
            let data_dir = format!("{}/{}/{}",
                                   base_props.get(KEY_MODULE).unwrap().as_str().unwrap(),
                                   base_props.get(KEY_PROVIDER).unwrap().as_str().unwrap(),
                                   base_props.get(KEY_DATA).unwrap().as_str().unwrap(),
            );
            let ctx = ScriptApplicationRenderContext {
                eut: config.eut.module.to_string(),
                name: base_props.get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                data: base_props.get(KEY_DATA).unwrap().as_str().unwrap().to_string(),
                refs: base_props.get(KEY_REF_ARTIFACTS_PATH).unwrap().as_object().unwrap().clone(),
                module: module.to_string(),
                project: config.project.clone(),
                release: m_props.get(KEY_RELEASE).unwrap().as_str().unwrap().to_string(),
                provider: base_props.get(KEY_PROVIDER).unwrap().as_str().unwrap().to_string(),
                data_dir,
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