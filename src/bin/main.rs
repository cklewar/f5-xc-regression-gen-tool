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

use std::sync::Arc;
use indradb;
use indradb::{Datastore, ValidationResult};
use serde_json::json;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::collections::{HashSet, HashMap};
use std::string::ToString;
use clap::Parser;
use tera::Tera;
use lazy_static::lazy_static;

const CONFIG_FILE_NAME: &str = "config.json";

const VERTEX_TYPE_EUT: &str = "eut";
const VERTEX_TYPE_RTE: &str = "project";
const VERTEX_TYPE_PROJECT: &str = "project";
const VERTEX_TYPE_FEATURE: &str = "feature";
const VERTEX_TYPE_TEST: &str = "test";
const VERTEX_TYPE_VALIDATION: &str = "validation";

const EDGE_TYPE_HAS_EUT: &str = "has_eut";
const EDGE_TYPE_HAS_FEATURE: &str = "has_feature";
const EDGE_TYPE_USES_RTE: &str = "uses_rte";
const EDGE_TYPE_USES_TEST: &str = "uses_test";


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
    Validation,
}

enum EdgeTypes {
    HasEut,
    HasFeature,
    UsesRte,
    USesTest,
}

impl VertexTypes {
    fn name(&self) -> &'static str {
        match *self {
            VertexTypes::Project => VERTEX_TYPE_RTE,
            VertexTypes::Feature => VERTEX_TYPE_FEATURE,
            VertexTypes::Eut => VERTEX_TYPE_EUT,
            VertexTypes::Rte => VERTEX_TYPE_RTE,
            VertexTypes::Test => VERTEX_TYPE_TEST,
            VertexTypes::Validation => VERTEX_TYPE_VALIDATION,
        }
    }
}

impl EdgeTypes {
    fn name(&self) -> &'static str {
        match *self {
            EdgeTypes::HasEut => EDGE_TYPE_HAS_EUT,
            EdgeTypes::HasFeature => EDGE_TYPE_HAS_FEATURE,
            EdgeTypes::UsesRte => EDGE_TYPE_USES_RTE,
            EdgeTypes::UsesTest => EDGE_TYPE_USES_TEST,
        }
    }
}

#[derive(Hash, Eq)]
struct TupleStruct(String, String);

//static EDGES: (String, String, String) = (VertexTypes::Eut.name().to_string(), "B".to_string(), "C".to_string());

