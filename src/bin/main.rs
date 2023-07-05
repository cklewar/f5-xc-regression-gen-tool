//! F5 XC regression test CI pipeline file generator.
//! Provides command line tool to generate Gitlab CI pipeline file.
//!
//! Consumes input from regression configuration file provided as command line argument.
//! Template file relays on tool provided data structure to render stage, job or variables sections.
//! Tool supports direct rendering of given template file or generates JSON output which could be
//! used as input for another program or workflow.
//!
//! Supported command line arguments:
//! --config <provide regression environment configuration file>

use indradb;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::string::ToString;
use clap::Parser;
use indradb::{BulkInsertItem, Identifier, RangeVertexQuery, Vertex, VertexProperties, VertexProperty};
use tera::{Context, Tera};
use lazy_static::lazy_static;
use std::option::Option;
use std::os::unix::fs::chroot;
use uuid::Uuid;
use serde_json::{json, Value};

const CONFIG_FILE_NAME: &str = "config.json";

const SCRIPT_TYPE_APPLY: &str = "apply";
const SCRIPT_TYPE_DESTROY: &str = "destroy";
const SCRIPT_TYPE_ARTIFACTS: &str = "artifacts";
const SCRIPT_TYPE_COLLECTOR_PATH: &str = "scripts";

const PIPELINE_FILE_NAME: &str = ".gitlab-ci.yml";
const PIPELINE_TEMPLATE_FILE_NAME: &str = ".gitlab-ci.yml.tpl";

const PROPERTY_TYPE_EUT: &str = "eut";
const PROPERTY_TYPE_MODULE: &str = "module";

const VERTEX_TYPE_CI: &str = "ci";
const VERTEX_TYPE_EUT: &str = "eut";
const VERTEX_TYPE_RTE: &str = "rte";
const VERTEX_TYPE_TEST: &str = "test";
const VERTEX_TYPE_STAGE: &str = "stage";
const VERTEX_TYPE_PROJECT: &str = "project";
const VERTEX_TYPE_FEATURE: &str = "feature";
const VERTEX_TYPE_VERIFICATION: &str = "verification";
const VERTEX_TYPE_STAGE_DEPLOY: &str = "stage_deploy";
const VERTEX_TYPE_STAGE_DESTROY: &str = "stage_destroy";
const VERTEX_TYPE_STAGE_DEPLOY_ROOT: &str = "stage_deploy_root";
const VERTEX_TYPE_STAGE_DESTROY_ROOT: &str = "stage_destroy_root";

const EDGE_TYPE_HAS_CI: &str = "has_ci";
const EDGE_TYPE_HAS_EUT: &str = "has_eut";
const EDGE_TYPE_USES_RTE: &str = "uses_rte";
const EDGE_TYPE_USES_TEST: &str = "uses_test";
const EDGE_TYPE_HAS_STAGE: &str = "has_stage";
const EDGE_TYPE_NEXT_STAGE: &str = "next_stage";
const EDGE_TYPE_HAS_FEATURE: &str = "has_feature";
const EDGE_TYPE_HAS_DEPLOY_STAGE: &str = "deploy_stage";
const EDGE_TYPE_HAS_DESTROY_STAGE: &str = "destroy_stage";
const EDGE_TYPE_NEEDS_VERIFICATION: &str = "needs_verification";
const EDGE_TYPE_HAS_DEPLOY_STAGE_ROOT: &str = "has_deploy_stage_root";
const EDGE_TYPE_HAS_DESTROY_STAGE_ROOT: &str = "has_destroy_stage_root";

enum PropertyType {
    Module,
    Eut,
}

enum VertexTypes {
    Ci,
    Eut,
    Rte,
    Test,
    Feature,
    Project,
    StageDeploy,
    StageDestroy,
    Verification,
    StageDeployRoot,
    StageDestroyRoot,
}

enum EdgeTypes {
    HasCi,
    HasEut,
    UsesRte,
    UsesTest,
    HasStage,
    NextStage,
    HasFeature,
    HasDeployStage,
    HasDestroyStage,
    NeedsVerification,
    HasDeployStageRoot,
    HasDestroyStageRoot,
}

