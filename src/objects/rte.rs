use std::any::Any;
use std::collections::{HashMap, HashSet};
use indradb::{Vertex, VertexProperties};
use log::{error, info};
use regex::Regex;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::{EdgeTypes, ObjRefs, PropertyType, RegressionConfig,
            render_script, RenderContext, Renderer, RteCiRenderContext, RteComponentRenderContext,
            RteCtxParameters, RteRenderContext, RteTestRenderContext, RteVerificationRenderContext,
            ScriptRteRenderContext, ScriptTestRenderContext, ScriptVerificationRenderContext};
use crate::constants::{KEY_ARTIFACTS_PATH, KEY_CI, KEY_COMPONENTS, KEY_CONNECTIONS, KEY_DATA,
                       KEY_DESTINATIONS, KEY_DST, KEY_FILE, KEY_ID_PATH, KEY_MODULE, KEY_NAME,
                       KEY_PROVIDER, KEY_REF_ARTIFACTS_PATH, KEY_RTE, KEY_SCRIPT,
                       KEY_SCRIPTS, KEY_SCRIPTS_PATH, KEY_SOURCE, KEY_SRC, KEY_TEST,
                       KEY_TESTS, KEY_TYPE, KEY_VERIFICATION, KEY_VERIFICATIONS, RTE_TYPE_A,
                       RTE_TYPE_B};
use crate::db::Db;
use crate::objects::object::{Object, ObjectExt};

use super::{Ci, ComponentDestination, Components, ComponentSource, Connection, ConnectionDestination,
            Connections, ConnectionSource, Eut, implement_object_ext, load_object_config, Project,
            Test, Verification};
use super::super::db::IdPath;
use super::super::VertexTypes;

#[typetag::serialize(tag = "type")]
pub trait RteExt<'a>: ObjectExt + Renderer<'a> + RenderContext {}

#[derive(serde::Serialize)]
pub struct Rte<'a> {
    object: Object<'a>,
}

