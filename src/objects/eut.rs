use std::any::Any;
use std::collections::HashMap;
use indradb::{Vertex};
use log::error;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{EdgeTypes, PropertyType, RegressionConfig, RenderContext, Renderer};
use crate::constants::{KEY_ID_PATH, KEY_MODULE};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};
use crate::objects::project::ProjectExt;

use super::{implement_object_ext, load_object_config};
use super::super::db::IdPath;
use super::super::VertexTypes;

#[typetag::serialize(tag = "type")]
pub trait EutExt<'a>: ObjectExt + Renderer<'a> + RenderContext {}

#[derive(serde::Serialize)]
pub struct Eut<'a> {
    object: Object<'a>,
}

impl<'a> Eut<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Eut, &mut path, label, pop);
        db.add_object_properties(&o, &config.eut, PropertyType::Base);
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o), &config.eut.module, &config);
        db.add_object_properties(&o, &module_cfg, PropertyType::Module);

        Box::new(Eut {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg,
            },
        })
    }

    pub fn load(db: &'a Db, object: &Box<(dyn ProjectExt + 'a)>, config: &RegressionConfig) -> Box<(dyn EutExt<'a> + 'a)> {
        error!("Loading eut object");
        let o = db.get_object_neighbour_with_properties_out(&object.get_id(), EdgeTypes::HasEut).unwrap();
        let p_base = o.props.get(PropertyType::Base.index()).unwrap();
        let arr = p_base.value.get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());
        let module = p_base.value.get(KEY_MODULE).unwrap().as_str().unwrap();
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o.vertex), module, &config);

        Box::new(Eut {
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

impl Renderer<'_> for Eut<'_> {
    fn gen_render_ctx(&self, _config: &RegressionConfig, _ctx: Vec<HashMap<String, Vec<String>>>) -> Box<dyn RenderContext> {
        todo!()
    }

    fn gen_script_render_ctx(&self, _config: &RegressionConfig) -> Vec<HashMap<String, Vec<String>>> {
        todo!()
    }
}

#[typetag::serialize]
impl RenderContext for Eut<'_> {
    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}

#[typetag::serialize]
impl EutExt<'_> for Eut<'_> {}

implement_object_ext!(Eut);