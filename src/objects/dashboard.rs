use std::collections::HashMap;

use indradb::Vertex;
use log::error;
use serde_json::{json, Map, to_value, Value};
use uuid::Uuid;

use crate::{EdgeTypes, PropertyType, RegressionConfig, render_script, RenderContext, ScriptDashboardRenderContext};
use crate::constants::{KEY_FILE, KEY_ID_PATH, KEY_NAME, KEY_PROVIDER, KEY_SCRIPT, KEY_SCRIPTS, KEY_SCRIPTS_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};
use crate::objects::provider::DashboardProvider;

use super::{implement_object_ext, load_object_config};
use super::super::db::IdPath;
use super::super::VertexTypes;

pub trait DashboardExt<'a>:ObjectExt + RenderContext<'a> {}

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

    pub fn load(db: &'a Db, object: &Vertex) -> Box<(dyn DashboardExt<'a> + 'a)> {
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::Has).unwrap();
        let arr = o.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());

        let dashboard = Box::new(Dashboard {
            object: Object {
                db,
                id: object.id,
                id_path,
                vertex: o.vertex,
                module_cfg: json!(null),
            },
        });

        dashboard
    }
}
impl DashboardExt<'_> for Dashboard<'_> {}
impl RenderContext<'_> for Dashboard<'_> {
    fn gen_render_ctx(&self, _config: &RegressionConfig, _ctx: Vec<HashMap<String, Vec<String>>>) -> Box<(dyn RenderContext + 'static)> {
        todo!()
    }

    fn gen_script_render_ctx(&self, config: &RegressionConfig) -> Vec<HashMap<String, Vec<String>>> {
        let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
        let dashboard_provider = self.object.db.get_object_neighbours_with_properties_out(&self.get_id(), EdgeTypes::UsesProvider);

        for provider in dashboard_provider {
            error!("PROVIDER: {:#?}", provider.props.get(PropertyType::Base.index()));
            let module_props = self.get_module_properties();
            let d_name = module_props.get(KEY_NAME).unwrap().as_str().unwrap().to_string();
            let scripts_path = module_props.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
            for script in module_props.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                let path = format!("{}/{}/{}/{}/{}", config.root_path, config.dashboard.path, d_name, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                let contents = std::fs::read_to_string(path).expect("panic while opening dashboard script file");
                let ctx = ScriptDashboardRenderContext {
                    name: d_name.to_string(),
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

implement_object_ext!(Dashboard);