impl<'a> Rte<'a> {
    pub fn init(db: &'a Db, config: &RegressionConfig, base_cfg: &Value, mut path: &mut Vec<String>, parent: &Box<(dyn ObjectExt + 'a)>, object_refs: &mut Vec<ObjRefs>, label: &str, pop: usize) -> Box<(dyn ObjectExt + 'a)> {
        error!("Initialize new rte object");
        let (o, id_path) = db.create_object_and_init(VertexTypes::Rte,
                                                     &mut path,
                                                     base_cfg.get(KEY_NAME).unwrap().as_str().unwrap(),
                                                     pop);
        db.create_relationship(&parent.get_object(), &o);
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&o), label, &config);
        let rte_provider = base_cfg.get(KEY_PROVIDER).unwrap().as_str().unwrap().to_string();
        let rte_name = base_cfg.get(KEY_NAME).unwrap().as_str().unwrap().to_string();
        let rte_module = base_cfg.get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
        let rte_type = module_cfg.get(KEY_TYPE).unwrap().as_str().unwrap().to_string();

        let _module_cfg: Value = json!({
            KEY_NAME: rte_module,
            KEY_TYPE: rte_type
        });

        let rte = Box::new(Rte {
            object: Object {
                db,
                id: o.id,
                id_path,
                vertex: o,
                module_cfg: module_cfg.clone(),
            },
        });

        rte.add_module_properties(json!(_module_cfg));

        let eut_o = db.get_object_neighbour_in_out_id(&parent.get_id(),
                                                      EdgeTypes::UsesRtes,
                                                      VertexTypes::Eut).unwrap();
        let project_o = db.get_object_neighbour_in_out_id(&eut_o.id,
                                                          EdgeTypes::HasEut,
                                                          VertexTypes::Project).unwrap();
        let project = Project::load(&db, &project_o.id, &config);
        let eut = Eut::load(&db, &project, &config);

        //RTE -> Features
        let eut_f_o = db.get_object_neighbour_out(&eut.get_id(), EdgeTypes::HasFeatures);
        db.create_relationship(&rte.get_object(), &eut_f_o.unwrap());

        //Rte components
        let rte_components = Components::init(&db, &config,
                                              &mut rte.get_id_path().get_vec(), "", 0);
        db.create_relationship(&rte.get_object(), &rte_components.get_object());

        for (k, v) in module_cfg.as_object().unwrap() {
            match k {
                k if k == KEY_COMPONENTS => {
                    if rte_type == RTE_TYPE_A {
                        let c_src_o = ComponentSource::init(&db,
                                                            &config,
                                                            v.get(KEY_SRC).unwrap(),
                                                            &mut rte_components.get_id_path().get_vec(),
                                                            "", 0);
                        db.create_relationship(&rte_components.get_object(), &c_src_o.get_object());
                        let c_dst_o = ComponentDestination::init(&db,
                                                                 &config, v.get(KEY_DST).unwrap(),
                                                                 &mut rte_components.get_id_path().get_vec(),
                                                                 "", 0);
                        db.create_relationship(&rte_components.get_object(), &c_dst_o.get_object());
                    } else if rte_type == RTE_TYPE_B {
                        let c_src_o = ComponentSource::init(&db,
                                                            &config, v.get(KEY_SRC).unwrap(),
                                                            &mut rte_components.get_id_path().get_vec(),
                                                            "", 0);
                        db.create_relationship(&rte_components.get_object(), &c_src_o.get_object());
                    }
                }
                k if k == KEY_CI => {
                    let rte_ci = Ci::init(db, &config, v, &mut rte.get_id_path().get_vec(),
                                          "", 0);
                    db.create_relationship(&rte.get_object(), &rte_ci.get_object());
                    rte_ci.add_base_properties(v.clone());
                }
                _ => {}
            }
        }

        //Rte connection src components
        let rte_src_component = Components::load_source_component(&db,
                                                                  &rte_components.get_object(),
                                                                  &config).unwrap();
        //Rte artifacts path
        let artifacts_path = format!("{}/{}/{}/{}/{}",
                                     config.rte.artifacts_dir,
                                     rte_module,
                                     rte_provider,
                                     rte_name,
                                     config.rte.artifacts_file);
        // Rte base cfg
        let _base_cfg: Value = json!({
            KEY_NAME: rte_name,
            KEY_MODULE: rte_module,
            KEY_TYPE: rte_type,
            KEY_PROVIDER: rte_provider,
            KEY_ARTIFACTS_PATH: artifacts_path
        });

        rte.add_base_properties(json!(&_base_cfg));

        //Rte
        for (k, v) in base_cfg.as_object().unwrap().iter() {
            match k {
                //Connections
                k if k == KEY_CONNECTIONS => {
                    let cs_o = Connections::init(&db, &config, &mut rte.get_id_path().get_vec(), "", 0);
                    db.create_relationship(&rte.get_object(), &cs_o.get_object());

                    for item in v.as_array().unwrap().iter() {
                        //Connection
                        let c_name = item.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                        let c_o = Connection::init(&db, &config,
                                                   &json!({KEY_NAME: c_name}),
                                                   &mut cs_o.get_id_path().get_vec(), "", 0);
                        db.create_relationship(&cs_o.get_object(), &c_o.get_object());

                        //Connection Source
                        let source = item.as_object().unwrap().get(KEY_SOURCE).unwrap().as_str().unwrap();
                        let src_o = ConnectionSource::init(&db, &config,
                                                           &json!({KEY_NAME: &source,
                                                               KEY_RTE: &base_cfg.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap()}),
                                                           &mut c_o.get_id_path().get_vec(),
                                                           "", 0);
                        db.create_relationship(&c_o.get_object(), &src_o.get_object());
                        let _sites = db.get_object_neighbour_out(&&eut.get_id(), EdgeTypes::HasSites);
                        let sites = db.get_object_neighbours_with_properties_out(&_sites.unwrap().id,
                                                                                 EdgeTypes::HasSite);

                        //Connection Source -> Site
                        for s in sites.iter() {
                            let site_name = s.props.get(PropertyType::Base.index()).unwrap().value.as_object().
                                unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                            if site_name == source {
                                db.create_relationship(&src_o.get_object(), &s.vertex);
                                //site --> rte
                                db.create_relationship(&s.vertex, &rte.get_object());
                            }
                        }

                        //Connection Destinations
                        let destinations = item.as_object().unwrap().
                            get(KEY_DESTINATIONS)
                            .unwrap().as_array().unwrap();

                        for d in destinations.iter() {
                            let re = Regex::new(d.as_str().unwrap()).unwrap();

                            for site in sites.iter() {
                                let site_name = site.props.get(PropertyType::Base.index())
                                    .unwrap().value.as_object()
                                    .unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                                if let Some(_t) = re.captures(site_name) {
                                    let dst_o = ConnectionDestination::init(&db,
                                                                            &config,
                                                                            &json!({KEY_NAME: &d,
                                                        KEY_RTE: &base_cfg.as_object().unwrap().get(KEY_MODULE)
                                                        .unwrap().as_str().unwrap()}), &mut c_o.get_id_path().get_vec(),
                                                                            "", 0);

                                    db.create_relationship(&src_o.get_object(), &dst_o.get_object());
                                    //Connection Destination -> Site
                                    db.create_relationship(&dst_o.get_object(), &site.vertex);
                                    //site --> rte
                                    db.create_relationship(&site.vertex, &rte.get_object());
                                }
                            }
                        }

                        //Tests
                        let tests = item.as_object().unwrap().get(KEY_TESTS)
                            .unwrap().as_array().unwrap();
                        for (index, test) in tests.iter().enumerate() {
                            let mut _index = 0;

                            match index {
                                0 => _index = 0,
                                1 => _index = index + 1,
                                _ => _index = index
                            }
                            let t_o = Test::init(&db, &config,
                                                 &test, &mut c_o.get_id_path().get_vec(),
                                                 &src_o.get_object(),
                                                 test[KEY_NAME].as_str().unwrap(),
                                                 _index);
                            let props = t_o.get_base_properties();

                            object_refs.push(ObjRefs {
                                refs: props.get("refs").unwrap().as_array().unwrap().clone(),
                                id: t_o.get_id(),
                            });

                            for (k, v) in test.as_object().unwrap().iter() {
                                match k {
                                    k if k == KEY_VERIFICATIONS => {
                                        for v in v.as_array().unwrap().iter() {
                                            let v_module = v.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                                            let v_o = Verification::init(&db, &config,
                                                                         &v, &mut t_o.get_id_path().get_vec(),
                                                                         v_module, 0);
                                            db.create_relationship(&t_o.get_object(), &v_o.get_object());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        //Rte type
        let rte_type_o = RteType::new(&rte_type, db);
        if let Some(r) = rte_type_o { r.init(&rte) }

        rte
    }

    pub fn load(db: &'a Db, object: &VertexProperties, config: &RegressionConfig) -> Box<(dyn RteExt<'a> + 'a)> {
        error!("Loading rte object");
        let arr = object.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let id_path = IdPath::load_from_array(arr.iter().map(|c| c.as_str().unwrap().to_string()).collect());
        let module = object.props.get(PropertyType::Base.index()).unwrap().
            value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
        let module_cfg = load_object_config(VertexTypes::get_name_by_object(&object.vertex), module, &config);

        Box::new(Rte {
            object: Object {
                db,
                id: object.vertex.id,
                id_path,
                vertex: object.vertex.clone(),
                module_cfg,
            }
        })
    }
}

#[typetag::serialize]
impl RenderContext for RteRenderContext {
    fn as_any(&self) -> &dyn Any { self }
}

#[typetag::serialize]
impl RenderContext for Rte<'_> {
    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}

impl Renderer<'_> for Rte<'_> {
    fn gen_render_ctx(&self, config: &RegressionConfig, scripts: Vec<HashMap<String, Vec<String>>>) -> Box<dyn RenderContext> {
        let rte_base_p = self.get_base_properties();
        let rte_name = rte_base_p.get(KEY_MODULE).unwrap().as_str().unwrap();
        let rte_ci = Ci::load(&self.object.db, &self.get_object(), &config);
        let rtes = self.object.db.get_object_neighbour_in_out_id(&self.get_id(), EdgeTypes::ProvidesRte, VertexTypes::Rtes).unwrap();
        let eut_o = self.object.db.get_object_neighbour_in_out_id(&rtes.id, EdgeTypes::UsesRtes, VertexTypes::Eut).unwrap();
        let project_o = self.object.db.get_object_neighbour_in_out_id(&eut_o.id, EdgeTypes::HasEut, VertexTypes::Project).unwrap();

        let rcrc = RteCiRenderContext {
            timeout: rte_ci.get_base_properties().get("timeout").unwrap().clone(),
            variables: rte_ci.get_base_properties().get("variables").unwrap().clone(),
            artifacts: rte_ci.get_base_properties().get("artifacts").unwrap().clone(),
        };

        let mut rte_crcs = RteRenderContext {
            ci: rcrc,
            name: rte_name.to_string(),
            base: self.get_base_properties(),
            tests: vec![],
            module: self.get_module_properties(),
            components: Default::default(),
        };

        //Process connections
        let rte_module_p = self.get_module_properties();
        let rte_type = rte_module_p.get(KEY_TYPE).unwrap().as_str().unwrap();
        let rte_type_o = RteType::new(rte_type, self.object.db);

        if let Some(r) = rte_type_o {
            r.build_conn_ctx(RteCtxParameters {
                rte: &self.get_object_with_properties(),
                config,
                project_config: config.project.clone(),
                eut: &self.object.db.get_object_with_properties(&eut_o.id),
                rte_name: rte_name.to_string(),
                rte_crcs: &mut rte_crcs,
                project: &project_o,
                rte_scripts: scripts,
            })
        }

        Box::new(rte_crcs)
    }

    fn gen_script_render_ctx(&self, config: &RegressionConfig) -> Vec<HashMap<String, Vec<String>>> {
        let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
        let module = self.get_base_properties().get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
        let rte_base_p: Map<String, Value> = self.get_base_properties();
        let rte_provider = rte_base_p.get(KEY_PROVIDER).unwrap().as_str().unwrap().to_string();
        let rte_artifacts_path = rte_base_p.get(KEY_ARTIFACTS_PATH).unwrap().as_str().unwrap().to_string();
        let components = Components::load_collection(&self.object.db, &self.get_object(), &config);
        let src_component = Components::load_source_component(&self.object.db, &components.get_object(), &config).unwrap();
        let src_component_base_p = src_component.get_base_properties();
        let scripts_path = src_component_base_p.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
        let src_component_name = src_component_base_p.get(KEY_NAME).unwrap().as_str().unwrap().to_string();

        for script in src_component_base_p.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
            let path = format!("{}/{}/{}/{}/{}/{}/{}", config.root_path, config.rte.path, module, scripts_path, rte_provider, src_component_name, script.as_object()
                .unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
            let contents = std::fs::read_to_string(path).expect("panic while opening rte script file");
            let ctx = ScriptRteRenderContext {
                eut: config.eut.module.to_string(),
                site: "".to_string(),
                base: self.get_base_properties(),
                module: self.get_module_properties(),
                release: "".to_string(),
                project: config.project.clone(),
                provider: rte_provider.clone(),
                destinations: "".to_string(),
                artifacts_path: rte_artifacts_path.clone(),
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

trait RteCharacteristics: {
    fn init<'b>(&self, rte: &Box<Rte<'b>>);
    fn build_conn_ctx(&self, params: RteCtxParameters);
}

struct RteTypeA<'a> {
    db: &'a Db,
}

impl<'a> RteCharacteristics for RteTypeA<'a> {
    fn init<'b>(&self, rte: &Box<Rte<'b>>) {
        error!("RTE TYPE A init connection components --> {:?}", &rte.get_base_properties().get(KEY_NAME).unwrap().as_str().unwrap());
        // Connection -> Component
        let _c = self.db.get_object_neighbour_out(&rte.get_id(), EdgeTypes::HasConnections);
        let connections = self.db.get_object_neighbours_out(&_c.unwrap().id, EdgeTypes::HasConnection);
        let _p = self.db.get_object_neighbour_out(&rte.get_id(), EdgeTypes::NeedsProvider);
        let rte_provider = self.db.get_object_neighbours_with_properties_out(&_p.unwrap().id, EdgeTypes::ProvidesProvider);

        for c in connections.iter() {
            let c_s = self.db.get_object_neighbour_with_properties_out(&c.id, EdgeTypes::HasConnectionSrc).unwrap();
            let site = self.db.get_object_neighbour_out(&c_s.vertex.id, EdgeTypes::RefersSite);
            let site_provider = self.db.get_object_neighbour_with_properties_out(&site.unwrap().id, EdgeTypes::UsesProvider).unwrap();
            let s_p_name = site_provider.props.get(PropertyType::Base.index()).unwrap().value.as_object().
                unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

            let _c_d_s: Vec<VertexProperties> = self.db.get_object_neighbours_with_properties_out(&c_s.vertex.id, EdgeTypes::HasConnectionDst);
            for p in rte_provider.iter() {
                let _components = self.db.get_object_neighbour_out(&p.vertex.id, EdgeTypes::HasComponents);
                let component_src = self.db.get_object_neighbour_out(&_components.unwrap().id, EdgeTypes::HasComponentSrc);
                let r_p_name = p.props.get(PropertyType::Module.index()).unwrap().value.as_object().
                    unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                if s_p_name == r_p_name {
                    self.db.create_relationship(&c_s.vertex, &component_src.unwrap());
                }
            }

            //CONNECTION DSTs
            for c_d in _c_d_s.iter() {
                for p in rte_provider.iter() {
                    let _components = self.db.get_object_neighbour_out(&p.vertex.id, EdgeTypes::HasComponents);
                    let component_dst = self.db.get_object_neighbour_out(&_components.unwrap().id, EdgeTypes::HasComponentDst);
                    self.db.create_relationship(&c_d.vertex, &component_dst.unwrap());
                }
            }
        }
        info!("Init rte type a connection components -> Done.");
    }

    fn build_conn_ctx(&self, params: RteCtxParameters) {
        error!("RTE TYPE A build conn ctx --> {}", params.rte_name);
        //Connection DST rt set
        let mut server_destinations: HashSet<String> = HashSet::new();

        let _c = self.db.get_object_neighbour_out(&params.rte.vertex.id, EdgeTypes::HasConnections);
        let connections = self.db.get_object_neighbours_with_properties_out(&_c.unwrap().id, EdgeTypes::HasConnection);
        let mut site_to_rte_map: HashMap<String, HashSet<String>> = HashMap::new();

        for conn in connections.iter() {
            let connection_name = conn.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            let src = self.db.get_object_neighbour_with_properties_out(&conn.vertex.id, EdgeTypes::HasConnectionSrc).unwrap();
            let src_name = src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            let src_site = self.db.get_object_neighbour_with_properties_out(&src.vertex.id, EdgeTypes::RefersSite).unwrap();
            let src_site_name = src_site.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            let src_provider = self.db.get_object_neighbour_with_properties_out(&src_site.vertex.id, EdgeTypes::UsesProvider).unwrap();
            let src_p_name = src_provider.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            let comp_src = self.db.get_object_neighbour_with_properties_out(&src.vertex.id, EdgeTypes::HasComponentSrc).unwrap();
            let comp_src_name = &comp_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            let rte_job_name = format!("{}_{}_{}_{}_{}_{}_{}", params.project_config.module, KEY_RTE, params.rte_name, &connection_name, &src_p_name, &src_name, &comp_src_name).replace('_', "-");

            //Process site_to_rte_map
            let mut _rtes: HashSet<String> = HashSet::new();
            _rtes.insert(params.rte_name.to_string());
            site_to_rte_map.entry(src_site_name.to_string()).or_insert(_rtes);

            //Process rte src component scripts
            //let scripts_path = comp_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
            let scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();

            //Process client destination list
            let mut client_destinations: HashSet<String> = HashSet::new();
            let dsts = self.db.get_object_neighbours_with_properties_out(&src.vertex.id, EdgeTypes::HasConnectionDst);
            for dst in dsts.iter() {
                client_destinations.insert(dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string());
            }

            /*for p in params.provider.iter() {
                let p_name = p.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                for script in comp_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                    if src_p_name == p_name {
                        let path = format!("{}/{}/{}/{}/{}/{}/{}", params.config.root_path, params.config.rte.path, params.rte_name, scripts_path, p_name, comp_src_name, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                        let contents = std::fs::read_to_string(&path).expect("panic while opening rte apply.script file");
                        let ctx = ScriptRteRenderContext {
                            rte: params.rte_name.to_string(),
                            eut: params.eut.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                            site: src_site_name.to_string(),
                            project: params.config.project.clone(),
                            release: "".to_string(),
                            provider: p_name.to_string(),
                            destinations: serde_json::to_string(&client_destinations).unwrap(),
                            artifacts_path: "".to_string(),
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
            }*/

            let rte_crc = RteComponentRenderContext {
                job: rte_job_name.clone(),
                rte: params.rte_name.to_string(),
                name: comp_src_name.to_string(),
                site: src_site_name.to_string(),
                provider: src_p_name.to_string(),
                scripts,
            };
            params.rte_crcs.components.push(rte_crc);

            //Process connection destinations
            for dst in dsts.iter() {
                let dst_p_base = dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap();
                let dst_name = dst_p_base.get(KEY_NAME).unwrap().as_str().unwrap();
                let dst_site = self.db.get_object_neighbour_with_properties_out(&dst.vertex.id, EdgeTypes::RefersSite).unwrap();
                let dst_site_name = dst_site.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let dst_provider = self.db.get_object_neighbour_with_properties_out(&dst_site.vertex.id, EdgeTypes::UsesProvider).unwrap();
                let dst_p_name = dst_provider.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let comp_dst = self.db.get_object_neighbour_with_properties_out(&dst.vertex.id, EdgeTypes::HasComponentDst).unwrap();
                let comp_dst_name = &comp_dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let rte_job_name = format!("{}_{}_{}_{}_{}_{}_{}", params.project_config.module, KEY_RTE, &params.rte_name, &connection_name, &dst_p_name, &dst_name, &comp_dst_name).replace('_', "-");

                //Process server destination list
                let rt_dsts = self.db.get_object_neighbours_with_properties_in(&dst.vertex.id, EdgeTypes::HasConnectionDst);
                for dst in rt_dsts.iter() {
                    server_destinations.insert(dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string());
                }

                //Process rte dst component scripts
                //let scripts_path = comp_dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                let scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();

                /*for p in params.provider.iter() {
                    let p_name = p.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                    for script in comp_dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                        if dst_p_name == p_name {
                            let path = format!("{}/{}/{}/{}/{}/{}/{}", params.config.root_path, params.config.rte.path, params.rte_name, scripts_path, p_name, comp_dst_name, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                            let contents = std::fs::read_to_string(path).expect("panic while opening rte apply.script file");
                            let ctx = ScriptRteRenderContext {
                                rte: params.rte_name.to_string(),
                                eut: params.eut.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                                site: dst_site_name.to_string(),
                                release: "".to_string(),
                                project: params.config.project.clone(),
                                provider: p_name.to_string(),
                                destinations: serde_json::to_string(&server_destinations).unwrap(),
                                artifacts_path: "".to_string(),
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
                }*/

                let rte_crc = RteComponentRenderContext {
                    job: rte_job_name.to_string(),
                    rte: params.rte_name.to_string(),
                    site: dst_site_name.to_string(),
                    name: comp_dst_name.to_string(),
                    scripts,
                    provider: dst_p_name.to_string(),
                };
                params.rte_crcs.components.push(rte_crc);
            }

            //Tests
            let tests_p = self.db.get_object_neighbours_with_properties_out(&src.vertex.id, EdgeTypes::Runs);
            for t in tests_p.iter() {
                let t_job_name = format!("{}_{}_{}_{}",
                                         params.project_config.module,
                                         KEY_TEST,
                                         src_name,
                                         t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap()
                ).replace('_', "-");

                //Process test scripts
                let t_p_base = t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();
                let t_p_module = t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();
                let t_name = t_p_base.get(KEY_NAME).unwrap().as_str().unwrap();
                let t_module = t_p_base.get(KEY_MODULE).unwrap().as_str().unwrap();
                let t_collector = self.db.get_object_neighbour_out(&t.vertex.id, EdgeTypes::TestRefersCollector);

                let collector = match t_collector {
                    Some(t) => self.db.get_object_properties(&t).unwrap().props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().to_string(),
                    None => "".to_string()
                };

                let scripts_path = t_p_module.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();

                for script in t_p_module.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                    let path = format!("{}/{}/{}/{}/{}", params.config.root_path, params.config.tests.path, t_module, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                    let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
                    let ctx = ScriptTestRenderContext {
                        rte: params.rte_name.to_string(),
                        eut: params.eut.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                        name: t_name.to_string(),
                        data: t_p_base.get(KEY_DATA).unwrap().as_str().unwrap().to_string(),
                        refs: t_p_base.get(KEY_REF_ARTIFACTS_PATH).unwrap().as_object().unwrap().clone(),
                        module: t_module.to_string(),
                        project: params.config.project.clone(),
                        provider: src_name.to_string(),
                        artifacts_path: "".to_string(),
                        rte_artifacts_path: "".to_string(),
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

                //Verifications
                let verifications_p = self.db.get_object_neighbours_with_properties_out(&t.vertex.id, EdgeTypes::Needs);
                let mut verifications: Vec<RteVerificationRenderContext> = Vec::new();
                for v in verifications_p.iter() {
                    let v_job_name = format!("{}_{}_{}_{}_{}",
                                             KEY_VERIFICATION,
                                             params.rte_name,
                                             src_name,
                                             &t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                             v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                    ).replace('_', "-");

                    //Process verification scripts
                    let v_name = v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                    let v_module = v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                    let v_data = v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_DATA).unwrap().as_str().unwrap();
                    let scripts_path = v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                    let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
                    for script in v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                        let path = format!("{}/{}/{}/{}/{}", params.config.root_path, params.config.verifications.path, v_module, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                        let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
                        let ctx = ScriptVerificationRenderContext {
                            rte_name: params.rte_name.to_string(),
                            rte_module: params.rte_name.to_string(),
                            rte_artifacts_path: "".to_string(),
                            name: v_name.to_string(),
                            data: v_data.to_string(),
                            module: v_module.to_string(),
                            provider: src_name.to_string(),
                            collector: collector.clone(),
                            test_name: t_name.to_string(),
                            test_module: t_module.to_string(),
                            test_artifacts_path: "".to_string(),
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

                    let rte_vrc = RteVerificationRenderContext {
                        ci: v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_CI).unwrap().as_object().unwrap().clone(),
                        test: t_name.to_string(),
                        rte: params.rte_name.to_string(),
                        job: v_job_name,
                        name: v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                        module: v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                        data: v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_DATA).unwrap().as_str().unwrap().to_string(),
                        scripts,
                    };
                    verifications.push(rte_vrc);
                }

                let rterc = RteTestRenderContext {
                    ci: t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_CI).unwrap().as_object().unwrap().clone(),
                    rte: params.rte_name.to_string(),
                    job: t_job_name,
                    name: t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                    data: t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_DATA).unwrap().as_str().unwrap().to_string(),
                    module: t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                    provider: src_name.to_string(),
                    scripts,
                    verifications,
                };
                params.rte_crcs.tests.push(rterc);
            }
        }
    }
}

struct RteTypeB<'a> {
    db: &'a Db,
}

impl<'a> RteCharacteristics for RteTypeB<'a> {
    fn init<'b>(&self, rte: &Box<Rte<'b>>) {
        error!("Init RTE TYPE B connection component --> {}", &rte.get_base_properties().get(KEY_NAME).unwrap().as_str().unwrap());
        // Connection -> Component
        let _c = self.db.get_object_neighbour_out(&rte.get_id(), EdgeTypes::HasConnections);
        let connections = self.db.get_object_neighbours_out(&_c.unwrap().id, EdgeTypes::HasConnection);

        for c in connections.iter() {
            let c_s = self.db.get_object_neighbour_with_properties_out(&c.id, EdgeTypes::HasConnectionSrc).unwrap();
            let _components = self.db.get_object_neighbour_out(&rte.get_id(), EdgeTypes::HasComponents);
            let component_src = self.db.get_object_neighbour_out(&_components.unwrap().id, EdgeTypes::HasComponentSrc);
            self.db.create_relationship(&c_s.vertex, &component_src.unwrap());
        }

        error!("Init RTE TYPE B connection component: {} --> Done.",  &rte.get_base_properties().get(KEY_NAME).unwrap().as_str().unwrap());
    }

    fn build_conn_ctx(&self, params: RteCtxParameters) {
        error!("RTE TYPE B build conn ctx --> {}", params.rte_name);
        let project = Project::load(&self.db, &params.project.id, &params.config);
        let eut = Eut::load(self.db, &project, &params.config);
        let eut_base_p = eut.get_base_properties();
        let eut_module = eut_base_p.get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
        let rte = Rte::load(&self.db, &params.rte, &params.config);
        let rte_base_p = rte.get_base_properties();
        let rte_provider = rte_base_p.get(KEY_PROVIDER).unwrap().as_str().unwrap();
        let _connections = Connections::load_collection(&self.db, &rte.get_object(), &params.config);
        let connections = Connections::load(&self.db, &_connections.get_object(), &params.config);

        for c in connections {
            let c_src = ConnectionSource::load(&self.db, &c.get_object(), &params.config);
            let c_src_base_p = c_src.get_base_properties();
            let c_src_name = c_src_base_p.get(KEY_NAME).unwrap().as_str().unwrap();
            let tests = self.db.get_object_neighbours_with_properties_out(&c_src.get_id(), EdgeTypes::Runs);
            let component_src_o = self.db.get_object_neighbour_with_properties_out(&c_src.get_id(), EdgeTypes::HasComponentSrc).unwrap();
            let component_src = ComponentSource::load(&self.db, &component_src_o, params.config);
            let component_src_base_p = component_src.get_base_properties();
            let component_src_name = component_src_base_p.get(KEY_NAME).unwrap().as_str().unwrap();

            let rte_job_name = format!("{}_{}_{}_{}_{}",
                                       params.project_config.module, KEY_RTE,
                                       params.rte_name,
                                       &rte_provider,
                                       &c_src_name).replace('_', "-");

            let rte_crc = RteComponentRenderContext {
                job: rte_job_name.clone(),
                rte: params.rte_name.to_string(),
                name: component_src_name.to_string(),
                site: "".to_string(),
                scripts: params.rte_scripts.clone(),
                provider: rte_provider.to_string(),
            };
            params.rte_crcs.components.push(rte_crc);

            for t in tests {
                let test = Test::load(&self.db, &t.vertex.id, &params.config);
                let test_base_p = test.get_base_properties();
                let test_module_p = test.get_module_properties();
                let test_name = test_base_p.get(KEY_NAME).unwrap().as_str().unwrap();
                let test_module = test_base_p.get(KEY_MODULE).unwrap().as_str().unwrap();
                let t_collector = self.db.get_object_neighbour_out(&t.vertex.id, EdgeTypes::TestRefersCollector);
                let collector = match t_collector {
                    Some(t) => {
                        self.db.get_object_properties(&t).unwrap().props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().to_string()
                    }
                    None => "".to_string()
                };
                let t_job_name = format!("{}_{}_{}",
                                         params.project_config.module,
                                         KEY_TEST,
                                         test_name).replace('_', "-");
                let scripts_path = test_module_p.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();

                for script in test_module_p.get(KEY_SCRIPTS).unwrap().as_array().unwrap() {
                    let path = format!("{}/{}/{}/{}/{}",
                                       params.config.root_path,
                                       params.config.tests.path,
                                       test_module, scripts_path,
                                       script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                    let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
                    let ctx = ScriptTestRenderContext {
                        rte: params.rte_name.to_string(),
                        eut: eut_module.to_string(),
                        name: test_name.to_string().replace('-', "_"),
                        data: test_base_p.get(KEY_DATA).unwrap().as_str().unwrap().to_string(),
                        refs: test_base_p.get(KEY_REF_ARTIFACTS_PATH).unwrap().as_object().unwrap().clone(),
                        module: test_module.to_string(),
                        project: params.config.project.clone(),
                        provider: rte_provider.to_string(),
                        artifacts_path: test_base_p.get(KEY_ARTIFACTS_PATH).unwrap().as_str().unwrap().to_string(),
                        rte_artifacts_path: rte_base_p.get(KEY_ARTIFACTS_PATH).unwrap().as_str().unwrap().to_string(),
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

                //Verifications
                let verifications_p = self.db.get_object_neighbours_with_properties_out(&t.vertex.id, EdgeTypes::Needs);
                let mut verifications: Vec<RteVerificationRenderContext> = Vec::new();

                for v in verifications_p.iter() {
                    //Process verification scripts
                    let v_p_base = v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap();
                    let v_p_module = v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();
                    let v_name = v_p_base.get(KEY_NAME).unwrap().as_str().unwrap();
                    let v_data = v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_DATA).unwrap().as_str().unwrap();
                    let v_module = v_p_base.get(KEY_MODULE).unwrap().as_str().unwrap();
                    let v_job_name = format!("{}_{}_{}",
                                             params.project_config.module,
                                             KEY_VERIFICATION,
                                             v_name).replace('_', "-");
                    let scripts_path = v_p_module.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                    let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
                    for script in v_p_module.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                        let path = format!("{}/{}/{}/{}/{}",
                                           params.config.root_path,
                                           params.config.verifications.path,
                                           v_module,
                                           scripts_path,
                                           script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                        let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
                        let ctx = ScriptVerificationRenderContext {
                            name: v_name.to_string(),
                            data: v_data.to_string(),
                            module: v_module.to_string(),
                            provider: rte_provider.to_string(),
                            collector: collector.clone(),
                            test_name: test_name.to_string(),
                            test_module: test_module.to_string(),
                            test_artifacts_path: test_base_p.get(KEY_ARTIFACTS_PATH).unwrap().as_str().unwrap().to_string(),
                            rte_name: rte_base_p.get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                            rte_module: rte_base_p.get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                            rte_artifacts_path: rte_base_p.get(KEY_ARTIFACTS_PATH).unwrap().as_str().unwrap().to_string(),
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

                    let rte_vrc = RteVerificationRenderContext {
                        ci: v_p_base.get(KEY_CI).unwrap().as_object().unwrap().clone(),
                        test: test_name.to_string(),
                        rte: params.rte_name.to_string(),
                        job: v_job_name,
                        name: v_p_base.get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                        module: v_p_base.get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                        data: v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_DATA).unwrap().as_str().unwrap().to_string(),
                        scripts,
                    };
                    verifications.push(rte_vrc);
                }

                let rtetrc = RteTestRenderContext {
                    ci: test_base_p.get(KEY_CI).unwrap().as_object().unwrap().clone(),
                    rte: params.rte_name.to_string(),
                    job: t_job_name,
                    name: test_base_p.get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                    data: test_base_p.get(KEY_DATA).unwrap().as_str().unwrap().to_string(),
                    module: test_base_p.get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                    provider: c_src_name.to_string(),
                    scripts,
                    verifications,
                };
                params.rte_crcs.tests.push(rtetrc);
            }
        }
    }
}

struct RteType<T> {
    rte: T,
    _type: String,
}

impl<'a> RteType<Box<dyn RteCharacteristics + 'a>> {
    fn new(rte_type: &str, db: &'a Db) -> Option<RteType<Box<dyn RteCharacteristics + 'a>>> {
        if rte_type == RTE_TYPE_A {
            Some(Self { rte: Box::new(RteTypeA { db }), _type: RTE_TYPE_A.to_string() })
        } else if rte_type == RTE_TYPE_B {
            Some(Self { rte: Box::new(RteTypeB { db }), _type: RTE_TYPE_B.to_string() })
        } else {
            None
        }
    }

    fn init(&self, rte: &Box<Rte<'_>>) {
        self.rte.init(rte);
    }

    fn build_conn_ctx(&self, params: RteCtxParameters) {
        self.rte.build_conn_ctx(params);
    }
}


#[typetag::serialize]
impl RteExt<'_> for Rte<'_> {}

implement_object_ext!(Rte);