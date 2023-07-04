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
use std::collections::{HashMap};
use std::io::Write;
use std::string::ToString;
use clap::Parser;
use indradb::{RangeVertexQuery, Vertex, VertexProperties};
use tera::Tera;
use lazy_static::lazy_static;
use std::option::Option;
use serde_json::json;

const CONFIG_FILE_NAME: &str = "config.json";
const PIPELINE_FILE_NAME: &str = ".gitlab-ci.yml";
const PIPELINE_TEMPLATE_FILE_NAME: &str = ".gitlab-ci.yml.tpl";

const VERTEX_PROP_DATA_IDENTIFIER: &str = "data";

const VERTEX_TYPE_EUT: &str = "eut";
const VERTEX_TYPE_RTE: &str = "rte";
const VERTEX_TYPE_TEST: &str = "test";
const VERTEX_TYPE_PROJECT: &str = "project";
const VERTEX_TYPE_FEATURE: &str = "feature";
const VERTEX_TYPE_VERIFICATION: &str = "verification";

const EDGE_TYPE_HAS_EUT: &str = "has_eut";
const EDGE_TYPE_USES_RTE: &str = "uses_rte";
const EDGE_TYPE_USES_TEST: &str = "uses_test";
const EDGE_TYPE_HAS_FEATURE: &str = "has_feature";
const EDGE_TYPE_NEEDS_VERIFICATION: &str = "needs_verification";


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

enum VertexTypes {
    Project,
    Eut,
    Feature,
    Rte,
    Test,
    Verification,
}

enum EdgeTypes {
    HasEut,
    HasFeature,
    UsesRte,
    UsesTest,
    NeedsVerification,
}

impl VertexTypes {
    fn name(&self) -> &'static str {
        match *self {
            VertexTypes::Eut => VERTEX_TYPE_EUT,
            VertexTypes::Rte => VERTEX_TYPE_RTE,
            VertexTypes::Test => VERTEX_TYPE_TEST,
            VertexTypes::Project => VERTEX_TYPE_PROJECT,
            VertexTypes::Feature => VERTEX_TYPE_FEATURE,
            VertexTypes::Verification => VERTEX_TYPE_VERIFICATION,
        }
    }
}

