use std::any::Any;
use std::collections::HashMap;
use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{ObjectRefsMap, PropertyType, RegressionConfig, render_script, RenderContext, Renderer,
            RteTestRenderContext, ScriptTestRenderContext};
use crate::constants::{KEY_FILE, KEY_ID_PATH, KEY_MODULE, KEY_NAME, KEY_SCRIPT, KEY_SCRIPTS,
                       KEY_SCRIPTS_PATH, KEY_TEST};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext, load_object_config};
use super::super::db::IdPath;
use super::super::VertexTypes;

#[typetag::serialize(tag = "type")]
pub trait TestExt<'a>: ObjectExt + Renderer<'a> + RenderContext {}

#[derive(serde::Serialize)]
pub struct Test<'a> {
    object: Object<'a>,
}

impl<'a> Test<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new test object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Test, &mut path, label, pop);
        db.add_object_properties(&o, base_cfg, PropertyType::Base);
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o),
                                            base_cfg.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap(), &config);
        db.add_object_properties(&o, &module_cfg, PropertyType::Module);

        Box::new(Test {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg,
            },
        })
    }

    pub fn load(db: &'a Db, id: &Uuid, config: &RegressionConfig) -> Box<(dyn TestExt<'a> + 'a)> {
        error!("Loading test object");
        let o = db.get_object_with_properties(&id);
        let p_base = o.props.get(PropertyType::Base.index()).unwrap();
        let arr = o.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());
        let module = p_base.value.get(KEY_MODULE).unwrap().as_str().unwrap();
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o.vertex), module, &config);

        Box::new(Test {
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

#[typetag::serialize]
impl RenderContext for RteTestRenderContext {
    fn as_any(&self) -> &dyn Any { self }
}

#[typetag::serialize]
impl RenderContext for Test<'_> {
    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}

impl Renderer<'_> for Test<'_> {
    fn gen_render_ctx(&self, config: &RegressionConfig, scripts: Vec<HashMap<String, Vec<String>>>) -> Box<dyn RenderContext> {
        Box::new(RteTestRenderContext {
            ci: Default::default(),
            rte: "".to_string(),
            job: format!("{}_{}_{}", config.project.module, KEY_TEST, self.get_module_properties()
                .get(KEY_NAME).unwrap().as_str().unwrap()).replace('_', "-"),
            name: "".to_string(),
            data: "".to_string(),
            module: "".to_string(),
            provider: "".to_string(),
            scripts: scripts.clone(),
            verifications: vec![],
        })
    }

    fn gen_script_render_ctx(&self, config: &RegressionConfig) -> Vec<HashMap<String, Vec<String>>> {
        let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
        let module = self.get_base_properties().get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
        let m_props: Map<String, Value> = self.get_module_properties();
        let scripts_path = m_props.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();

        for script in m_props.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
            let path = format!("{}/{}/{}/{}/{}", config.root_path, config.rte.path, module, scripts_path, script.as_object()
                .unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
            let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
            let ctx = ScriptTestRenderContext {
                eut: config.eut.module.to_string(),
                name: "".to_string(),
                data: "".to_string(),
                rte: "".to_string(),
                project: config.project.clone(),
                provider: "".to_string(),
                features: vec![],
                module: module.clone(),
                refs: ObjectRefsMap { refs: Default::default() },
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
impl TestExt<'_> for Test<'_> {}

implement_object_ext!(Test);