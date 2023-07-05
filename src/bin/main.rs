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

enum PropertyType {
    Module,
    Eut,
}

enum VertexTypes {
    Ci,
    Eut,
    Rte,
    Test,
    Stage,
    Feature,
    Project,
    StageDeploy,
    StageDestroy,
    Verification,
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
}

impl VertexTypes {
    fn name(&self) -> &'static str {
        match *self {
            VertexTypes::Ci => VERTEX_TYPE_TEST,
            VertexTypes::Eut => VERTEX_TYPE_EUT,
            VertexTypes::Rte => VERTEX_TYPE_RTE,
            VertexTypes::Test => VERTEX_TYPE_TEST,
            VertexTypes::Stage => VERTEX_TYPE_STAGE,
            VertexTypes::Project => VERTEX_TYPE_PROJECT,
            VertexTypes::Feature => VERTEX_TYPE_FEATURE,
            VertexTypes::StageDeploy => VERTEX_TYPE_STAGE_DEPLOY,
            VertexTypes::StageDestroy => VERTEX_TYPE_STAGE_DESTROY,
            VertexTypes::Verification => VERTEX_TYPE_VERIFICATION,
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
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::HasDeployStage.name());
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::HasDestroyStage.name());
        map.insert(VertexTuple(VertexTypes::StageDeploy.name().to_string(), VertexTypes::Stage.name().to_string()), EdgeTypes::HasStage.name());
        map.insert(VertexTuple(VertexTypes::StageDestroy.name().to_string(), VertexTypes::Stage.name().to_string()), EdgeTypes::HasStage.name());
        map.insert(VertexTuple(VertexTypes::Stage.name().to_string(), VertexTypes::Stage.name().to_string()), EdgeTypes::NextStage.name());
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
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigCollector {
    path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigFeatures {
    path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigRte {
    path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigTests {
    path: String,
    stages: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigVerifications {
    path: String,
    stages: Vec<String>,
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

    fn add_ci_stage(&self, start: &Vertex, value: &Value) -> Option<Vertex> {
        let new = self.create_object(VertexTypes::Stage);
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
        let stage_deploy = self.create_object(VertexTypes::StageDeploy);
        self.create_relationship(&ci, &stage_deploy);
        let stage_destroy = self.create_object(VertexTypes::StageDestroy);
        self.create_relationship(&ci, &stage_destroy);

        // Eut
        let eut = self.create_object(VertexTypes::Eut);
        let cfg = self.load_object_config(&VertexTypes::Eut, &self.config.eut.name);
        self.add_object_properties(&eut, &cfg.unwrap().data, PropertyType::Eut);
        self.create_relationship(&project, &eut);
        let eut_p = self.get_object_properties(&eut);
        let mut deploy_stage: Option<Vertex> = self.add_ci_stage(&stage_deploy, &eut_p.get(0).unwrap().props.get(0).unwrap().value["eut"]["stages"]["deploy"]);
        let mut destroy_stage: Option<Vertex> = self.add_ci_stage(&stage_destroy, &eut_p.get(0).unwrap().props.get(0).unwrap().value["eut"]["stages"]["destroy"]);

        // Features
        for feature in eut_p.get(PropertyType::Eut.index()).unwrap().props.get(0).unwrap().value.get("eut").unwrap().get("features").iter() {
            let o = self.create_object(VertexTypes::Feature);
            //println!("{:?}", feature[0]);
            /*let cfg = self.load_object_config(&VertexTypes::Feature, &String::from(feature[0]["module"].as_str().unwrap()));
            self.add_object_properties(&o, &cfg, PropertyType::Module);*/
            let cfg = self.load_object_config(&VertexTypes::Feature, &String::from(feature[0]["module"].as_str().unwrap()));
            self.add_object_properties(&o, &cfg, PropertyType::Module);
            let feature_p = self.get_object_properties(&o);
            self.create_relationship(&eut, &o);
            //println!("{:?}", json!([&feature_p.get(0).unwrap().props.get(0).unwrap().value["data"]]));
            deploy_stage = self.add_ci_stage(&deploy_stage.unwrap(), &feature_p.get(0).unwrap().props.get(0).unwrap().value["data"]["stages"]["deploy"]);
            destroy_stage = self.add_ci_stage(&destroy_stage.unwrap(), &feature_p.get(0).unwrap().props.get(0).unwrap().value["data"]["stages"]["destroy"]);
        }

        // Rtes
        for rte in eut_p.get(PropertyType::Eut.index()).unwrap().props.get(0).unwrap().value.get("eut").unwrap().get("rtes").unwrap().as_array().unwrap().iter() {
            let r_o = self.create_object(VertexTypes::Rte);
            self.add_object_properties(&r_o, &rte, PropertyType::Eut);
            let cfg = self.load_object_config(&VertexTypes::Rte, &String::from(rte["module"].as_str().unwrap()));
            self.add_object_properties(&r_o, &cfg.unwrap().data, PropertyType::Module);
            let rte_p = self.get_object_properties(&r_o);
            // println!("Rte properties: {:#?}", &rte_p);
            // println!("Rte properties: {:#?}", &rte_p.get(PropertyType::Eut.index()).unwrap().props.get(PropertyType::Module.index()).unwrap().value["stages"]);
            self.create_relationship(&eut, &r_o);
            let rte_deploy_stage = self.add_ci_stage(deploy_stage.as_ref().unwrap(), &rte_p.get(PropertyType::Eut.index()).unwrap().props.get(PropertyType::Module.index()).unwrap().value["stages"]["deploy"]);
            self.add_ci_stage(&destroy_stage.as_ref().unwrap(), &rte_p.get(PropertyType::Eut.index()).unwrap().props.get(PropertyType::Module.index()).unwrap().value["stages"]["destroy"]);

            // Tests
            for test in rte["tests"].as_array().unwrap() {
                let t_o = self.create_object(VertexTypes::Test);
                //println!("Test: {:?}", &t_o);
                self.add_object_properties(&t_o, &test, PropertyType::Eut);
                let test_p = self.get_object_properties(&t_o);
                //println!("Test properties: {:?}", &test_p);
                // println!("Test properties: {:?}", &test_p.get(0).unwrap().props.get(0).unwrap().value["name"]);
                self.create_relationship(&r_o, &t_o);
                let test_deploy_stage = self.add_ci_stage(rte_deploy_stage.as_ref().unwrap(), &test_p.get(0).unwrap().props.get(0).unwrap().value["name"]);

                for verification in test["verifications"].as_array().unwrap() {
                    let v_o = self.create_object(VertexTypes::Verification);
                    //println!("Verification: {:?}", &v_o);
                    self.add_object_properties(&v_o, &verification, PropertyType::Eut);
                    let verification_p = self.get_object_properties(&v_o);
                    // println!("Verification properties: {:?}", &verification_p);
                    // println!("Verification properties: {:?}", &verification_p.get(0).unwrap().props.get(0).unwrap().value);
                    self.create_relationship(&t_o, &v_o);
                    self.add_ci_stage(test_deploy_stage.as_ref().unwrap(), &verification_p.get(0).unwrap().props.get(0).unwrap().value);
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

        let _rtes = self.get_direct_neighbour_objects_by_identifier(&eut.get(0).unwrap(), VertexTypes::Rte);
        let mut rtes = Vec::new();

        for rte in _rtes.iter() {
            let rte_p = self.get_object_properties(&rte);
            rtes.push(rte_p.get(0).unwrap().props.get(0).unwrap().value.clone());
        }

        //let mut _unique_stages: HashSet<String> = HashSet::new();
        //let mut stages = Vec::new();
        let s_deploy = self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::StageDeploy);
        for stage in self.get_direct_neighbour_objects_by_identifier(&s_deploy[0], VertexTypes::Stage).iter() {
            let _p = &self.get_object_properties(&stage);
            let p = &_p.get(0).unwrap().props.get(0).unwrap().value;
            println!("{:?} --> {:?}", &stage.t, &p);
            /*if !_unique_stages.contains(p) {
                stages.push(p.clone());
                _unique_stages.insert(stage.t.to_string());
            }*/
        }

        let mut context = Context::new();
        context.insert("config", &self.config);
        context.insert("project", &project_p.get(0).unwrap().props[0].value);
        context.insert("eut", &eut_p.get(0).unwrap().props[0].value["eut"]);
        context.insert("rtes", &rtes);
        // println!("{:#?}", context);
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
