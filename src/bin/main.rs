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
use std::collections::{HashSet, HashMap};
use std::fmt::format;
use std::future::poll_fn;
use std::string::ToString;
use clap::Parser;
use indradb::{Datastore, QueryExt, RangeVertexQuery, Vertex, VertexProperties, VertexProperty};
use serde::de::Unexpected::Option as serde_option;
use tera::Tera;
use lazy_static::lazy_static;
use std::option::Option;


const CONFIG_FILE_NAME: &str = "config.json";

const VERTEX_PROP_DATA_IDENTIFIER: &str = "data";

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
    UsesTest,
}

enum RegressionConfigInterface {
    Project(RegressionConfigProject),
    Eut(RegressionConfigEut),
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

#[derive(Hash, Eq, PartialEq, Debug)]
struct VertexTuple(String, String);

lazy_static! {
    static ref EDGES: HashMap<VertexTuple, &'static str> = {
        let mut map = HashMap::new();
        map.insert(VertexTuple(VertexTypes::Project.name().to_string(), VertexTypes::Eut.name().to_string()), EdgeTypes::HasEut.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Feature.name().to_string()), EdgeTypes::HasFeature.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Rte.name().to_string()), EdgeTypes::UsesRte.name());
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

    fn load_object_config(&self, module: &String) -> Option<Data> {
        println!("Loading module <{module}> configuration data...");

        match module.as_str() {
            "eut" => {
                let file = format!("{}/{}/{}/{}", self.config.common.root_path, self.config.eut.path, self.config.eut.name, CONFIG_FILE_NAME);
                let data: String = String::from(file);
                let raw = std::fs::read_to_string(&data).unwrap();
                let cfg = serde_json::from_str::<Data>(&raw).unwrap();
                println!("Loading module <{module}> configuration data -> Done.");
                Some(cfg)
            }
            _ => {
                None
            }
        }
    }

    fn init(&self) {
        // Project
        let project = self.create_object(VertexTypes::Project);
        println!("Project: {:?}", project);
        self.add_object_properties(&project, &self.config.project);
        let project_p = self.get_object_properties(&project);
        println!("Project properties: {:?}", project_p);

        // Eut
        let eut = self.create_object(VertexTypes::Eut);
        println!("Eut: {:?}", eut);
        let cfg = self.load_object_config(&VertexTypes::Eut.name().to_string());
        /*self.add_object_properties(&eut, &self.config.eut);
        let eut_p = self.get_object_properties(&eut);*/
        //println!("Project properties: {:?}", eut_p);

        // Features
        let feature = self.create_object(VertexTypes::Feature);
        println!("Feature: {:?}", feature);
        self.add_object_properties(&feature, &self.config.eut);
        let feature_p = self.get_object_properties(&feature);
        println!("Feature properties: {:?}", feature_p);

        // Rte
        let rte = self.create_object(VertexTypes::Rte);
        println!("Rte: {:?}", rte);
        let rte_p = self.add_object_properties(&rte, &self.config.rte);
        println!("Rte properties {:?}", &rte_p);

        // Relationships
        self.create_relationship(&project, &eut);
        self.create_relationship(&eut, &feature);


        // println!("{:?}", self.get_relationship(&project, &eut));
        // self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::Eut);

        /*indradb::Vertex::new(indradb::Identifier::new(VertexTypes::Feature.name()).unwrap());
        indradb::Vertex::new(indradb::Identifier::new(VertexTypes::Rte.name()).unwrap());
        indradb::Vertex::new(indradb::Identifier::new(VertexTypes::Test.name()).unwrap());
        indradb::Vertex::new(indradb::Identifier::new(VertexTypes::Test.name()).unwrap());*/
    }

    fn create_object(&self, object_type: VertexTypes) -> Vertex {
        println!("Create new object of type <{}>...", object_type.name());
        let o = indradb::Vertex::new(indradb::Identifier::new(object_type.name()).unwrap());
        let status = self.regression.create_vertex(&o).expect("panic while creating project db entry");
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
        let mut rvq = indradb::RangeVertexQuery::new();
        rvq.t = Option::from(indradb::Identifier::new(VertexTypes::Eut.name()).unwrap());
        rvq.limit = 1;
        rvq.start_id = Option::from(object.id);
        let result = indradb::util::extract_vertices(self.regression.get(rvq).unwrap()).unwrap();
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