impl VertexTypes {
    fn name(&self) -> &'static str {
        match *self {
            VertexTypes::Ci => VERTEX_TYPE_TEST,
            VertexTypes::Eut => VERTEX_TYPE_EUT,
            VertexTypes::Rte => VERTEX_TYPE_RTE,
            VertexTypes::Test => VERTEX_TYPE_TEST,
            VertexTypes::Project => VERTEX_TYPE_PROJECT,
            VertexTypes::Feature => VERTEX_TYPE_FEATURE,
            VertexTypes::StageDeploy => VERTEX_TYPE_STAGE_DEPLOY,
            VertexTypes::StageDestroy => VERTEX_TYPE_STAGE_DESTROY,
            VertexTypes::Verification => VERTEX_TYPE_VERIFICATION,
            VertexTypes::StageDeployRoot => VERTEX_TYPE_STAGE_DEPLOY_ROOT,
            VertexTypes::StageDestroyRoot => VERTEX_TYPE_STAGE_DESTROY_ROOT,
        }
    }
}

impl EdgeTypes {
    fn name(&self) -> &'static str {
        match *self {
            EdgeTypes::HasCi => EDGE_TYPE_HAS_CI,
            EdgeTypes::HasEut => EDGE_TYPE_HAS_EUT,
            EdgeTypes::UsesRte => EDGE_TYPE_USES_RTE,
            EdgeTypes::UsesTest => EDGE_TYPE_USES_TEST,
            EdgeTypes::HasStage => EDGE_TYPE_HAS_STAGE,
            EdgeTypes::NextStage => EDGE_TYPE_NEXT_STAGE,
            EdgeTypes::HasFeature => EDGE_TYPE_HAS_FEATURE,
            EdgeTypes::HasDeployStage => EDGE_TYPE_HAS_DEPLOY_STAGE,
            EdgeTypes::HasDestroyStage => EDGE_TYPE_HAS_DESTROY_STAGE,
            EdgeTypes::NeedsVerification => EDGE_TYPE_NEEDS_VERIFICATION,
            EdgeTypes::HasDeployStageRoot => EDGE_TYPE_HAS_DEPLOY_STAGE_ROOT,
            EdgeTypes::HasDestroyStageRoot => EDGE_TYPE_HAS_DESTROY_STAGE_ROOT,
        }
    }
}

impl PropertyType {
    fn name(&self) -> &'static str {
        match *self {
            PropertyType::Eut => PROPERTY_TYPE_EUT,
            PropertyType::Module => PROPERTY_TYPE_MODULE,
        }
    }
    fn index(&self) -> usize {
        match *self {
            PropertyType::Eut => 0,
            PropertyType::Module => 1,
        }
    }
}

