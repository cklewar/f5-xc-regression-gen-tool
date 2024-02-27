use std::collections::HashMap;

use indradb::Vertex;
use log::error;
use serde_json::{json, Map, to_value, Value};
use uuid::Uuid;

use crate::{DashboardRenderContext, EdgeTypes, PropertyType, RegressionConfig, render_script, RenderContext, Renderer, ScriptDashboardRenderContext};
use crate::constants::{KEY_FILE, KEY_ID_PATH, KEY_MODULE, KEY_NAME, KEY_PROVIDER, KEY_SCRIPT, KEY_SCRIPTS, KEY_SCRIPTS_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};
use crate::objects::provider::DashboardProvider;

use super::{implement_object_ext, load_object_config};
use super::super::db::IdPath;
use super::super::VertexTypes;

#[typetag::serialize(tag = "type")]
pub trait DashboardExt<'a>: ObjectExt + Renderer<'a> + RenderContext {}

#[derive(serde::Serialize)]
pub struct Dashboard<'a> {
    object: Object<'a>,
}

impl<'a> Dashboard<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new dashboard object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Dashboard, &mut path, label, pop);
        db.add_object_properties(&o, &config.dashboard, PropertyType::Base);
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o), &config.dashboard.module, &config);

        let dashboard = Box::new(Dashboard {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o.clone(),
                module_cfg: module_cfg.clone(),
            },
        });
        let mut i = 0;

        for (k, v) in dashboard.get_module_cfg() {
            match &k {
                k if k == KEY_PROVIDER => {
                    for (p, q) in v.as_object().unwrap() {
                        let mut cfg = q.as_object().unwrap().clone();

                        cfg.insert(KEY_NAME.to_string(), json!(p));
                        let provider = DashboardProvider::init(db, &to_value(cfg).unwrap(), path, p, i);
                        db.create_relationship(&dashboard.get_object(), &provider.get_object());
                        i = i + 1;
                    }
                }
                _ => {}
            }
            match &k {
                _ => {}
            }
        }

        dashboard
    }

    pub fn load(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Box<(dyn DashboardExt<'a> + 'a)> {
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::Has).unwrap();
        let arr = o.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());
        let module = o.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o.vertex), module, &config);

        let dashboard = Box::new(Dashboard {
            object: Object {
                db,
                id: o.vertex.id,
                id_path,
                vertex: o.vertex,
                module_cfg,
            },
        });

        dashboard
    }
}

#[typetag::serialize]
impl RenderContext for DashboardRenderContext {}

#[typetag::serialize]
impl RenderContext for Dashboard<'_> {}

impl Renderer<'_> for Dashboard<'_> {
    fn gen_render_ctx(&self, config: &RegressionConfig, scripts: Vec<HashMap<String, Vec<String>>>) -> Box<dyn RenderContext> {
        let p_name = self.get_base_properties().get(KEY_PROVIDER).unwrap().as_str().unwrap().to_string();
        let dashboard_provider = DashboardProvider::load(&self.object.db, &self.get_object(), config);
        let mut ctx: Box<DashboardRenderContext> = Box::new(DashboardRenderContext {
            base: Default::default(),
            module: Default::default(),
            project: Default::default(),
            provider: Default::default(),
            scripts: Default::default(),
        });

        for p in dashboard_provider {
            let m_props = p.get_module_properties();

            if p_name == m_props.get(KEY_NAME).unwrap().as_str().unwrap() {
                ctx = Box::new(DashboardRenderContext {
                    base: self.get_base_properties(),
                    module: self.get_module_properties(),
                    project: config.project.clone(),
                    provider: m_props.clone(),
                    scripts: scripts.clone(),
                });
            }
        }
        ctx
    }

    fn gen_script_render_ctx(&self, config: &RegressionConfig) -> Vec<HashMap<String, Vec<String>>> {
        let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
        let dashboard_provider = DashboardProvider::load(&self.object.db, &self.get_object(), config);
        let module = self.get_base_properties().get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
        let p_name = self.get_base_properties().get(KEY_PROVIDER).unwrap().as_str().unwrap().to_string();

        for provider in dashboard_provider {
            let m_props = provider.get_module_properties();
            let scripts_path = m_props.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();

            for script in m_props.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                let path = format!("{}/{}/{}/{}/{}/{}", config.root_path, config.dashboard.path, module, scripts_path, p_name, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                let contents = std::fs::read_to_string(path).expect("panic while opening dashboard script file");
                let ctx = ScriptDashboardRenderContext {
                    name: p_name.to_string(),
                    module: module.to_string(),
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
        }
        scripts
    }
}

#[typetag::serialize]
impl DashboardExt<'_> for Dashboard<'_> {}

implement_object_ext!(Dashboard);