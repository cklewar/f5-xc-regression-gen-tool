use std::any::Any;
use std::collections::HashMap;
use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{CollectorRenderContext, EdgeTypes, PropertyType, RegressionConfig, render_script,
            RenderContext, Renderer, ScriptCollectorRenderContext};
use crate::constants::{EDGE_TYPE_TEST_REFERS_COLLECTION, KEY_ARTIFACTS_PATH, KEY_COLLECTOR, KEY_DATA, KEY_FILE, KEY_ID_PATH, KEY_MODULE, KEY_NAME, KEY_PROVIDER, KEY_RTE, KEY_SCRIPT, KEY_SCRIPTS, KEY_SCRIPTS_PATH};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{implement_object_ext, load_object_config, Rte, Test};
use super::super::db::IdPath;
use super::super::VertexTypes;

#[typetag::serialize(tag = "type")]
pub trait CollectorExt<'a>: ObjectExt + Renderer<'a> + RenderContext {}

#[derive(serde::Serialize)]
pub struct Collector<'a> {
    object: Object<'a>,
}

impl<'a> Collector<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new collector object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Collector, &mut path, label, pop);
        let collector_name = base_cfg.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
        let artifacts_path = format!("{}/{}/{}", config.collectors.artifacts_dir, collector_name, config.collectors.artifacts_file);
        let mut _base_cfg = base_cfg.as_object().unwrap().clone();
        _base_cfg.insert(KEY_ARTIFACTS_PATH.to_string(), json!(artifacts_path));
        db.add_object_property(&o, &json!(_base_cfg), PropertyType::Base);
        let module_name = base_cfg.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o), module_name, &config);
        db.add_object_property(&o, &module_cfg, PropertyType::Module);

        Box::new(Collector {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg,
            },
        })
    }

    pub fn load(db: &'a Db, object: &VertexProperties, config: &RegressionConfig) -> Box<(dyn CollectorExt<'a> + 'a)> {
        error!("Loading collector object");
        let arr = object.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());
        let module = object.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&object.vertex), module, &config);

        Box::new(Collector {
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
impl RenderContext for CollectorRenderContext {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[typetag::serialize]
impl RenderContext for Collector<'_> {
    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}

impl Renderer<'_> for Collector<'_> {
    fn gen_render_ctx(&self, config: &RegressionConfig, scripts: Vec<HashMap<String, Vec<String>>>) -> Box<dyn RenderContext> {
        Box::new(CollectorRenderContext {
            job: format!("{}_{}_{}", config.project.module, KEY_COLLECTOR,
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
        let mut collectibles: Vec<HashMap<String, String>> = vec![];

        for e in self.object.db.get_object_edges(&self.get_id()) {
            match e.t.as_str() {
                EDGE_TYPE_TEST_REFERS_COLLECTION => {
                    let t_o = Test::load(&self.object.db, &self.object.db.get_object(&e.outbound_id).id, &config);
                    let t_o_p_base = t_o.get_base_properties();
                    let test_name = t_o_p_base.get(KEY_NAME).unwrap().as_str().unwrap();
                    let test_module = t_o_p_base.get(KEY_MODULE).unwrap().as_str().unwrap();
                    let conn_src = self.object.db.get_object_neighbour_out_by_v_type(&t_o.get_id(), EdgeTypes::Runs, VertexTypes::ConnectionSrc);
                    let component_src = self.object.db.get_object_neighbour_out(&conn_src.unwrap().id, EdgeTypes::HasComponentSrc);
                    let components = self.object.db.get_object_neighbour_out_by_v_type(&component_src.unwrap().id, EdgeTypes::HasComponentSrc, VertexTypes::Components);
                    let rte_o = self.object.db.get_object_neighbour_out_by_v_type(&components.unwrap().id, EdgeTypes::HasComponents, VertexTypes::Rte);
                    let rte_p = self.object.db.get_object_properties(&rte_o.unwrap());
                    let rte = Rte::load(&self.object.db, &rte_p.unwrap(), &config);
                    let rte_p_base = rte.get_base_properties();
                    let rte_module = rte_p_base.get(KEY_MODULE).unwrap().as_str().unwrap();
                    let collectible: HashMap<String, String> = HashMap::from([
                        (KEY_RTE.to_string(), rte_module.to_string()),
                        (KEY_NAME.to_string(), test_name.to_string()),
                        (KEY_MODULE.to_string(), test_module.to_string()),
                        (KEY_PROVIDER.to_string(), rte.get_base_properties().get(KEY_PROVIDER).unwrap().as_str().unwrap().to_string()),
                    ]);
                    collectibles.push(collectible);
                }
                _ => {}
            };
        }

        for script in props_module.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
            let path = format!("{}/{}/{}/{}/{}", config.root_path,
                               config.collectors.path, module,
                               scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
            let contents = std::fs::read_to_string(path).expect("panic while opening collector script file");
            let ctx = ScriptCollectorRenderContext {
                eut: config.eut.module.to_string(),
                name: module.to_string(),
                data: props_base.get(KEY_DATA).unwrap().as_str().unwrap().to_string(),
                collectibles: collectibles.clone(),
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

        scripts
    }
}

#[typetag::serialize]
impl CollectorExt<'_> for Collector<'_> {}

implement_object_ext!(Collector);