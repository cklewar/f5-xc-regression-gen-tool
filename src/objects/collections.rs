use indradb::{Vertex, VertexProperties};
use log::error;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{EdgeTypes, PropertyType, RegressionConfig, RenderContext};
use crate::constants::{KEY_APPLICATION, KEY_DEPLOY, KEY_DESTROY, KEY_ID_PATH, KEY_NAME, KEY_REPORT};
use crate::db::Db;
use crate::objects::application::ApplicationExt;
use crate::objects::feature::FeatureExt;
use crate::objects::rte::RteExt;
use crate::objects::collector::CollectorExt;
use crate::objects::component::{ComponentDestinationExt, ComponentSourceExt};
use crate::objects::report::{Report, ReportExt};

use super::{Rte, Application, Collector, Feature, implement_object_ext, ComponentDestination, ComponentSource};
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

pub struct Reports<'a> {
    object: Object<'a>,
}

impl<'a> Collectors<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new collectors collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Collectors, &mut path, label, pop);
        db.add_object_property(&o, &json!({"": ""}), PropertyType::Base);

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
        error!("Loading collector objects");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasCollectors).unwrap();
        let mut collectors: Vec<Box<(dyn CollectorExt + 'a)>> = Vec::new();
        let _collectors = db.get_object_neighbours_with_properties_out(&o.vertex.id, EdgeTypes::ProvidesCollector);

        for app in _collectors {
            collectors.push(Collector::load(db, &app, config));
        }

        collectors
    }

    pub fn load_collection(db: &'a Db, object: &Vertex, _config: &RegressionConfig) -> Box<(dyn ObjectExt + 'a)> {
        error!("Loading collector collection object");
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
        error!("Loading specific collector object");
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

    pub fn gen_render_ctx(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<Box<dyn RenderContext>> {
        let mut collectors_rc: Vec<Box<dyn RenderContext>> = Vec::new();
        let collectors = Self::load(db, object, config);

        for c in collectors {
            let collector_rc = c.gen_render_ctx(config, c.gen_script_render_ctx(config));
            collectors_rc.push(collector_rc);
        }

        collectors_rc
    }
}

impl<'a> Components<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new components collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Components, &mut path, label, pop);
        db.add_object_property(&o, &json!({"": ""}), PropertyType::Base);

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

    pub fn load_source_component(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Option<Box<(dyn ComponentSourceExt<'a> + 'a)>> {
        error!("Loading specific source component object");
        let src_component = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasComponentSrc);

        match src_component {
            None => None,
            Some(c) => {
                Some(ComponentSource::load(db, &c, config))
            }
        }
    }

    pub fn load_destination_component(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Option<Box<(dyn ComponentDestinationExt<'a> + 'a)>> {
        error!("Loading specific destination component object");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasComponents).unwrap();
        let dst_component = db.get_object_neighbour_with_properties_out(&o.vertex.id, EdgeTypes::HasComponentDst);

        match dst_component {
            None => None,
            Some(c) => {
                Some(ComponentDestination::load(db, &c, config))
            }
        }
    }


    pub fn load_collection(db: &'a Db, object: &Vertex, _config: &RegressionConfig) -> Box<(dyn ObjectExt + 'a)> {
        error!("Loading component collection object");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasComponents).unwrap();
        let arr = o.props.get(PropertyType::Base.index()).unwrap().value.as_object()
            .unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str()
            .unwrap().to_string()).collect());

        Box::new(Components {
            object: Object {
                db,
                id: o.vertex.id,
                id_path,
                vertex: o.vertex,
                module_cfg: json!(null),
            },
        })
    }
}