lazy_static! {
    static ref EDGED: HashMap<TupleStruct, &'static str> = {
        let mut map = HashMap::new();
        map.insert(TupleStruct(VertexTypes::Project.name().to_string(), VertexTypes::Eut.name().to_string()), EdgeTypes::HasEut.name().to_string());
        map
    };
}
//static HM: HashMap<TupleStruct, String> = (TupleStruct(VertexTypes::Project.name().to_string(), VertexTypes::Eut.name().to_string()), EdgeTypes::HasEut.name().to_string());


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
    collector: RegressionConfigCollector,
    tests: RegressionConfigTests,
    common: RegressionConfigCommon,
    project: RegressionConfigProject,
    verifications: RegressionConfigVerifications,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
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

    fn init(&self) {
        let v_p = indradb::Vertex::new(indradb::Identifier::new(VertexTypes::Project.name()).unwrap());
        let v_eut = indradb::Vertex::new(indradb::Identifier::new(VertexTypes::Eut.name()).unwrap());
        self.regression.create_vertex(&v_p).expect("panic while creating project db entry");
        self.regression.create_vertex(&v_eut).expect("panic while creating eut db entry");

        // Project
        let v = serde_json::to_value(&self.config.project).unwrap();
        let bi = indradb::BulkInsertItem::VertexProperty(v_eut.id, indradb::Identifier::new("data").unwrap(), indradb::Json::new(v.clone()));
        self.regression.bulk_insert(vec![bi]).unwrap();

        // Eut
        let file = format!("{}/{}/{}/{}", self.config.common.root_path, self.config.eut.path, self.config.eut.name, CONFIG_FILE_NAME);
        let raw = std::fs::read_to_string(file).expect("panic while opening regression config file");
        let cfg: Data = serde_json::from_str::<Data>(&raw).unwrap();
        let v = serde_json::to_value(cfg).unwrap();
        let bi = indradb::BulkInsertItem::VertexProperty(v_eut.id, indradb::Identifier::new("data").unwrap(), indradb::Json::new(v.get("data").unwrap().get("eut").unwrap().clone()));
        self.regression.bulk_insert(vec![bi]).unwrap();

        /*indradb::Vertex::new(indradb::Identifier::new(VertexTypes::Feature.name()).unwrap());
        indradb::Vertex::new(indradb::Identifier::new(VertexTypes::Rte.name()).unwrap());
        indradb::Vertex::new(indradb::Identifier::new(VertexTypes::Test.name()).unwrap());
        indradb::Vertex::new(indradb::Identifier::new(VertexTypes::Test.name()).unwrap());*/
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
    /*let db: indradb::Database<indradb::MemoryDatastore> = indradb::MemoryDatastore::new_db();
    let v_project = indradb::Vertex::new(indradb::Identifier::new(VERTEX_TYPE_PROJECT).unwrap());
    let v_eut = indradb::Vertex::new(indradb::Identifier::new(VertexTypeEut).unwrap());
    let v_rte = indradb::Vertex::new(indradb::Identifier::new(VERTEX_TYPE_RTE).unwrap());
    let root_path = String::from("/home/cklewar/Projects/gitlab.com/F5/volterra/solution/sense8");
    let file = format!("{}/{}/{}/{}", root_path, "eut", "mcn", CONFIG_FILE_NAME);
    let raw = std::fs::read_to_string(file).expect("panic while opening rte config file");
    let mut cfg = serde_json::from_str::<Data>(&raw).unwrap();
    db.create_vertex(&v_project).expect("TODO: panic message");
    let v = serde_json::to_value(cfg).unwrap();
    println!("{:?}", &v.get("eut").unwrap());
    let p = v.get("eut").unwrap();
    let bi = indradb::BulkInsertItem::VertexProperty(v_project.id, indradb::Identifier::new("data").unwrap(), indradb::Json::new(p.clone()));
    let mut data = Vec::new();
    data.push(bi);
    db.bulk_insert(data).unwrap();
    let b = Box::new(indradb::Query::SpecificVertex(indradb::SpecificVertexQuery::single(v_project.id)));
    let q = indradb::PipePropertyQuery::new(b).unwrap();
    let props: Vec<indradb::QueryOutputValue> = db.get(q).unwrap();
    let e = indradb::util::extract_vertex_properties(props).unwrap();
    println!("V_PROP_AFTER_INSERT: {:?}", e[0].props[0]);*/

    /*println!("{:?}", e[0].props[0].value.as_object());
    println!("{:?}", e[0].props[0].value.get("age"));*/

    //serde_json::from_value(e[0].props[0].value).unwrap();
    //let p: Person = serde_json::from_str().unwrap();
    //println!("Please call {} at the number {}", p.name, p.phones[0]);

    //let vp = indradb::VertexProperty::new(v_project.id, indradb::Json::new(john));
    /*db.create_vertex(&v_eut).expect("TODO: panic message");
    db.create_vertex(&v_rte).expect("TODO: panic message");
    let edge = indradb::Edge::new(v_project.id, indradb::Identifier::new(EDGE_TYPE_HAS).unwrap(), v_eut.id);
    db.create_edge(&edge).expect("TODO: panic message");
    let output: Vec<indradb::QueryOutputValue> = db.get(indradb::SpecificEdgeQuery::single(edge.clone())).unwrap();
    println!("{:?}", &output);
    let e = indradb::util::extract_edges(output).unwrap();
    println!("{:?}", &e);
    let edge = indradb::Edge::new(v_eut.id, indradb::Identifier::new(EDGE_TYPE_USES).unwrap(), v_rte.id);
    db.create_edge(&edge).expect("TODO: panic message");
    let output: Vec<indradb::QueryOutputValue> = db.get(indradb::SpecificEdgeQuery::single(edge.clone())).unwrap();
    println!("{:?}", &output);
    let e = indradb::util::extract_edges(output).unwrap();
    println!("{:?}", &e);
    //assert_eq!(e.len(), 1);
    //assert_eq!(edge, e[0]);
    println!("{:?}", v_project);
    let vertex: Vec<indradb::QueryOutputValue> = db.get(indradb::SpecificVertexQuery::single(v_project.id)).unwrap();
    println!("{:?}", vertex);
    let b = Box::new(indradb::Query::SpecificVertex(indradb::SpecificVertexQuery::single(v_project.id)));
    let q = indradb::PipePropertyQuery::new(b).unwrap();
    println!("{:?}", q.inner);*/
}
