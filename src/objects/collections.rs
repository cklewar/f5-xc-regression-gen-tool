use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{EdgeTypes, PropertyType, RegressionConfig, RenderContext};
use crate::constants::{KEY_APPLICATION, KEY_DEPLOY, KEY_DESTROY, KEY_ID_PATH, KEY_NAME};
use crate::db::Db;
use crate::objects::application::ApplicationExt;
use crate::objects::feature::FeatureExt;
use crate::objects::collector::CollectorExt;

use super::{Application, Collector, Feature, implement_object_ext};
use super::object::{Object, ObjectExt};
use super::super::db::IdPath;
use super::super::VertexTypes;

pub struct Collectors<'a> {
    object: Object<'a>,
}

pub struct Components<'a> {
    object: Object<'a>,
}

pub struct Connections<'a> {
    object: Object<'a>,
}

pub struct Features<'a> {
    object: Object<'a>,
}

pub struct Providers<'a> {
    object: Object<'a>,
}

pub struct Sites<'a> {
    object: Object<'a>,
}

pub struct Rtes<'a> {
    object: Object<'a>,
}

pub struct Applications<'a> {
    object: Object<'a>,
}

impl<'a> Collectors<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new collectors collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Collectors, &mut path, label, pop);
        db.add_object_properties(&o, &json!({"": ""}), PropertyType::Base);

        Box::new(Collectors {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<Box<(dyn CollectorExt<'a> + 'a)>> {
        error!("Loading eut collector objects");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasCollectors).unwrap();
        let mut collectors: Vec<Box<(dyn CollectorExt + 'a)>> = Vec::new();
        let _collectors = db.get_object_neighbours_with_properties_out(&o.vertex.id, EdgeTypes::ProvidesCollector);

        for app in _collectors {
            collectors.push(Collector::load(db, &app, config));
        }

        collectors
    }

    pub fn load_collection(db: &'a Db, object: &Vertex, _config: &RegressionConfig) -> Box<(dyn ObjectExt + 'a)> {
        error!("Loading eut collector collection object");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasCollectors).unwrap();
        let arr = o.props.get(PropertyType::Base.index()).unwrap().value.as_object()
            .unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str()
            .unwrap().to_string()).collect());

        Box::new(Features {
            object: Object {
                db,
                id: o.vertex.id,
                id_path,
                vertex: o.vertex,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load_collector(db: &'a Db, object: &Vertex, name: &str, config: &RegressionConfig) -> Option<Box<(dyn CollectorExt<'a> + 'a)>> {
        error!("Loading specific eut collector object");
        let collectors = db.get_object_neighbours_with_properties_out(&object.id, EdgeTypes::ProvidesCollector);
        for collector in collectors {
            let a = collector.props.get(PropertyType::Module.index()).unwrap().value.as_object()
                .unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string();
            if name == a {
                return Some(Collector::load(db, &collector, config));
            }
        }

        None
    }
}

impl<'a> Components<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new components collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Components, &mut path, label, pop);
        db.add_object_properties(&o, &json!({"": ""}), PropertyType::Base);

        Box::new(Components {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }
}

impl<'a> Connections<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new connections collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Connections, &mut path, label, pop);
        db.add_object_properties(&o, &json!({"": ""}), PropertyType::Base);

        Box::new(Connections {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }
}

impl<'a> Features<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut features collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Features, &mut path, label, pop);
        db.add_object_properties(&o, &config.features, PropertyType::Base);

        Box::new(Features {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<Box<(dyn FeatureExt<'a> + 'a)>> {
        error!("Loading eut feature objects");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasFeatures).unwrap();
        let mut features: Vec<Box<(dyn FeatureExt + 'a)>> = Vec::new();
        let _features = db.get_object_neighbours_with_properties_out(&o.vertex.id, EdgeTypes::HasFeature);

        for app in _features {
            features.push(Feature::load(db, &app, config));
        }

        features
    }

    pub fn load_collection(db: &'a Db, object: &Vertex, _config: &RegressionConfig) -> Box<(dyn ObjectExt + 'a)> {
        error!("Loading eut feature collection object");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasFeatures).unwrap();
        let arr = o.props.get(PropertyType::Base.index()).unwrap().value.as_object()
            .unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str()
            .unwrap().to_string()).collect());

        Box::new(Features {
            object: Object {
                db,
                id: o.vertex.id,
                id_path,
                vertex: o.vertex,
                module_cfg: json!(null),
            },
        })
    }

    pub fn gen_render_ctx(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<Box<dyn RenderContext>> {
        let mut features_rc: Vec<Box<dyn RenderContext>> = Vec::new();
        let features = Self::load(db, object, config);

        for f in features {
            let feature_rc = f.gen_render_ctx(config, f.gen_script_render_ctx(config));
            features_rc.push(feature_rc);
        }

        features_rc
    }
}

impl<'a> Providers<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new providers collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Providers, &mut path, label, pop);
        db.add_object_properties(&o, &json!({"": ""}), PropertyType::Base);

        Box::new(Providers {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }
}

impl<'a> Rtes<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new rtes collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Rtes, &mut path, label, pop);
        db.add_object_properties(&o, &config.rte, PropertyType::Base);

        Box::new(Rtes {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }
}

impl<'a> Sites<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut sites collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Sites, &mut path, label, pop);
        db.add_object_properties(&o, &config, PropertyType::Base);

        Box::new(Sites {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }
}

impl<'a> Applications<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut applications collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Applications, &mut path, label, pop);
        db.add_object_properties(&o, &config.applications, PropertyType::Base);

        Box::new(Applications {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<Box<(dyn ApplicationExt<'a> + 'a)>> {
        error!("Loading eut application objects");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasApplications).unwrap();
        let mut applications: Vec<Box<(dyn ApplicationExt + 'a)>> = Vec::new();
        let _applications = db.get_object_neighbours_with_properties_out(&o.vertex.id, EdgeTypes::ProvidesApplication);

        for app in _applications {
            applications.push(Application::load(db, &app, config));
        }

        applications
    }

    pub fn load_collection(db: &'a Db, object: &Vertex, _config: &RegressionConfig) -> Box<(dyn ObjectExt + 'a)> {
        error!("Loading eut applications collection object");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasApplications).unwrap();
        let arr = o.props.get(PropertyType::Base.index()).unwrap().value.as_object()
            .unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str()
            .unwrap().to_string()).collect());

        Box::new(Applications {
            object: Object {
                db,
                id: o.vertex.id,
                id_path,
                vertex: o.vertex,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load_application(db: &'a Db, object: &Vertex, name: &str, config: &RegressionConfig) -> Option<Box<(dyn ApplicationExt<'a> + 'a)>> {
        error!("Loading specific eut application object");
        let applications = db.get_object_neighbours_with_properties_out(&object.id, EdgeTypes::ProvidesApplication);
        for app in applications {
            let a = app.props.get(PropertyType::Module.index()).unwrap().value.as_object()
                .unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string();
            if name == a {
                return Some(Application::load(db, &app, config));
            }
        }

        None
    }

    pub fn gen_deploy_stage(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<String> {
        error!("Generating eut application deploy stages");
        let mut stages: Vec<String> = Vec::new();
        let applications = Self::load(db, object, config);

        for a in applications {
            let name = format!("{}-{}-{}",
                               KEY_APPLICATION,
                               a.get_module_properties().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                               KEY_DEPLOY
            ).replace('_', "-");
            stages.push(name);
        }

        stages
    }

    pub fn gen_destroy_stage(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<String> {
        error!("Generating eut application destroy stages");
        let mut stages: Vec<String> = Vec::new();
        let applications = Self::load(db, object, config);

        for a in applications {
            let name = format!("{}-{}-{}",
                               KEY_APPLICATION,
                               a.get_module_properties().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                               KEY_DESTROY
            ).replace('_', "-");
            stages.push(name);
        }

        stages
    }

    pub fn gen_render_ctx(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<Box<dyn RenderContext>> {
        let mut applications_rc: Vec<Box<dyn RenderContext>> = Vec::new();
        let applications = Self::load(db, object, config);

        for a in applications {
            let scripts = a.gen_script_render_ctx(config);
            let application_rc = a.gen_render_ctx(config, scripts.clone());
            applications_rc.push(application_rc);
        }

        applications_rc
    }
}

implement_object_ext!(Collectors, Components, Connections, Features, Providers, Rtes, Sites,
    Applications);