use std::any::Any;
use std::collections::HashMap;
use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{EutSiteRenderContext, PropertyType, RegressionConfig, render_script, RenderContext,
            Renderer, ScriptApplicationRenderContext};
use crate::constants::{KEY_APPLICATION, KEY_ARTIFACTS_PATH, KEY_FILE, KEY_ID_PATH, KEY_MODULE,
                       KEY_NAME, KEY_PROVIDER, KEY_REF_ARTIFACTS_PATH, KEY_RELEASE, KEY_SCRIPT,
                       KEY_SCRIPTS, KEY_SCRIPTS_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext};
use super::super::db::IdPath;
use super::super::VertexTypes;

#[typetag::serialize(tag = "type")]
pub trait SiteExt<'a>: ObjectExt + Renderer<'a> + RenderContext {}

#[derive(serde::Serialize)]
pub struct Site<'a> {
    object: Object<'a>,
}

impl<'a> Site<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut site object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Site, &mut path, label, pop);
        let artifacts_path = format!("{}/{}/{}",
                                     config.eut.artifacts_dir,
                                     config.eut.module,
                                     config.eut.artifacts_file);
        let mut _base_cfg = base_cfg.as_object().unwrap().clone();
        _base_cfg.insert(KEY_ARTIFACTS_PATH.to_string(), json!(artifacts_path));
        db.add_object_property(&o, &json!(_base_cfg), PropertyType::Base);

        Box::new(Site {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load(db: &'a Db, object: &VertexProperties, _config: &RegressionConfig) -> Box<(dyn SiteExt<'a> + 'a)> {
        error!("Loading eut site object");
        let arr = object.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());

        Box::new(Site {
            object: Object {
                db,
                id: object.vertex.id,
                id_path,
                vertex: object.vertex.clone(),
                module_cfg: json!(null),
            },
        })
    }
}

#[typetag::serialize]
impl RenderContext for EutSiteRenderContext {
    fn as_any(&self) -> &dyn Any { self }
}

#[typetag::serialize]
impl RenderContext for Site<'_> {
    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}

impl Renderer<'_> for Site<'_> {
    fn gen_render_ctx(&self, config: &RegressionConfig, scripts: Vec<HashMap<String, Vec<String>>>) -> Box<dyn RenderContext> {
        let job: String;
        let provider: String = self.get_base_properties().get(KEY_PROVIDER).unwrap().as_str().unwrap().to_string();

        if provider.len() > 0 {
            job = format!("{}_{}_{}_{}", config.project.module, KEY_APPLICATION, self.get_module_properties().get(KEY_NAME).unwrap().as_str().unwrap(), provider).replace('_', "-")
        } else {
            job = format!("{}_{}_{}", config.project.module, KEY_APPLICATION, self.get_module_properties().get(KEY_NAME).unwrap().as_str().unwrap()).replace('_', "-")
        }

        Box::new(EutSiteRenderContext {
            job,
            name: "".to_string(),
            index: 0,
            scripts: scripts.clone(),
            provider,
        })
    }

    fn gen_script_render_ctx(&self, config: &RegressionConfig) -> Vec<HashMap<String, Vec<String>>> {
        let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
        let module = self.get_base_properties().get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
        let m_props: Map<String, Value> = self.get_module_properties();
        let base_props: Map<String, Value> = self.get_base_properties();
        let scripts_path = m_props.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();

        for script in m_props.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
            let path = format!("{}/{}/{}/{}/{}",
                               config.root_path,
                               config.applications.path, module,
                               scripts_path,
                               script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
            let contents = std::fs::read_to_string(path).expect("panic while opening eut site script file");

            let ctx = ScriptApplicationRenderContext {
                eut: config.eut.module.to_string(),
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
impl SiteExt<'_> for Site<'_> {}

implement_object_ext!(Site);