lazy_static! {
    static ref EDGE_TYPES: HashMap<VertexTuple, &'static str> = {
        let mut map = HashMap::new();
        map.insert(VertexTuple(VertexTypes::Project.name().to_string(), VertexTypes::Eut.name().to_string()), EdgeTypes::HasEut.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Feature.name().to_string()), EdgeTypes::HasFeature.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Rte.name().to_string()), EdgeTypes::UsesRte.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Test.name().to_string()), EdgeTypes::UsesTest.name());
        map.insert(VertexTuple(VertexTypes::Test.name().to_string(), VertexTypes::Verification.name().to_string()), EdgeTypes::NeedsVerification.name());
        map.insert(VertexTuple(VertexTypes::Project.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::HasCi.name());
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDeployRoot.name().to_string()), EdgeTypes::HasDeployStageRoot.name());
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDestroyRoot.name().to_string()), EdgeTypes::HasDestroyStageRoot.name());
        map.insert(VertexTuple(VertexTypes::StageDeployRoot.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::HasDeployStage.name());
        map.insert(VertexTuple(VertexTypes::StageDestroyRoot.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::HasDestroyStage.name());
        map.insert(VertexTuple(VertexTypes::StageDeploy.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::NextStage.name());
        map.insert(VertexTuple(VertexTypes::StageDestroy.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::NextStage.name());
        map
    };
    static ref EDGES_COUNT: usize = EDGE_TYPES.len();
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Regression configuration file
    #[arg(short, long)]
    config: String,
    /// Write CI pipeline file
    #[arg(short, long)]
    write: bool,
    /// Write CI pipeline file
    #[arg(short, long)]
    json: bool,
    /// Render CI pipline file
    #[arg(short, long)]
    render: bool,
    /// Debug internal data structure
    #[arg(short, long)]
    debug: bool,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct VertexTuple(String, String);

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigGenericCiStages {
    deploy: Vec<String>,
    destroy: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigGenericCi {
    stages: RegressionConfigGenericCiStages,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigCiVariables {
    name: String,
    value: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigCiArtifacts {
    path: String,
    expire_in: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigCi {
    tags: Vec<String>,
    image: String,
    artifacts: RegressionConfigCiArtifacts,
    variables: Vec<RegressionConfigCiVariables>,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigEut {
    name: String,
    path: String,
    ci: RegressionConfigGenericCi,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigCollector {
    path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigFeatures {
    path: String,
    ci: RegressionConfigGenericCi,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigRte {
    path: String,
    ci: RegressionConfigGenericCi,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigTests {
    path: String,
    ci: RegressionConfigGenericCi,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigVerifications {
    path: String,
    ci: RegressionConfigGenericCi,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigProject {
    name: String,
    templates: String,
    root_path: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RegressionConfig {
    ci: RegressionConfigCi,
    eut: RegressionConfigEut,
    rte: RegressionConfigRte,
    tests: RegressionConfigTests,
    project: RegressionConfigProject,
    features: RegressionConfigFeatures,
    collector: RegressionConfigCollector,
    verifications: RegressionConfigVerifications,
}

#[derive(Serialize, Debug)]
struct ScriptRenderContext {
    provider: String,
    rte_name: Option<String>,
    rte_names: Option<Vec<String>>,
    collector_name: Option<String>,
}

impl ScriptRenderContext {
    pub fn new(provider: String) -> Self {
        Self {
            provider,
            rte_name: None,
            rte_names: None,
            collector_name: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonData {
    data: serde_json::Map<String, Value>,
}

struct Regression {
    regression: indradb::Database<indradb::MemoryDatastore>,
    config: RegressionConfig,
}

impl Regression {
    fn new(file: &String) -> Self {
        Regression { regression: indradb::MemoryDatastore::new_db(), config: Regression::load_regression_config(&file) }
    }

    fn load_regression_config(file: &String) -> RegressionConfig {
        println!("Loading regression configuration data...");
        let data: String = String::from(file);
        let raw = std::fs::read_to_string(&data).unwrap();
        let cfg = serde_json::from_str::<RegressionConfig>(&raw).unwrap();
        println!("Loading regression configuration data -> Done.");

        println!("Render regression configuration file...");
        let mut _tera = Tera::new("../../regression/config/*").unwrap();
        let mut context = Context::new();
        context.insert("eut", &cfg.eut);
        context.insert("rte", &cfg.rte);
        context.insert("tests", &cfg.tests);
        context.insert("project", &cfg.project);
        context.insert("features", &cfg.features);
        context.insert("collector", &cfg.collector);
        context.insert("verifications", &cfg.verifications);

        let eutc = _tera.render("regression.json", &context).unwrap();
        println!("Render regression configuration file -> Done.");

        println!("Loading regression configuration data...");
        let cfg = serde_json::from_str::<RegressionConfig>(&eutc).unwrap();
        println!("Loading regression configuration data -> Done.");

        cfg
    }

    fn load_object_config(&self, _type: &VertexTypes, module: &String) -> Option<JsonData> {
        println!("Loading module <{module}> configuration data...");
        let file: String;
        match _type.name() {
            "eut" => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.eut.path, module, CONFIG_FILE_NAME);
            }
            "rte" => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.rte.path, module, CONFIG_FILE_NAME);
            }
            "feature" => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.features.path, module, CONFIG_FILE_NAME);
            }
            "test" => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.tests.path, module, CONFIG_FILE_NAME);
            }
            "verification" => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.verifications.path, module, CONFIG_FILE_NAME);
            }
            _ => {
                return None;
            }
        }

        let data: String = String::from(&file);
        let raw = std::fs::read_to_string(&data).unwrap();
        let cfg = serde_json::from_str::<JsonData>(&raw).unwrap();
        println!("Loading module <{module}> configuration data -> Done.");
        Some(cfg)
    }

    fn render_script(context: &ScriptRenderContext, input: &String) -> String {
        println!("Render regression pipeline file script section...");
        let ctx = tera::Context::from_serialize(context);
        let rendered = Tera::one_off(input, &ctx.unwrap(), true).unwrap();
        println!("Render regression pipeline file script section -> Done.");
        rendered
    }

    fn add_ci_stage(&self, start: &Vertex, value: &Value, stage_type: VertexTypes) -> Option<Vertex> {
        let new = self.create_object(stage_type);
        self.add_object_properties(&new, &value, PropertyType::Eut);
        self.create_relationship(&start, &new);
        Some(new)
    }

    fn init(&self) -> uuid::Uuid {
        // Project
        let project = self.create_object(VertexTypes::Project);
        self.add_object_properties(&project, &self.config.project, PropertyType::Eut);

        // Ci
        let ci = self.create_object(VertexTypes::Ci);
        // println!("Ci: {:?}", &ci);
        self.add_object_properties(&ci, &self.config.ci, PropertyType::Eut);
        self.create_relationship(&project, &ci);

        // Ci stages
        let stage_deploy = self.create_object(VertexTypes::StageDeployRoot);
        self.create_relationship(&ci, &stage_deploy);
        let stage_destroy = self.create_object(VertexTypes::StageDestroyRoot);
        self.create_relationship(&ci, &stage_destroy);

        // Eut
        let eut = self.create_object(VertexTypes::Eut);
        let cfg = self.load_object_config(&VertexTypes::Eut, &self.config.eut.name);
        self.add_object_properties(&eut, &cfg.unwrap().data, PropertyType::Eut);
        self.create_relationship(&project, &eut);
        let eut_p = self.get_object_properties(&eut);

        for feature in eut_p.get(PropertyType::Eut.index()).unwrap().props.get(0).unwrap().value.get("eut").unwrap().get("features").unwrap().as_array().unwrap().iter() {
            let o = self.create_object(VertexTypes::Feature);
            let cfg = self.load_object_config(&VertexTypes::Feature, &String::from(feature["module"].as_str().unwrap()));
            self.add_object_properties(&o, &cfg.unwrap().data, PropertyType::Module);
            let feature_p = self.get_object_properties(&o);
            self.create_relationship(&eut, &o);
        }

        //Stages Deploy
        let mut deploy_stage: Option<Vertex> = self.add_ci_stage(&stage_deploy, &json!(self.config.eut.ci.stages.deploy), VertexTypes::StageDeploy);
        if eut_p.get(PropertyType::Eut.index()).unwrap().props.get(0).unwrap().value.get("eut").unwrap().get("features").iter().len() > 0 {
            deploy_stage = self.add_ci_stage(&deploy_stage.unwrap(), &json!(self.config.features.ci.stages.deploy), VertexTypes::StageDeploy);
        }
        let rte_deploy_stage = self.add_ci_stage(deploy_stage.as_ref().unwrap(), &json!(&self.config.rte.ci.stages.deploy), VertexTypes::StageDeploy);

        //Stages Destroy
        let mut destroy_stage: Option<Vertex> = self.add_ci_stage(&stage_destroy, &json!(&self.config.rte.ci.stages.destroy), VertexTypes::StageDestroy);
        if eut_p.get(PropertyType::Eut.index()).unwrap().props.get(0).unwrap().value.get("eut").unwrap().get("features").iter().len() > 0 {
            destroy_stage = self.add_ci_stage(&destroy_stage.unwrap(), &json!(self.config.features.ci.stages.destroy), VertexTypes::StageDestroy);
        }
        self.add_ci_stage(&destroy_stage.unwrap(), &json!(self.config.eut.ci.stages.destroy), VertexTypes::StageDestroy);

        // Rtes
        for rte in eut_p.get(PropertyType::Eut.index()).unwrap().props.get(0).unwrap().value.get("eut").unwrap().get("rtes").unwrap().as_array().unwrap().iter() {
            let r_o = self.create_object(VertexTypes::Rte);
            self.add_object_properties(&r_o, &rte, PropertyType::Eut);
            let cfg = self.load_object_config(&VertexTypes::Rte, &String::from(rte["module"].as_str().unwrap()));
            println!("##################################################################");
            println!("RTE CFG: {:#?}", &cfg);
            println!("##################################################################");
            /*for script in &mut rte.scripts {
                        let mut ctx: ScriptRenderContext = ScriptRenderContext::new(provider.clone());
                        match script.name.as_ref() {
                            SCRIPT_TYPE_APPLY => {
                                ctx.rte_name = Option::from(rte.name.clone());
                            }
                            SCRIPT_TYPE_DESTROY => {}
                            SCRIPT_TYPE_ARTIFACTS => {
                                ctx.rte_name = Option::from(rte.name.clone());
                            }
                            _ => {
                                println!("Given script type does not match any know types")
                            }
                        }
                        script.value = render_script(&ctx, &script.value);
                    }*/
            self.add_object_properties(&r_o, &cfg.unwrap().data, PropertyType::Module);
            self.create_relationship(&eut, &r_o);

            // Tests
            for test in rte["tests"].as_array().unwrap() {
                let t_o = self.create_object(VertexTypes::Test);
                self.add_object_properties(&t_o, &test, PropertyType::Eut);
                let test_p = self.get_object_properties(&t_o);
                self.create_relationship(&r_o, &t_o);
                let name = format!("{}-{}-{}", self.config.tests.ci.stages.deploy[0], &rte["module"].as_str().unwrap().replace("_", "-"), &test_p.get(0).unwrap().props.get(0).unwrap().value["name"].as_str().unwrap());
                let test_deploy_stage = self.add_ci_stage(rte_deploy_stage.as_ref().unwrap(), &json!([name]), VertexTypes::StageDeploy);

                // Verifications
                for verification in test["verifications"].as_array().unwrap() {
                    let v_o = self.create_object(VertexTypes::Verification);
                    self.add_object_properties(&v_o, &verification, PropertyType::Eut);
                    let verification_p = self.get_object_properties(&v_o);
                    self.create_relationship(&t_o, &v_o);
                    let name = format!("{}-{}-{}", self.config.verifications.ci.stages.deploy[0], &rte["module"].as_str().unwrap().replace("_", "-"), &test_p.get(0).unwrap().props.get(0).unwrap().value["name"].as_str().unwrap());
                    self.add_ci_stage(test_deploy_stage.as_ref().unwrap(), &json!([name]), VertexTypes::StageDeploy);
                }
            }
        }

        project.id
    }

    fn create_object(&self, object_type: VertexTypes) -> Vertex {
        println!("Create new object of type <{}>...", object_type.name());
        let o = Vertex::new(indradb::Identifier::new(object_type.name()).unwrap());
        self.regression.create_vertex(&o).expect("panic while creating project db entry");
        println!("Create new object of type <{}> -> Done", object_type.name());
        o
    }

    fn create_relationship_identifier(&self, a: &Vertex, b: &Vertex) -> indradb::Identifier {
        println!("Create relationship identifier for <{}> and <{}>...", a.t.as_str(), b.t.as_str());
        let i = indradb::Identifier::new(self.get_relationship_type(&a, &b)).unwrap();
        println!("Create relationship identifier for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        i
    }

    fn create_relationship(&self, a: &Vertex, b: &Vertex) -> bool {
        println!("Create relationship for <{}> and <{}>...", a.t.as_str(), b.t.as_str());
        let i = self.create_relationship_identifier(&a, &b);
        let e = indradb::Edge::new(a.id, i, b.id);
        let status = self.regression.create_edge(&e).expect(&format!("panic build relationship between {} and {}", a.t.as_str(), b.t.as_str()));
        println!("Create relationship for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        status
    }

    fn add_object_properties<T: serde::Serialize>(&self, object: &Vertex, value: &T, property_type: PropertyType) {
        println!("Add new property to object <{}>...", object.t.to_string());
        let v = serde_json::to_value(value).unwrap();
        let mut p: BulkInsertItem;

        match property_type {
            PropertyType::Eut => {
                p = BulkInsertItem::VertexProperty(object.id, indradb::Identifier::new(PROPERTY_TYPE_EUT).unwrap(), indradb::Json::new(v.clone()));
            }
            PropertyType::Module => {
                p = BulkInsertItem::VertexProperty(object.id, indradb::Identifier::new(PROPERTY_TYPE_MODULE).unwrap(), indradb::Json::new(v.clone()));
            }
        }

        self.regression.bulk_insert(vec![p]).unwrap();
        println!("Add new property to object <{}> -> Done", object.t.to_string());
    }

    fn get_relationship_count() -> usize {
        println!("Relationship count: <{}>", *EDGES_COUNT);
        *EDGES_COUNT
    }

    fn get_relationship_type(&self, a: &Vertex, b: &Vertex) -> &str {
        println!("Get relationship type for <{}> and <{}>...", a.t.as_str(), b.t.as_str());
        let e = EDGE_TYPES.get(&VertexTuple(a.t.to_string(), b.t.to_string())).unwrap();
        println!("Get relationship type for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        e
    }

    fn get_object(&self, id: uuid::Uuid) -> Vertex {
        let q = self.regression.get(indradb::SpecificVertexQuery::single(id));
        let objs = indradb::util::extract_vertices(q.unwrap());
        let obj = objs.unwrap();
        let o = obj.get(0).unwrap();
        o.clone()
    }

    fn get_direct_neighbour_object_by_identifier(&self, object: &Vertex, identifier: VertexTypes) -> Vec<Vertex> {
        println!("Get direct neighbor of <{}>...", object.t.as_str());
        let mut rvq = RangeVertexQuery::new();
        rvq.t = Option::from(indradb::Identifier::new(identifier.name()).unwrap());
        rvq.limit = 1;
        rvq.start_id = Option::from(object.id);
        let result = indradb::util::extract_vertices(self.regression.get(rvq).unwrap()).unwrap();
        println!("Get direct neighbor of <{}> -> Done.", object.t.as_str());
        result
    }

    fn get_direct_neighbour_objects_by_identifier(&self, start: &Vertex, identifier: VertexTypes) -> Vec<Vertex> {
        println!("Get direct neighbors of <{}>...", start.t.as_str());
        let mut rvq = RangeVertexQuery::new();
        rvq.t = Option::from(indradb::Identifier::new(identifier.name()).unwrap());
        rvq.limit = 10;
        rvq.start_id = Option::from(start.id);
        let result = indradb::util::extract_vertices(self.regression.get(rvq).unwrap()).unwrap();
        println!("Get direct neighbors of <{}> -> Done.", start.t.as_str());
        result
    }

    fn get_object_properties(&self, object: &Vertex) -> Vec<VertexProperties> {
        println!("Get object <{}> properties...", object.t.as_str());
        let b = Box::new(indradb::Query::SpecificVertex(indradb::SpecificVertexQuery::single(object.id)));
        let q = indradb::PipePropertyQuery::new(b).unwrap();
        let r = self.regression.get(q).unwrap();
        println!("Get object <{}> properties -> Done.", object.t.as_str());
        indradb::util::extract_vertex_properties(r).unwrap()
    }

    fn get_relationship(&self, a: &Vertex, b: &Vertex) -> Vec<indradb::Edge> {
        println!("Get relationship for <{}> and <{}>...", &a.t.as_str(), &b.t.as_str());
        let i = self.create_relationship_identifier(&a, &b);
        let e = indradb::Edge::new(a.id, i, b.id);
        let r: Vec<indradb::QueryOutputValue> = self.regression.get(indradb::SpecificEdgeQuery::single(e.clone())).unwrap();
        let e = indradb::util::extract_edges(r).unwrap();
        println!("Get relationship for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        e
    }

    fn build_context(&self, id: uuid::Uuid) -> Context {
        println!("Build render context...");
        let project = self.get_object(id);
        let project_p = self.get_object_properties(&project);

        let eut = self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::Eut);
        let eut_p = self.get_object_properties(&eut.get(0).unwrap());

        let _features = self.get_direct_neighbour_objects_by_identifier(&eut[0], VertexTypes::Feature);
        let mut features = Vec::new();
        for feature in _features.iter() {
            let feature_p = self.get_object_properties(&feature);
            features.push(feature_p.get(0).unwrap().props.get(0).unwrap().value.clone())
        }

        let _rtes = self.get_direct_neighbour_objects_by_identifier(&eut.get(0).unwrap(), VertexTypes::Rte);
        let mut rtes = Vec::new();

        for rte in _rtes.iter() {
            let rte_p = self.get_object_properties(&rte);
            rtes.push(rte_p.get(0).unwrap().props.get(0).unwrap().value.clone());
        }
        let mut stages: Vec<String> = Vec::new();
        let mut deploy_stages: Vec<String> = Vec::new();
        let mut destroy_stages: Vec<String> = Vec::new();
        let s_deploy = self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::StageDeployRoot);
        let s_destroy = self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::StageDestroyRoot);

        for _stage in self.get_direct_neighbour_objects_by_identifier(&s_deploy[0], VertexTypes::StageDeploy).iter() {
            let _p = &self.get_object_properties(&_stage);
            let p = &_p.get(0).unwrap().props.get(0).unwrap().value;

            for stage in p.as_array().unwrap().iter() {
                deploy_stages.push(stage.as_str().unwrap().to_string());
            }
        }

        for _stage in self.get_direct_neighbour_objects_by_identifier(&s_destroy[0], VertexTypes::StageDestroy).iter() {
            let _p = &self.get_object_properties(&_stage);
            let p = &_p.get(0).unwrap().props.get(0).unwrap().value;

            for stage in p.as_array().unwrap().iter() {
                destroy_stages.push(stage.as_str().unwrap().to_string());
            }
        }

        stages.append(&mut deploy_stages);
        stages.append(&mut destroy_stages);

        let mut context = Context::new();
        context.insert("eut", &eut_p.get(0).unwrap().props[0].value["eut"]);
        context.insert("rtes", &rtes);
        context.insert("config", &self.config);
        context.insert("stages", &stages);
        context.insert("features", &features);
        context.insert("project", &project_p.get(0).unwrap().props[0].value);

        println!("{:#?}", context);
        println!("Build render context -> Done.");
        context
    }

    pub fn render(&self, context: &Context) -> String {
        println!("Render regression pipeline file first step...");
        let mut _tera = Tera::new(&self.config.project.templates).unwrap();
        let rendered = _tera.render(PIPELINE_TEMPLATE_FILE_NAME, &context).unwrap();
        println!("Render regression pipeline file first step -> Done.");
        rendered
    }

    pub fn to_json(&self) -> String {
        let j = json!({
                "config": &self.config,
            });
        j.to_string()
    }

    pub fn to_file(&self, data: &String, file: &str) {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file)
            .expect("Couldn't open file");

        f.write_all(data.as_bytes()).expect("panic while writing to file");
    }
}

fn main() {
    let cli = Cli::parse();
    let r = Regression::new(&cli.config);
    let root = r.init();
    r.to_file(&r.render(&r.build_context(root)), PIPELINE_FILE_NAME);


    /*if cli.write {
        r.to_file(&PIPELINE_FILE_NAME.to_string());
    }
    if cli.json {
        r.to_json();
        println!("{}", r.to_json());
    }*/
    /*if cli.render {
        println!("{}", r.render());
    }*/

    /*if cli.debug {
        println!("{:#?}", r);
    }*/
}