impl EdgeTypes {
    fn name(&self) -> &'static str {
        match *self {
            EdgeTypes::HasEut => EDGE_TYPE_HAS_EUT,
            EdgeTypes::UsesRte => EDGE_TYPE_USES_RTE,
            EdgeTypes::UsesTest => EDGE_TYPE_USES_TEST,
            EdgeTypes::HasFeature => EDGE_TYPE_HAS_FEATURE,
            EdgeTypes::NeedsVerification => EDGE_TYPE_NEEDS_VERIFICATION,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct VertexTuple(String, String);

lazy_static! {
    static ref EDGES: HashMap<VertexTuple, &'static str> = {
        let mut map = HashMap::new();
        map.insert(VertexTuple(VertexTypes::Project.name().to_string(), VertexTypes::Eut.name().to_string()), EdgeTypes::HasEut.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Feature.name().to_string()), EdgeTypes::HasFeature.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Rte.name().to_string()), EdgeTypes::UsesRte.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Test.name().to_string()), EdgeTypes::UsesTest.name());
        map.insert(VertexTuple(VertexTypes::Test.name().to_string(), VertexTypes::Verification.name().to_string()), EdgeTypes::NeedsVerification.name());
        map
    };
    static ref EDGES_COUNT: usize = EDGES.len();
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigCommon {
    templates: String,
    root_path: String,
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
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RegressionConfig {
    ci: RegressionConfigCi,
    eut: RegressionConfigEut,
    rte: RegressionConfigRte,
    tests: RegressionConfigTests,
    common: RegressionConfigCommon,
    project: RegressionConfigProject,
    features: RegressionConfigFeatures,
    collector: RegressionConfigCollector,
    verifications: RegressionConfigVerifications,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonData {
    data: serde_json::Map<String, serde_json::Value>,
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
        let mut context = tera::Context::new();
        context.insert("eut", &cfg.eut);
        context.insert("rte", &cfg.rte);
        context.insert("common", &cfg.common);
        context.insert("collector", &cfg.collector);
        context.insert("tests", &cfg.tests);
        context.insert("verifications", &cfg.verifications);
        context.insert("project", &cfg.project);
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
                file = format!("{}/{}/{}/{}", self.config.common.root_path, self.config.eut.path, module, CONFIG_FILE_NAME);
            }
            "rte" => {
                file = format!("{}/{}/{}/{}", self.config.common.root_path, self.config.rte.path, module, CONFIG_FILE_NAME);
            }
            "feature" => {
                file = format!("{}/{}/{}/{}", self.config.common.root_path, self.config.features.path, module, CONFIG_FILE_NAME);
            }
            "test" => {
                file = format!("{}/{}/{}/{}", self.config.common.root_path, self.config.tests.path, module, CONFIG_FILE_NAME);
            }
            "verification" => {
                file = format!("{}/{}/{}/{}", self.config.common.root_path, self.config.verifications.path, module, CONFIG_FILE_NAME);
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

    fn init(&self) {
        // Project
        let project = self.create_object(VertexTypes::Project);
        println!("Project: {:?}", &project);
        self.add_object_properties(&project, &self.config.project);
        let project_p = self.get_object_properties(&project);
        println!("Project properties: {:?}", &project_p);

        // Eut
        let eut = self.create_object(VertexTypes::Eut);
        println!("Eut: {:?}", &eut);
        let cfg = self.load_object_config(&VertexTypes::Eut, &self.config.eut.name);
        self.add_object_properties(&eut, &cfg.unwrap().data);
        let eut_p = self.get_object_properties(&eut);
        println!("Eut properties: {:?}", &eut_p);

        self.create_relationship(&project, &eut);

        // Features
        for feature in eut_p.get(0).unwrap().props.get(0).unwrap().value.get("eut").unwrap().get("features").iter() {
            let o = self.create_object(VertexTypes::Feature);
            println!("Feature: {:?}", &o);
            let cfg = self.load_object_config(&VertexTypes::Feature, &String::from(feature[0]["module"].as_str().unwrap()));
            println!("{:?}", &cfg);
            self.add_object_properties(&o, &cfg);
            let feature_p = self.get_object_properties(&o);
            println!("Feature properties: {:?}", &feature_p);
            self.create_relationship(&eut, &o);
        }

        // Rtes
        for rte in eut_p.get(0).unwrap().props.get(0).unwrap().value.get("eut").unwrap().get("rtes").iter() {
            let r_o = self.create_object(VertexTypes::Rte);
            println!("Rte: {:?}", &r_o);
            let cfg = self.load_object_config(&VertexTypes::Rte, &String::from(rte[0]["module"].as_str().unwrap()));
            self.add_object_properties(&r_o, &cfg);
            let rte_p = self.get_object_properties(&r_o);
            println!("Rte properties: {:?}", &rte_p);
            self.create_relationship(&eut, &r_o);

            // Tests
            for test in rte.get(0).unwrap()["tests"].as_array().unwrap() {
                let t_o = self.create_object(VertexTypes::Test);
                println!("Test: {:?}", &t_o);
                let cfg = self.load_object_config(&VertexTypes::Test, &String::from(test["module"].as_str().unwrap()));
                self.add_object_properties(&t_o, &cfg);
                let test_p = self.get_object_properties(&t_o);
                println!("Test properties: {:?}", &test_p);
                self.create_relationship(&r_o, &t_o);

                for verification in test["verifications"].as_array().unwrap() {
                    let v_o = self.create_object(VertexTypes::Verification);
                    println!("Verification: {:?}", &v_o);
                    //let cfg = self.load_object_config(&VertexTypes::Verification, &String::from(test["module"].as_str().unwrap()));
                    self.add_object_properties(&v_o, &verification);
                    let verification_p = self.get_object_properties(&v_o);
                    println!("Test properties: {:?}", &verification_p);
                    self.create_relationship(&t_o, &v_o);
                }
            }
        }

        // println!("{:?}", self.get_relationship(&project, &eut));
        // self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::Eut);
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

    fn add_object_properties<T: serde::Serialize>(&self, object: &Vertex, value: &T) {
        println!("Add new property to object <{}>...", object.t.to_string());
        let v = serde_json::to_value(value).unwrap();
        let p = indradb::BulkInsertItem::VertexProperty(object.id, indradb::Identifier::new(VERTEX_PROP_DATA_IDENTIFIER).unwrap(), indradb::Json::new(v.clone()));
        self.regression.bulk_insert(vec![p]).unwrap();
        println!("Add new property to object <{}> -> Done", object.t.to_string());
    }

    fn get_relationship_count() -> usize {
        println!("Relationship count: <{}>", *EDGES_COUNT);
        *EDGES_COUNT
    }

    fn get_relationship_type(&self, a: &Vertex, b: &Vertex) -> &str {
        println!("Get relationship type for <{}> and <{}>...", a.t.as_str(), b.t.as_str());
        let e = EDGES.get(&VertexTuple(a.t.to_string(), b.t.to_string())).unwrap();
        println!("Get relationship type for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        e
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

    pub fn render(&self) -> String {
        println!("Render regression pipeline file first step...");
        let mut _tera = Tera::new(&self.config.common.templates).unwrap();
        let mut context = tera::Context::new();
        context.insert("config", &self.config);
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

    pub fn to_file(&self, file: &String) {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file)
            .expect("Couldn't open file");

        f.write_all(&self.render().as_bytes()).expect("panic while writing to file");
    }
}

fn main() {
    let cli = Cli::parse();
    let r = Regression::new(&cli.config);

    /*if cli.write {
        e.to_file(String::from(".gitlab-ci.yml"));
    }
    if cli.json {
        e.to_json();
        println!("{}", e.to_json());
    }
    if cli.render {
        println!("{}", e.render());
    }

    if cli.debug {
        println!("{:#?}", e);
    }*/


    r.init();
    r.to_file(&PIPELINE_FILE_NAME.to_string());

    /*let o = ObjectEut { name: "eutA".to_string() };
    o.get_path(String::from("rte"));*/
}