impl<'a> Connections<'a> {
    pub fn init(db: &'a Db, _config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new connections collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Connections, &mut path, label, pop);
        db.add_object_property(&o, &json!({"": ""}), PropertyType::Base);

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
        db.add_object_property(&o, &config.features, PropertyType::Base);

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

        for feature in _features {
            features.push(Feature::load(db, &feature, config));
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

    pub fn load_feature(db: &'a Db, object: &Vertex, name: &str, config: &RegressionConfig) -> Option<Box<(dyn FeatureExt<'a> + 'a)>> {
        error!("Loading specific eut feature object");
        let features = db.get_object_neighbours_with_properties_out(&object.id, EdgeTypes::HasFeature);

        for feature in features {
            let r = feature.props.get(PropertyType::Module.index()).unwrap().value.as_object()
                .unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string();
            if name == r {
                return Some(Feature::load(db, &feature, config));
            }
        }

        None
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
        db.add_object_property(&o, &json!({"": ""}), PropertyType::Base);

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
        db.add_object_property(&o, &config.rte, PropertyType::Base);

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

    pub fn load(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<Box<(dyn RteExt<'a> + 'a)>> {
        error!("Loading eut rte objects");
        let o = db.get_object_neighbour_out(&object.id, EdgeTypes::UsesRtes);
        let mut rtes: Vec<Box<(dyn RteExt + 'a)>> = Vec::new();
        let _rtes = db.get_object_neighbours_with_properties_out(&o.unwrap().id, EdgeTypes::ProvidesRte);

        for r in _rtes {
            rtes.push(Rte::load(db, &r, config));
        }

        rtes
    }

    pub fn load_collection(db: &'a Db, object: &Vertex, _config: &RegressionConfig) -> Box<(dyn ObjectExt + 'a)> {
        error!("Loading eut rte collection object");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::UsesRtes).unwrap();
        let arr = o.props.get(PropertyType::Base.index()).unwrap().value.as_object()
            .unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str()
            .unwrap().to_string()).collect());

        Box::new(Rtes {
            object: Object {
                db,
                id: o.vertex.id,
                id_path,
                vertex: o.vertex,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load_rte(db: &'a Db, object: &Vertex, name: &str, config: &RegressionConfig) -> Option<Box<(dyn RteExt<'a> + 'a)>> {
        error!("Loading specific eut rte object");
        let rtes = db.get_object_neighbours_with_properties_out(&object.id, EdgeTypes::ProvidesRte);

        for rte in rtes {
            let r = rte.props.get(PropertyType::Module.index()).unwrap().value.as_object()
                .unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string();
            if name == r {
                return Some(Rte::load(db, &rte, config));
            }
        }

        None
    }

    pub fn gen_render_ctx(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<Box<dyn RenderContext>> {
        let mut rtes_rc: Vec<Box<dyn RenderContext>> = Vec::new();
        let rtes = Self::load(db, object, config);

        for r in rtes {
            let feature_rc = r.gen_render_ctx(config, r.gen_script_render_ctx(config));
            rtes_rc.push(feature_rc);
        }

        rtes_rc
    }
}

impl<'a> Sites<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new eut sites collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Sites, &mut path, label, pop);
        db.add_object_property(&o, &config, PropertyType::Base);

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
        db.add_object_property(&o, &config.applications, PropertyType::Base);

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

impl<'a> Reports<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, mut path: &mut Vec<String>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new reports collection object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Reports, &mut path, label, pop);
        db.add_object_property(&o, &config.reports, PropertyType::Base);

        Box::new(Reports {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<Box<(dyn ReportExt<'a> + 'a)>> {
        error!("Loading report objects");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasReports).unwrap();
        let mut reports: Vec<Box<(dyn ReportExt + 'a)>> = Vec::new();
        let _reports = db.get_object_neighbours_with_properties_out(&o.vertex.id, EdgeTypes::ProvidesReport);

        for report in _reports {
            reports.push(Report::load(db, &report, config));
        }

        reports
    }

    pub fn load_collection(db: &'a Db, object: &Vertex, _config: &RegressionConfig) -> Box<(dyn ObjectExt + 'a)> {
        error!("Loading report collection object");
        let o = db.get_object_neighbour_with_properties_out(&object.id, EdgeTypes::HasReports).unwrap();
        let arr = o.props.get(PropertyType::Base.index()).unwrap().value.as_object()
            .unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str()
            .unwrap().to_string()).collect());

        Box::new(Reports {
            object: Object {
                db,
                id: o.vertex.id,
                id_path,
                vertex: o.vertex,
                module_cfg: json!(null),
            },
        })
    }

    pub fn load_report(db: &'a Db, object: &Vertex, name: &str, config: &RegressionConfig) -> Option<Box<(dyn ReportExt<'a> + 'a)>> {
        error!("Loading specific report object");
        let reports = db.get_object_neighbours_with_properties_out(&object.id, EdgeTypes::ProvidesReport);
        for report in reports {
            let a = report.props.get(PropertyType::Module.index()).unwrap().value.as_object()
                .unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string();
            if name == a {
                return Some(Report::load(db, &report, config));
            }
        }

        None
    }

    pub fn gen_deploy_stage(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<String> {
        error!("Generating report deploy stages");
        let mut stages: Vec<String> = Vec::new();
        let reports = Self::load(db, object, config);

        for r in reports {
            let name = format!("{}-{}-{}",
                               KEY_REPORT,
                               r.get_module_properties().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                               KEY_DEPLOY
            ).replace('_', "-");
            stages.push(name);
        }

        stages
    }

    pub fn gen_destroy_stage(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<String> {
        error!("Generating report destroy stages");
        let mut stages: Vec<String> = Vec::new();
        let reports = Self::load(db, object, config);

        for r in reports {
            let name = format!("{}-{}-{}",
                               KEY_REPORT,
                               r.get_module_properties().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                               KEY_DESTROY
            ).replace('_', "-");
            stages.push(name);
        }

        stages
    }

    pub fn gen_render_ctx(db: &'a Db, object: &Vertex, config: &RegressionConfig) -> Vec<Box<dyn RenderContext>> {
        let mut reports_rc: Vec<Box<dyn RenderContext>> = Vec::new();
        let reports = Self::load(db, object, config);

        for r in reports {
            let scripts = r.gen_script_render_ctx(config);
            let report_rc = r.gen_render_ctx(config, scripts.clone());
            reports_rc.push(report_rc);
        }

        reports_rc
    }
}

implement_object_ext!(Collectors, Components, Connections, Features, Providers, Rtes, Sites,
    Applications, Reports);