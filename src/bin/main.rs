/*!
F5 XC regression test CI pipeline file generator.
Provides command line tool to generate Gitlab CI pipeline file.

Consumes input from regression configuration file provided as command line argument.
Template file relays on tool provided data structure to render stage, job or variables sections.
Tool supports direct rendering of given template file or generates JSON output which could be
used as input for another program or workflow.

Supported command line arguments:
--config <provide regression environment configuration file>
 */

use std::collections::HashMap;
use std::io::Write;
use std::option::Option;
use std::string::ToString;
use std::vec;

use clap::error::ErrorKind::Format as clap_format;
use clap::Parser;
use graphviz_rust::attributes::defaultdist;
use indradb;
use indradb::{AllVertexQuery, BulkInsertItem, Edge, EdgeDirection, Identifier, Json, PipeQuery, Query, QueryExt, RangeVertexQuery, SpecificEdgeQuery, Vertex, VertexProperties, VertexWithPropertyValueQuery};
use indradb::Query::VertexWithPropertyValue;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use serde_json::Value::Null;
use tera::{Context, Tera};
use uuid::{Uuid, uuid};

//use clap::builder::Resettable::Value;

const CONFIG_FILE_NAME: &str = "config.json";

// const SCRIPT_TYPE_APPLY: &str = "apply";
// const SCRIPT_TYPE_DESTROY: &str = "destroy";
// const SCRIPT_TYPE_ARTIFACTS: &str = "artifacts";
// const SCRIPT_TYPE_COLLECTOR_PATH: &str = "scripts";

// KEYS
const CI: &str = "ci";
const EUT: &str = "eut";
const RTE: &str = "rte";
const RTES: &str = "rtes";
const TEST: &str = "test";
const GVID: &str = "id";
const GV_LABEL: &str = "label";
const NAME: &str = "name";
const MODULE: &str = "module";
const SCRIPTS: &str = "scripts";
const SOURCES: &str = "sources";
const PROJECT: &str = "project";
const FEATURE: &str = "feature";
const FEATURES: &str = "features";
const PROVIDER: &str = "provider";
const PROVIDERS: &str = "providers";
const COMPONENTS: &str = "components";
const CONNECTIONS: &str = "connections";
const VERIFICATION: &str = "verification";

const PIPELINE_FILE_NAME: &str = ".gitlab-ci.yml";
const PIPELINE_TEMPLATE_FILE_NAME: &str = ".gitlab-ci.yml.tpl";

const PROPERTY_TYPE_GV: &str = "gv";
const PROPERTY_TYPE_BASE: &str = "base";
const PROPERTY_TYPE_MODULE: &str = "module";

//Objects types
const VERTEX_TYPE_CI: &str = "ci";
const VERTEX_TYPE_EUT: &str = "eut";
const VERTEX_TYPE_RTE: &str = "rte";
const VERTEX_TYPE_RTES: &str = "rtes";
const VERTEX_TYPE_TEST: &str = "test";
const VERTEX_TYPE_NONE: &str = "none";
const VERTEX_TYPE_SCRIPT: &str = "script";
const VERTEX_TYPE_SCRIPTS: &str = "scripts";
const VERTEX_TYPE_PROJECT: &str = "project";
const VERTEX_TYPE_FEATURE: &str = "feature";
const VERTEX_TYPE_FEATURES: &str = "features";
const VERTEX_TYPE_EUT_PROVIDER: &str = "eut_provider";
const VERTEX_TYPE_RTE_PROVIDER: &str = "rte_provider";
const VERTEX_TYPE_PROVIDERS: &str = "providers";
const VERTEX_TYPE_CONNECTION: &str = "connection";
const VERTEX_TYPE_CONNECTIONS: &str = "connections";
const VERTEX_TYPE_COLLECTOR: &str = "collector";
const VERTEX_TYPE_COMPONENTS: &str = "components";
const VERTEX_TYPE_CONNECTION_SRC: &str = "connection_src";
const VERTEX_TYPE_CONNECTION_DST: &str = "connection_dst";
const VERTEX_TYPE_VERIFICATION: &str = "verification";
const VERTEX_TYPE_STAGE_DEPLOY: &str = "deploy";
const VERTEX_TYPE_STAGE_DESTROY: &str = "stage_destroy";
const VERTEX_TYPE_COMPONENT_SRC: &str = "component_src";
const VERTEX_TYPE_COMPONENT_DST: &str = "component_dst";

// Rel type
const EDGE_TYPE_HAS: &str = "has";
const EDGE_TYPE_USES: &str = "uses";
const EDGE_TYPE_RUNS: &str = "runs";
const EDGE_TYPE_NEXT: &str = "next";
const EDGE_TYPE_NEEDS: &str = "needs";
const EDGE_TYPE_PROVIDES: &str = "provides";

enum PropertyType {
    Gv,
    Base,
    Module,
}

#[derive(Clone, PartialEq, Debug)]
enum VertexTypes {
    Ci,
    Eut,
    Rte,
    Rtes,
    Test,
    Script,
    Project,
    Scripts,
    Feature,
    Features,
    Providers,
    Collector,
    Components,
    Connection,
    Connections,
    RteProvider,
    EutProvider,
    StageDeploy,
    StageDestroy,
    Verification,
    ComponentSrc,
    ComponentDst,
    ConnectionSrc,
    ConnectionDst,
    None,
}

enum EdgeTypes {
    Has,
    Uses,
    Runs,
    Next,
    Needs,
    Provides,
}

impl VertexTypes {
    fn name(&self) -> &'static str {
        match *self {
            VertexTypes::Ci => VERTEX_TYPE_CI,
            VertexTypes::Eut => VERTEX_TYPE_EUT,
            VertexTypes::Rte => VERTEX_TYPE_RTE,
            VertexTypes::Rtes => VERTEX_TYPE_RTES,
            VertexTypes::Test => VERTEX_TYPE_TEST,
            VertexTypes::Script => VERTEX_TYPE_SCRIPT,
            VertexTypes::Scripts => VERTEX_TYPE_SCRIPTS,
            VertexTypes::Project => VERTEX_TYPE_PROJECT,
            VertexTypes::Feature => VERTEX_TYPE_FEATURE,
            VertexTypes::Features => VERTEX_TYPE_FEATURES,
            VertexTypes::EutProvider => VERTEX_TYPE_EUT_PROVIDER,
            VertexTypes::RteProvider => VERTEX_TYPE_RTE_PROVIDER,
            VertexTypes::Providers => VERTEX_TYPE_PROVIDERS,
            VertexTypes::Connection => VERTEX_TYPE_CONNECTION,
            VertexTypes::Connections => VERTEX_TYPE_CONNECTIONS,
            VertexTypes::Collector => VERTEX_TYPE_COLLECTOR,
            VertexTypes::Components => VERTEX_TYPE_COMPONENTS,
            VertexTypes::ConnectionSrc => VERTEX_TYPE_CONNECTION_SRC,
            VertexTypes::ConnectionDst => VERTEX_TYPE_CONNECTION_DST,
            VertexTypes::StageDeploy => VERTEX_TYPE_STAGE_DEPLOY,
            VertexTypes::StageDestroy => VERTEX_TYPE_STAGE_DESTROY,
            VertexTypes::Verification => VERTEX_TYPE_VERIFICATION,
            VertexTypes::ComponentSrc => VERTEX_TYPE_COMPONENT_SRC,
            VertexTypes::ComponentDst => VERTEX_TYPE_COMPONENT_DST,
            VertexTypes::None => VERTEX_TYPE_NONE,
        }
    }

    fn get_name_by_object(object: &Vertex) -> &'static str {
        match object.t.as_str() {
            VERTEX_TYPE_CI => VertexTypes::Ci.name(),
            VERTEX_TYPE_RTE => VertexTypes::Rte.name(),
            VERTEX_TYPE_EUT => VertexTypes::Eut.name(),
            VERTEX_TYPE_RTES => VertexTypes::Rtes.name(),
            VERTEX_TYPE_TEST => VertexTypes::Test.name(),
            VERTEX_TYPE_SCRIPT => VertexTypes::Script.name(),
            VERTEX_TYPE_SCRIPTS => VertexTypes::Scripts.name(),
            VERTEX_TYPE_PROJECT => VertexTypes::Project.name(),
            VERTEX_TYPE_FEATURE => VertexTypes::Feature.name(),
            VERTEX_TYPE_EUT_PROVIDER => VertexTypes::EutProvider.name(),
            VERTEX_TYPE_RTE_PROVIDER => VertexTypes::RteProvider.name(),
            VERTEX_TYPE_PROVIDERS => VertexTypes::Providers.name(),
            VERTEX_TYPE_COLLECTOR => VertexTypes::Collector.name(),
            VERTEX_TYPE_CONNECTION => VertexTypes::Connection.name(),
            VERTEX_TYPE_CONNECTIONS => VertexTypes::Connections.name(),
            VERTEX_TYPE_COMPONENTS => VertexTypes::Components.name(),
            VERTEX_TYPE_VERIFICATION => VertexTypes::Verification.name(),
            VERTEX_TYPE_STAGE_DEPLOY => VertexTypes::StageDeploy.name(),
            VERTEX_TYPE_STAGE_DESTROY => VertexTypes::StageDestroy.name(),
            VERTEX_TYPE_COMPONENT_SRC => VertexTypes::ComponentSrc.name(),
            VERTEX_TYPE_COMPONENT_DST => VertexTypes::ComponentDst.name(),
            VERTEX_TYPE_CONNECTION_SRC => VertexTypes::ConnectionSrc.name(),
            VERTEX_TYPE_CONNECTION_DST => VertexTypes::ConnectionDst.name(),
            _ => "None"
        }
    }

    fn get_type_by_key(key: &str) -> VertexTypes {
        match key {
            VERTEX_TYPE_CI => VertexTypes::Ci,
            VERTEX_TYPE_RTE => VertexTypes::Rte,
            VERTEX_TYPE_EUT => VertexTypes::Eut,
            VERTEX_TYPE_RTES => VertexTypes::Rtes,
            VERTEX_TYPE_TEST => VertexTypes::Test,
            VERTEX_TYPE_SCRIPT => VertexTypes::Script,
            VERTEX_TYPE_SCRIPTS => VertexTypes::Scripts,
            VERTEX_TYPE_PROJECT => VertexTypes::Project,
            VERTEX_TYPE_FEATURE => VertexTypes::Feature,
            VERTEX_TYPE_FEATURES => VertexTypes::Features,
            VERTEX_TYPE_EUT_PROVIDER => VertexTypes::EutProvider,
            VERTEX_TYPE_RTE_PROVIDER => VertexTypes::RteProvider,
            VERTEX_TYPE_PROVIDERS => VertexTypes::Providers,
            VERTEX_TYPE_CONNECTION => VertexTypes::Connection,
            VERTEX_TYPE_CONNECTIONS => VertexTypes::Connections,
            VERTEX_TYPE_COMPONENTS => VertexTypes::Components,
            VERTEX_TYPE_COLLECTOR => VertexTypes::Collector,
            VERTEX_TYPE_CONNECTION_SRC => VertexTypes::ConnectionSrc,
            VERTEX_TYPE_CONNECTION_DST => VertexTypes::ConnectionDst,
            VERTEX_TYPE_VERIFICATION => VertexTypes::Verification,
            VERTEX_TYPE_STAGE_DEPLOY => VertexTypes::StageDeploy,
            VERTEX_TYPE_STAGE_DESTROY => VertexTypes::StageDestroy,
            VERTEX_TYPE_COMPONENT_SRC => VertexTypes::ComponentSrc,
            VERTEX_TYPE_COMPONENT_DST => VertexTypes::ComponentDst,
            _ => VertexTypes::None
        }
    }
}

impl EdgeTypes {
    fn name(&self) -> &'static str {
        match *self {
            EdgeTypes::Has => EDGE_TYPE_HAS,
            EdgeTypes::Uses => EDGE_TYPE_USES,
            EdgeTypes::Runs => EDGE_TYPE_RUNS,
            EdgeTypes::Next => EDGE_TYPE_NEXT,
            EdgeTypes::Needs => EDGE_TYPE_NEEDS,
            EdgeTypes::Provides => EDGE_TYPE_PROVIDES,
        }
    }
}

impl PropertyType {
    fn name(&self) -> &'static str {
        match *self {
            PropertyType::Gv => PROPERTY_TYPE_GV,
            PropertyType::Base => PROPERTY_TYPE_BASE,
            PropertyType::Module => PROPERTY_TYPE_MODULE,
        }
    }

    fn index(&self) -> usize {
        match *self {
            PropertyType::Gv => 1,
            PropertyType::Base => 0,
            PropertyType::Module => 2,
        }
    }
}

lazy_static! {
    static ref EDGE_TYPES: HashMap<VertexTuple, &'static str> = {
        let mut map = HashMap::new();
        map.insert(VertexTuple(VertexTypes::Project.name().to_string(), VertexTypes::Eut.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Project.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Features.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Rtes.name().to_string()), EdgeTypes::Uses.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Providers.name().to_string()), EdgeTypes::Uses.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Scripts.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Scripts.name().to_string(), VertexTypes::Script.name().to_string()), EdgeTypes::Uses.name());
        map.insert(VertexTuple(VertexTypes::Features.name().to_string(), VertexTypes::Feature.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Rtes.name().to_string(), VertexTypes::Rte.name().to_string()), EdgeTypes::Uses.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Providers.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Connections.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Collector.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Scripts.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Features.name().to_string()), EdgeTypes::Needs.name());
        map.insert(VertexTuple(VertexTypes::Providers.name().to_string(), VertexTypes::EutProvider.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Providers.name().to_string(), VertexTypes::RteProvider.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::EutProvider.name().to_string(), VertexTypes::Components.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::EutProvider.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::RteProvider.name().to_string(), VertexTypes::Components.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::RteProvider.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Components.name().to_string(), VertexTypes::ComponentSrc.name().to_string()), EdgeTypes::Provides.name());
        map.insert(VertexTuple(VertexTypes::Components.name().to_string(), VertexTypes::ComponentDst.name().to_string()), EdgeTypes::Provides.name());
        map.insert(VertexTuple(VertexTypes::Connections.name().to_string(), VertexTypes::Connection.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Connection.name().to_string(), VertexTypes::ConnectionSrc.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::ConnectionDst.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::Test.name().to_string()), EdgeTypes::Runs.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::ComponentSrc.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::ConnectionDst.name().to_string(), VertexTypes::ComponentDst.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Test.name().to_string(), VertexTypes::Verification.name().to_string()), EdgeTypes::Needs.name());
        map.insert(VertexTuple(VertexTypes::Test.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::StageDeploy.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::Next.name());
        map.insert(VertexTuple(VertexTypes::StageDestroy.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::Next.name());
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
    module: String,
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
struct EutRenderContext {
    base: Map<String, Value>,
    module: Map<String, Value>,
    ci: Map<String, Value>,
    provider: Vec<String>,
}

#[derive(Serialize, Debug)]
struct RteConnectionRenderContext {
    job: String,
    rte: String,
    component: String,
    provider: String,
}

#[derive(Serialize, Debug)]
struct RteProviderCiRenderContext {
    timeout: Value,
    variables: Value,
}

#[derive(Serialize, Debug)]
struct RteRenderContext {
    ci: HashMap<String, RteProviderCiRenderContext>,
    connections: Vec<RteConnectionRenderContext>,
}

/*
ci: Map<String, Value>,
scripts: Vec<Map<String, Value>>,
*/

#[derive(Serialize, Debug)]
struct ScriptRenderContext {
    provider: String,
    rte_name: Option<String>,
    rte_names: Option<Vec<String>>,
    collector_name: Option<String>,
}

/*impl ScriptRenderContext {
    pub fn new(provider: String) -> Self {
        Self {
            provider,
            rte_name: None,
            rte_names: None,
            collector_name: None,
        }
    }
}*/

struct Regression {
    regression: indradb::Database<indradb::MemoryDatastore>,
    config: RegressionConfig,
}

impl Regression {
    fn new(file: &str) -> Self {
        Regression { regression: indradb::MemoryDatastore::new_db(), config: Regression::load_regression_config(&file) }
    }

    fn load_regression_config(file: &str) -> RegressionConfig {
        info!("Loading regression configuration data...");
        let data: String = String::from(file);
        let raw = std::fs::read_to_string(&data).unwrap();
        let cfg = serde_json::from_str::<RegressionConfig>(&raw).unwrap();
        info!("Loading regression configuration data -> Done.");

        info!("Render regression configuration file...");
        let mut _tera = Tera::new("../../regression/config/*").unwrap();
        let mut context = Context::new();
        context.insert(EUT, &cfg.eut);
        context.insert(RTE, &cfg.rte);
        context.insert("tests", &cfg.tests);
        context.insert("project", &cfg.project);
        context.insert(FEATURES, &cfg.features);
        context.insert("collector", &cfg.collector);
        context.insert("verifications", &cfg.verifications);

        let eutc = _tera.render("regression.json", &context).unwrap();
        info!("Render regression configuration file -> Done.");

        info!("Loading regression configuration data...");
        let cfg = serde_json::from_str::<RegressionConfig>(&eutc).unwrap();
        info!("Loading regression configuration data -> Done.");

        cfg
    }

    fn load_object_config(&self, _type: &str, module: &str) -> Value {
        info!("Loading module <{module}> configuration data...");
        let file: String;
        match _type {
            EUT => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.eut.path, module, CONFIG_FILE_NAME);
            }
            RTE => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.rte.path, module, CONFIG_FILE_NAME);
            }
            FEATURE => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.features.path, module, CONFIG_FILE_NAME);
            }
            TEST => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.tests.path, module, CONFIG_FILE_NAME);
            }
            VERIFICATION => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.verifications.path, module, CONFIG_FILE_NAME);
            }
            _ => {
                return Null;
            }
        }

        let data: String = String::from(&file);
        let raw = std::fs::read_to_string(&data).unwrap();
        let cfg: Value = serde_json::from_str(&raw).unwrap();
        info!("Loading module <{module}> configuration data -> Done.");
        cfg
    }

    #[allow(dead_code)]
    fn render_script(context: &ScriptRenderContext, input: &str) -> String {
        info!("Render regression pipeline file script section...");
        let ctx = Context::from_serialize(context);
        let rendered = Tera::one_off(input, &ctx.unwrap(), true).unwrap();
        info!("Render regression pipeline file script section -> Done.");
        rendered
    }

    fn add_ci_stages(&self, ancestor: &Vertex, stages: &Vec<String>, object_type: &VertexTypes) -> Option<Vertex> {
        let mut curr = Vertex { id: Default::default(), t: Default::default() };

        for (i, stage) in stages.iter().enumerate() {
            let new = self.create_object(object_type.clone());
            self.add_object_properties(&new, &stage, PropertyType::Base);
            self.add_object_properties(&new, &json!({
                GVID: stage.replace("-", "_"),
                GV_LABEL: stage,
            }), PropertyType::Gv);

            if i == 0 {
                self.create_relationship(&ancestor, &new);
                curr = new.clone();
            } else {
                self.create_relationship(&curr, &new);
                curr = new.clone();
            }
        }
        Some(curr)
    }

    fn init(&self) -> Uuid {
        // self.regression.index_property(indradb::Identifier::new(PropertyType::Base.name()).unwrap()).expect("error adding index for rte key");
        // Project
        let project = self.create_object(VertexTypes::Project);
        self.add_object_properties(&project, &self.config.project, PropertyType::Base);
        self.add_object_properties(&project, &json!({
            GVID: self.config.project.name.replace("-", "_"),
            GV_LABEL: self.config.project.name.replace("-", "_"),
        }), PropertyType::Gv);

        // Ci
        let ci = self.create_object(VertexTypes::Ci);
        self.add_object_properties(&ci, &self.config.ci, PropertyType::Base);
        self.add_object_properties(&ci, &json!({
            GVID: format!("{}_{}", self.config.project.name.replace("-", "_"), CI),
            GV_LABEL: CI,
        }), PropertyType::Gv);
        self.create_relationship(&project, &ci);

        // Eut
        let eut = self.create_object(VertexTypes::Eut);
        self.add_object_properties(&eut, &self.config.eut, PropertyType::Base);
        self.add_object_properties(&eut, &json!({
            GVID: self.config.eut.module.replace("-", "_"),
            GV_LABEL: self.config.eut.module,
        }), PropertyType::Gv);
        let module = self.load_object_config(&VertexTypes::get_name_by_object(&eut), &self.config.eut.module);
        let v = serde_json::to_value(module).unwrap();
        self.create_relationship(&project, &eut);

        let eut_providers = self.create_object(VertexTypes::Providers);
        self.create_relationship(&eut, &eut_providers);

        for (k, v) in v.as_object().unwrap().iter() {
            match k {
                k if k == NAME => {
                    let eut_o_p = self.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.clone(), v.clone());
                    self.add_object_properties(&eut, &p, PropertyType::Module);
                    self.add_object_properties(&eut_providers, &json!({
                        GVID: format!("{}_{}_{}", eut.t.as_str(), PROVIDERS, &v.as_str().unwrap()),
                        GV_LABEL: PROVIDERS
                    }), PropertyType::Gv);
                }
                k if k == "release" => {
                    let eut_o_p = self.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.clone(), v.clone());
                    self.add_object_properties(&eut, &p, PropertyType::Module);
                }
                k if k == PROVIDER => {
                    for p in v.as_array().unwrap().iter() {
                        let p_o = self.create_object(VertexTypes::EutProvider);
                        self.create_relationship(&eut_providers, &p_o);
                        self.add_object_properties(&p_o, &json!({NAME: &p.as_str().unwrap()}), PropertyType::Base);
                        self.add_object_properties(&p_o, &json!({
                            GVID: format!("{}_{}_{}", eut.t.as_str(), PROVIDER, &p.as_str().unwrap()),
                            GV_LABEL: &p.as_str().unwrap()
                        }), PropertyType::Gv);
                    }
                }
                k if k == CI => {
                    let o = self.create_object(VertexTypes::get_type_by_key(k));
                    self.create_relationship(&eut, &o);
                    self.add_object_properties(&o, &v, PropertyType::Module);
                    self.add_object_properties(&o, &json!({
                            GVID: CI,
                            GV_LABEL: CI,
                        }), PropertyType::Gv);
                }
                k if k == FEATURES => {
                    let o = self.create_object(VertexTypes::get_type_by_key(k));
                    self.create_relationship(&eut, &o);
                    for f in v.as_array().unwrap().iter() {
                        for (k, v) in f.as_object().unwrap().iter() {
                            let f_o = self.create_object(VertexTypes::get_type_by_key(k));
                            self.add_object_properties(&f_o, &json!({NAME: &v.as_str().unwrap()}), PropertyType::Base);
                            self.add_object_properties(&f_o, &json!({
                                    GVID: format!("{}_{}", &k, &v.as_str().unwrap()),
                                    GV_LABEL: &v.as_str().unwrap()
                                }), PropertyType::Gv);
                            self.create_relationship(&o, &f_o);
                        }
                    }
                }
                k if k == "scripts_path" => {
                    let eut_o_p = self.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.clone(), v.clone());
                    self.add_object_properties(&eut, &p, PropertyType::Module);
                }
                k if k == SCRIPTS => {
                    let o = self.create_object(VertexTypes::get_type_by_key(k));
                    self.create_relationship(&eut, &o);

                    for s in v.as_array().unwrap().iter() {
                        for (k, v) in s.as_object().unwrap().iter() {
                            let mut f_o = Vertex { id: Default::default(), t: Default::default() };
                            match k {
                                k if k == "script" => {
                                    f_o = self.create_object(VertexTypes::get_type_by_key(k));
                                    self.create_relationship(&o, &f_o);
                                    self.add_object_properties(&f_o, &json!({
                                            GVID: format!("{}_{}", &k, &v.as_str().unwrap()),
                                            GV_LABEL: &v.as_str().unwrap()
                                        }), PropertyType::Gv);
                                }
                                k if k == "file" => {
                                    self.add_object_properties(&f_o, &json!({k: v.as_str().unwrap()}), PropertyType::Module);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                k if k == RTES => {
                    let o = self.create_object(VertexTypes::get_type_by_key(k));
                    self.create_relationship(&eut, &o);
                    //Rte
                    for r in v.as_array().unwrap().iter() {
                        let r_o = self.create_object(VertexTypes::Rte);
                        self.create_relationship(&o, &r_o);
                        self.add_object_properties(&r_o, &json!({
                            GVID: format!("{}_{}", RTE, &r.as_object().unwrap().get(MODULE).unwrap().as_str().unwrap()),
                            GV_LABEL: &r.as_object().unwrap().get(MODULE).unwrap().as_str().unwrap()
                        }), PropertyType::Gv);

                        let rte_p_o = self.create_object(VertexTypes::Providers);
                        self.create_relationship(&r_o, &rte_p_o);
                        self.add_object_properties(&rte_p_o, &json!({
                            GVID: format!("{}_{}_{}", RTE, PROVIDERS, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                            GV_LABEL: PROVIDERS
                        }), PropertyType::Gv);
                        // REL: RTE -> Features
                        let eut_f_o = self.get_direct_neighbour_object_by_identifier(&eut, VertexTypes::Features);
                        self.create_relationship(&r_o, &eut_f_o);

                        for (k, v) in r.as_object().unwrap().iter() {
                            match k {
                                k if k == MODULE => {
                                    let r_o_p = self.get_object_properties(&r_o).unwrap().props;
                                    let mut p = r_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                    p.insert(k.clone(), v.clone());
                                    self.add_object_properties(&r_o, &p, PropertyType::Base);
                                }
                                //Collector
                                k if k == "collector" => {
                                    let c_o = self.create_object(VertexTypes::get_type_by_key(k));
                                    self.create_relationship(&r_o, &c_o);
                                    self.add_object_properties(&c_o, &json!({
                                                GVID: format!("{}_{}", &k, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                GV_LABEL: &c_o.t.as_str()
                                            }), PropertyType::Gv);
                                }
                                //Connections
                                k if k == CONNECTIONS => {
                                    let cs_o = self.create_object(VertexTypes::get_type_by_key(k));
                                    self.add_object_properties(&cs_o, &json!({
                                                GVID: format!("{}_{}", &k, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                GV_LABEL: &cs_o.t.as_str()
                                            }), PropertyType::Gv);
                                    self.create_relationship(&r_o, &cs_o);

                                    for item in v.as_array().unwrap().iter() {
                                        //Connection
                                        let c_o = self.create_object(VertexTypes::Connection);
                                        self.create_relationship(&cs_o, &c_o);
                                        let c_name = item.as_object().unwrap().get(NAME).unwrap().as_str().unwrap();
                                        self.add_object_properties(&c_o, &json!({NAME: c_name}), PropertyType::Base);
                                        self.add_object_properties(&c_o, &json!({
                                                GVID: format!("{}_{}_{}", "connection", c_name.replace("-", "_"), &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                GV_LABEL: "connection"
                                            }), PropertyType::Gv);

                                        //Sources
                                        let sources = item.as_object().unwrap().get(SOURCES).unwrap().as_array().unwrap();
                                        for s in sources.iter() {
                                            let src_o = self.create_object(VertexTypes::ConnectionSrc);
                                            self.create_relationship(&c_o, &src_o);
                                            self.add_object_properties(&src_o, &json!({NAME: &s, RTE: &r.as_object().unwrap().get(MODULE).unwrap().as_str().unwrap()}), PropertyType::Base);
                                            self.add_object_properties(&src_o, &json!({
                                                    GVID: format!("{}_{}_{}", "connection_src", s.as_str().unwrap(), &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                    GV_LABEL: s.as_str().unwrap()
                                                }), PropertyType::Gv);
                                            //Destinations
                                            let destinations = item.as_object().unwrap().get("destinations").unwrap().as_array().unwrap();
                                            for d in destinations.iter() {
                                                let dst_o = self.create_object(VertexTypes::ConnectionDst);
                                                self.create_relationship(&src_o, &dst_o);
                                                self.add_object_properties(&dst_o, &json!({NAME: &d, RTE: &r.as_object().unwrap().get(MODULE).unwrap().as_str().unwrap()}), PropertyType::Base);
                                                self.add_object_properties(&dst_o, &json!({
                                                         GVID: format!("{}_{}_{}", "connection_dst", d.as_str().unwrap(), &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                         GV_LABEL: d.as_str().unwrap()
                                                     }), PropertyType::Gv);
                                            }
                                            //Tests
                                            let tests = item.as_object().unwrap().get("tests").unwrap().as_array().unwrap();
                                            for test in tests.iter() {
                                                let t_o = self.create_object(VertexTypes::Test);
                                                self.create_relationship(&src_o, &t_o);

                                                for (k, v) in test.as_object().unwrap().iter() {
                                                    match k {
                                                        k if k == NAME => {
                                                            let t_o_p = self.get_object_properties(&t_o).unwrap().props;
                                                            let mut p = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                            p.insert(k.clone(), v.clone());
                                                            self.add_object_properties(&t_o, &p, PropertyType::Base);
                                                        }
                                                        k if k == MODULE => {
                                                            let t_o_p = self.get_object_properties(&t_o).unwrap().props;
                                                            let mut p = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                            p.insert(k.clone(), v.clone());
                                                            self.add_object_properties(&t_o, &p, PropertyType::Base);
                                                        }
                                                        k if k == "parallel" => {
                                                            let mut t_o_p = self.get_object_properties(&t_o).unwrap().props;
                                                            let mut p = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                            p.insert(k.clone(), v.clone());
                                                            self.add_object_properties(&t_o, &p, PropertyType::Base);
                                                            t_o_p = self.get_object_properties(&t_o).unwrap().props;
                                                            let t_name = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(NAME).unwrap().as_str().unwrap();
                                                            let t_module = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(MODULE).unwrap().as_str().unwrap();
                                                            self.add_object_properties(&t_o, &json!({
                                                                             GVID: format!("{}_{}_{}", t_o.t.as_str(), t_name.replace("-", "_"), &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                             GV_LABEL: t_module
                                                                         }), PropertyType::Gv);
                                                        }
                                                        k if k == CI => {
                                                            let t_o_p = self.get_object_properties(&t_o).unwrap().props;
                                                            let mut p = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                            p.append(&mut json!({k: v.as_object().unwrap().clone()}).as_object().unwrap().clone());
                                                            self.add_object_properties(&t_o, &p, PropertyType::Base);
                                                        }
                                                        k if k == "verifications" => {
                                                            for v in v.as_array().unwrap().iter() {
                                                                let v_o = self.create_object(VertexTypes::Verification);
                                                                let v_name = v.as_object().unwrap().get(NAME).unwrap().as_str().unwrap();
                                                                let v_module = v.as_object().unwrap().get(MODULE).unwrap().as_str().unwrap();
                                                                self.add_object_properties(&v_o, v, PropertyType::Base);
                                                                self.add_object_properties(&v_o, &json!({
                                                                                 GVID: format!("{}_{}_{}", v_o.t.as_str(), v_name.replace("-", "_"), &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                                 GV_LABEL: v_module
                                                                             }), PropertyType::Gv);
                                                                self.create_relationship(&t_o, &v_o);
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        //Rte module cfg
                        let r_p = self.get_object_properties(&r_o).unwrap().props;
                        let module = r_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(MODULE).unwrap().as_str().unwrap();
                        let cfg = self.load_object_config(&VertexTypes::get_name_by_object(&r_o), &module);

                        for (k, v) in cfg.as_object().unwrap().iter() {
                            match k {
                                k if k == NAME => {
                                    self.add_object_properties(&r_o, v, PropertyType::Module);
                                }
                                k if k == PROVIDER => {
                                    for (p, v) in v.as_object().unwrap().iter() {
                                        let o = self.create_object(VertexTypes::RteProvider);
                                        self.create_relationship(&rte_p_o, &o);
                                        self.add_object_properties(&o, &json!({NAME: p}), PropertyType::Module);
                                        self.add_object_properties(&o, &json!({
                                                GVID: format!("{}_{}_{}", PROVIDER, p, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                GV_LABEL: p
                                            }), PropertyType::Gv);

                                        for (k, v) in v.as_object().unwrap().iter() {
                                            match k {
                                                k if k == CI => {
                                                    let p_ci_o = self.create_object(VertexTypes::Ci);
                                                    self.create_relationship(&o, &p_ci_o);
                                                    self.add_object_properties(&p_ci_o, &json!({
                                                            GVID: format!("{}_{}_{}_{}", PROVIDER, k, p, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                            GV_LABEL: k
                                                        }), PropertyType::Gv);
                                                    self.add_object_properties(&p_ci_o, &v.as_object().unwrap(), PropertyType::Base);
                                                }
                                                k if k == COMPONENTS => {
                                                    let c_o = self.create_object(VertexTypes::Components);
                                                    self.create_relationship(&o, &c_o);
                                                    self.add_object_properties(&c_o, &json!({
                                                            GVID: format!("{}_{}_{}_{}", PROVIDER, p, k, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                            GV_LABEL: k
                                                        }), PropertyType::Gv);

                                                    for (k, v) in v.as_object().unwrap().iter() {
                                                        match k {
                                                            k if k == "src" => {
                                                                let c_src_o = self.create_object(VertexTypes::ComponentSrc);
                                                                self.create_relationship(&c_o, &c_src_o);
                                                                self.add_object_properties(&c_src_o, &json!({k: v}), PropertyType::Base);
                                                                self.add_object_properties(&c_src_o, &json!({
                                                                        GVID: format!("{}_{}_{}_{}_{}", "rte", p, "component", k, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                        GV_LABEL: k
                                                                    }), PropertyType::Gv);
                                                            }
                                                            k if k == "dst" => {
                                                                let c_dst_o = self.create_object(VertexTypes::ComponentDst);
                                                                self.create_relationship(&c_o, &c_dst_o);
                                                                error!("K: {:?} -----> V: {:?}", k, v);
                                                                self.add_object_properties(&c_dst_o, &json!({k: v}), PropertyType::Base);
                                                                self.add_object_properties(&c_dst_o, &json!({
                                                                        GVID: format!("{}_{}_{}_{}_{}", "rte", p, "component", k, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                        GV_LABEL: k
                                                                    }), PropertyType::Gv);
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                k if k == SCRIPTS => {
                                    let r_s_o = self.create_object(VertexTypes::Scripts);
                                    self.create_relationship(&r_o, &r_s_o);
                                    self.add_object_properties(&r_s_o, &json!({
                                            GVID: format!("{}_{}_{}_{}", RTE, &r_o.t.as_str(), k, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                            GV_LABEL: k
                                        }), PropertyType::Gv);
                                    for s in v.as_array().unwrap().iter() {
                                        let s_o = self.create_object(VertexTypes::Script);
                                        self.create_relationship(&r_s_o, &s_o);
                                        let s_name = s.as_object().unwrap().get("script").unwrap().as_str().unwrap();
                                        self.add_object_properties(&s_o, v, PropertyType::Base);
                                        self.add_object_properties(&s_o, &json!({
                                                GVID: format!("{}_{}_{}", "script", s_name, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                GV_LABEL: s_name
                                            }), PropertyType::Gv);
                                    }
                                }
                                k if k == "script_path" => {
                                    self.add_object_properties(&r_o, v, PropertyType::Module);
                                }
                                _ => {}
                            }
                        }
                        // Connection -> Component
                        let _c = self.get_direct_neighbour_object_by_identifier(&r_o, VertexTypes::Connections);
                        let connections = self.get_neighbour_objects_by_identifier(&_c, VertexTypes::Connection);
                        let provider = self.get_neighbour_objects_by_identifier(&r_o, VertexTypes::RteProvider);

                        for c in connections.iter() {
                            let sources = self.get_neighbour_objects_by_identifier(&c, VertexTypes::ConnectionSrc);

                            for c_s in sources.iter() {
                                let c_p = self.get_object_properties(c_s).unwrap().props;
                                let c_src_name = c_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(NAME).unwrap().as_str().unwrap();
                                let _c_d_s: Vec<Vertex> = self.get_neighbour_objects_by_identifier(&c_s, VertexTypes::ConnectionDst);

                                for _p in provider.iter() {
                                    let p_p = self.get_object_properties(_p).unwrap().props;
                                    let p_name = p_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(NAME).unwrap().as_str().unwrap();

                                    if c_src_name == p_name {
                                        let _c = self.get_direct_neighbour_object_by_identifier(&_p, VertexTypes::Components);
                                        let component_src = self.get_direct_neighbour_object_by_identifier(&_c, VertexTypes::ComponentSrc);
                                        self.create_relationship(&c_s, &component_src);
                                    }
                                }

                                //CONNECTION DSTs
                                for c_d in _c_d_s.iter() {
                                    let c_d_p = self.get_object_properties(c_d).unwrap().props;
                                    let c_dst_name = c_d_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(NAME).unwrap().as_str().unwrap();

                                    for _p in provider.iter() {
                                        let p_p = self.get_object_properties(_p).unwrap().props;
                                        let p_name = p_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(NAME).unwrap().as_str().unwrap();

                                        if c_dst_name == p_name {
                                            let _c = self.get_direct_neighbour_object_by_identifier(&_p, VertexTypes::Components);
                                            let component_dst = self.get_direct_neighbour_object_by_identifier(&_c, VertexTypes::ComponentDst);
                                            self.create_relationship(&c_d, &component_dst);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                &_ => {}
            }
        }

        //Eut Stages Deploy
        let eut_stage_deploy = self.add_ci_stages(&ci, &self.config.eut.ci.stages.deploy, &VertexTypes::StageDeploy);

        //Rte Stages Deploy
        let rte_stage_deploy = self.add_ci_stages(&eut_stage_deploy.unwrap(), &self.config.rte.ci.stages.deploy, &VertexTypes::StageDeploy);

        //Feature Stages Deploy
        self.add_ci_stages(&rte_stage_deploy.unwrap(), &self.config.features.ci.stages.deploy, &VertexTypes::StageDeploy);

        //Feature Stages Destroy
        let mut stage_destroy: Option<Vertex> = None;
        let features = self.get_neighbour_objects_by_identifier(&eut, VertexTypes::Feature);
        if features.len() > 0 {
            stage_destroy = self.add_ci_stages(&ci, &self.config.features.ci.stages.destroy, &VertexTypes::StageDestroy);
        }

        //Rte Stages Destroy
        match stage_destroy {
            Some(p) => stage_destroy = self.add_ci_stages(&p, &self.config.rte.ci.stages.destroy, &VertexTypes::StageDestroy),
            None => stage_destroy = self.add_ci_stages(&ci, &self.config.rte.ci.stages.destroy, &VertexTypes::StageDestroy)
        }

        //Eut Stages Destroy
        self.add_ci_stages(&stage_destroy.unwrap(), &self.config.eut.ci.stages.destroy, &VertexTypes::StageDestroy);

        project.id
    }

    fn create_object(&self, object_type: VertexTypes) -> Vertex {
        info!("Create new object of type <{}>...", object_type.name());
        let o = Vertex::new(Identifier::new(object_type.name()).unwrap());
        self.regression.create_vertex(&o).expect("panic while creating project db entry");
        self.add_object_properties(&o, &json!({}), PropertyType::Base);
        self.add_object_properties(&o, &json!({}), PropertyType::Gv);
        self.add_object_properties(&o, &json!({}), PropertyType::Module);
        info!("Create new object of type <{}> -> Done", object_type.name());
        o
    }

    fn create_relationship_identifier(&self, a: &Vertex, b: &Vertex) -> Identifier {
        info!("Create relationship identifier for <{}> and <{}>...", a.t.as_str(), b.t.as_str());
        let i = Identifier::new(self.get_relationship_type(&a, &b)).unwrap();
        info!("Create relationship identifier for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        i
    }

    fn create_relationship(&self, a: &Vertex, b: &Vertex) -> bool {
        info!("Create relationship for <{}> and <{}>...", a.t.as_str(), b.t.as_str());
        let i = self.create_relationship_identifier(&a, &b);
        let e = Edge::new(a.id, i, b.id);
        let status = self.regression.create_edge(&e).expect(&format!("panic build relationship between {} and {}", a.t.as_str(), b.t.as_str()));
        info!("Create relationship for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        status
    }

    fn add_object_properties<T: serde::Serialize>(&self, object: &Vertex, value: &T, property_type: PropertyType) {
        info!("Add new property to object <{}>...", object.t.to_string());
        let v = serde_json::to_value(value).unwrap();
        let p: BulkInsertItem;

        match property_type {
            PropertyType::Gv => {
                p = BulkInsertItem::VertexProperty(object.id, Identifier::new(PROPERTY_TYPE_GV)
                    .unwrap(), Json::new(v.clone()));
            }
            PropertyType::Base => {
                p = BulkInsertItem::VertexProperty(object.id, Identifier::new(PROPERTY_TYPE_BASE)
                    .unwrap(), Json::new(v.clone()));
            }
            PropertyType::Module => {
                p = BulkInsertItem::VertexProperty(object.id, Identifier::new(PROPERTY_TYPE_MODULE)
                    .unwrap(), Json::new(v.clone()));
            }
        }

        self.regression.bulk_insert(vec![p]).unwrap();
        info!("Add new property to object <{}> -> Done", object.t.to_string());
    }

    #[allow(dead_code)]
    fn add_relationship_properties<T: serde::Serialize>(&self, object: &Edge, value: &T, property_type: PropertyType) {
        info!("Add new property to relationship <{}>...", object.t.to_string());
        let v = serde_json::to_value(value).unwrap();
        let p: BulkInsertItem;

        match property_type {
            PropertyType::Gv => {
                p = BulkInsertItem::EdgeProperty(object.clone(), Identifier::new(PROPERTY_TYPE_GV)
                    .unwrap(), Json::new(v.clone()));
            }
            PropertyType::Base => {
                p = BulkInsertItem::EdgeProperty(object.clone(), Identifier::new(PROPERTY_TYPE_BASE)
                    .unwrap(), Json::new(v.clone()));
            }
            PropertyType::Module => {
                p = BulkInsertItem::EdgeProperty(object.clone(), Identifier::new(PROPERTY_TYPE_MODULE)
                    .unwrap(), Json::new(v.clone()));
            }
        }
        self.regression.bulk_insert(vec![p]).unwrap();
        info!("Add new property to relationship <{}> -> Done", object.t.to_string());
    }

    #[allow(dead_code)]
    fn get_relationship_count() -> usize {
        info!("Relationship count: <{}>", *EDGES_COUNT);
        *EDGES_COUNT
    }

    fn get_relationship_type(&self, a: &Vertex, b: &Vertex) -> &str {
        info!("Get relationship type for <{}> and <{}>...", a.t.as_str(), b.t.as_str());
        error!("RELA -----> {:?}, {:?}", a.t.to_string(), b.t.to_string());
        let e = EDGE_TYPES.get(&VertexTuple(a.t.to_string(), b.t.to_string())).unwrap();
        info!("Get relationship type for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        e
    }

    fn get_object(&self, id: Uuid) -> Vertex {
        let q = self.regression.get(indradb::SpecificVertexQuery::single(id));
        let objs = indradb::util::extract_vertices(q.unwrap());
        let obj = objs.unwrap();
        let o = obj.get(0).unwrap();
        o.clone()
    }

    fn get_direct_neighbour_object_by_identifier(&self, object: &Vertex, identifier: VertexTypes) -> Vertex {
        info!("Get direct neighbor of <{}>...", object.t.as_str());
        let mut rvq = RangeVertexQuery::new();
        rvq.t = Option::from(Identifier::new(identifier.name()).unwrap());
        rvq.limit = 1;
        rvq.start_id = Option::from(object.id);
        let result = indradb::util::extract_vertices(self.regression.get(rvq).unwrap()).unwrap();
        info!("Get direct neighbor of <{}> -> Done.", object.t.as_str());
        result.get(0).unwrap().clone()
    }

    fn get_neighbour_objects_by_identifier(&self, start: &Vertex, identifier: VertexTypes) -> Vec<Vertex> {
        info!("Get direct neighbors of <{}>...", start.t.as_str());
        let mut rvq = RangeVertexQuery::new();
        rvq.t = Option::from(Identifier::new(identifier.name()).unwrap());
        rvq.limit = 20;
        rvq.start_id = Option::from(start.id);
        let result = indradb::util::extract_vertices(self.regression.get(rvq).unwrap()).unwrap();
        info!("Get direct neighbors of <{}> -> Done.", start.t.as_str());
        result
    }

    fn get_objects_by_property_value(&self, identifier: VertexTypes, value: &str) -> Vec<Vertex> {
        info!("Get objects with value <{}>...", value);
        let vwpvq = VertexWithPropertyValueQuery { name: Identifier::new(identifier.name()).unwrap(), value: Json::new(serde_json::Value::from(value)) };
        let result = indradb::util::extract_vertices(self.regression.get(vwpvq).unwrap()).unwrap();
        info!("Get objects with value <{}> -> Done", value);
        result
    }

    fn get_object_properties(&self, object: &Vertex) -> Option<VertexProperties> {
        info!("Get object <{}> properties...", object.t.as_str());
        let b = indradb::SpecificVertexQuery::new(vec!(object.id)).properties().unwrap();
        let _r = self.regression.get(b);
        return match _r {
            Ok(qov) => {
                let vp = indradb::util::extract_vertex_properties(qov);
                if vp.clone().unwrap().get(0).is_some() {
                    info!("Get object <{}> properties -> Done.", object.t.as_str());
                    Some(vp.unwrap().get(0).unwrap().clone())
                } else {
                    info!("Get object <{}> properties -> Done.", object.t.as_str());
                    None
                }
            }
            Err(e) => {
                error!("Error in properties query: {}", &e);
                None
            }
        };
    }

    #[allow(dead_code)]
    fn get_relationship(&self, a: &Vertex, b: &Vertex) -> Vec<Edge> {
        info!("Get relationship for <{}> and <{}>...", &a.t.as_str(), &b.t.as_str());
        let i = self.create_relationship_identifier(&a, &b);
        let e = Edge::new(a.id, i, b.id);
        let r: Vec<indradb::QueryOutputValue> = self.regression.get(SpecificEdgeQuery::single(e.clone())).unwrap();
        let e = indradb::util::extract_edges(r).unwrap();
        info!("Get relationship for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        e
    }

    fn build_context(&self, id: Uuid) -> Context {
        info!("Build render context...");
        let project = self.get_object(id);
        let project_p = self.get_object_properties(&project).unwrap().props;
        let project_p_base = project_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap();

        let eut = self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::Eut);
        let eut_p = self.get_object_properties(&eut).unwrap().props;
        let eut_p_base = eut_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap();
        let eut_p_module = eut_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();

        let _providers = self.get_direct_neighbour_object_by_identifier(&eut, VertexTypes::Providers);
        let _eut_provider = self.get_neighbour_objects_by_identifier(&_providers, VertexTypes::EutProvider);
        let mut eut_provider = Vec::new();

        for p in _eut_provider.iter() {
            let _p = self.get_object_properties(&p).unwrap().props;
            let name = _p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(NAME).unwrap().as_str().unwrap();
            eut_provider.push(String::from(name));
        }

        let _eut_ci_o = self.get_direct_neighbour_object_by_identifier(&eut, VertexTypes::Ci);
        let eut_ci_p = self.get_object_properties(&_eut_ci_o).unwrap().props;
        let eut_ci = eut_ci_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();

        let _features = self.get_neighbour_objects_by_identifier(&eut, VertexTypes::Feature);
        let mut features = Vec::new();
        for feature in _features.iter() {
            let feature_p = self.get_object_properties(&feature).unwrap().props;
            features.push(feature_p.get(PropertyType::Base.index()).unwrap().value.clone())
        }

        let _rtes = self.get_neighbour_objects_by_identifier(&eut, VertexTypes::Rte);

        let mut rtes: Vec<RteRenderContext> = Vec::new();
        error!("##################################################################");
        for rte in _rtes.iter() {
            let rte_p = self.get_object_properties(&rte).unwrap().props;
            let rte_name = rte_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("module").unwrap().as_str().unwrap();
            let _c = self.get_direct_neighbour_object_by_identifier(rte, VertexTypes::Connections);
            let connections = self.get_neighbour_objects_by_identifier(&_c, VertexTypes::Connection);
            let mut rte_crcs = RteRenderContext { connections: Default::default(), ci: HashMap::new() };
            let provider = self.get_neighbour_objects_by_identifier(rte, VertexTypes::RteProvider);

            for p in provider.iter() {
                let p_props = self.get_object_properties(&p).unwrap().props;
                let ci_o = self.get_direct_neighbour_object_by_identifier(p, VertexTypes::Ci);
                let _ci_p = self.get_object_properties(&ci_o).unwrap().props;
                let p_name = p_props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(NAME).unwrap().as_str().unwrap();
                rte_crcs.ci.insert(p_name.to_string(), RteProviderCiRenderContext {
                    timeout: _ci_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("timeout").unwrap().clone(),
                    variables: _ci_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("variables").unwrap().clone(),
                });
            }

            for conn in connections.iter() {
                let srcs = self.get_neighbour_objects_by_identifier(conn, VertexTypes::ConnectionSrc);

                for src in srcs.iter() {
                    let src_p = self.get_object_properties(&src).unwrap().props;
                    let _rte_src_name = src_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("rte").unwrap().as_str().unwrap();
                    if rte_name == _rte_src_name {
                        let src_name = src_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(NAME).unwrap().as_str().unwrap();
                        let component_src = self.get_direct_neighbour_object_by_identifier(src, VertexTypes::ComponentSrc);
                        let comp_prop = self.get_object_properties(&component_src).unwrap().props;
                        let comp_src_name = &comp_prop.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("src").unwrap().as_str().unwrap();
                        let rte_job_name = format!("rte_{}_{}_{}", &rte_name, &src_name, &comp_src_name);
                        let rte_crc = RteConnectionRenderContext {
                            job: rte_job_name.clone(),
                            rte: rte_name.to_string(),
                            component: comp_src_name.to_string(),
                            provider: src_name.to_string(),
                        };
                        rte_crcs.connections.push(rte_crc);

                        let dsts = self.get_neighbour_objects_by_identifier(src, VertexTypes::ConnectionDst);
                        for dst in dsts.iter() {
                            let dst_p = self.get_object_properties(&dst).unwrap().props;
                            let _rte_dst_name = dst_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("rte").unwrap().as_str().unwrap();
                            if rte_name == _rte_dst_name {
                                let dst_p = self.get_object_properties(dst).unwrap().props;
                                let dst_name = dst_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(NAME).unwrap().as_str().unwrap();
                                let component_dst = self.get_direct_neighbour_object_by_identifier(dst, VertexTypes::ComponentDst);
                                let comp_prop = self.get_object_properties(&component_dst).unwrap().props;
                                let comp_dst_name = &comp_prop.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("dst").unwrap().as_str().unwrap();
                                let rte_job_name = format!("rte_{}_{}_{}", &rte_name, &dst_name, &comp_dst_name);
                                let rte_crc = RteConnectionRenderContext {
                                    job: rte_job_name.clone(),
                                    rte: rte_name.to_string(),
                                    component: comp_dst_name.to_string(),
                                    provider: dst_name.to_string(),
                                };
                                rte_crcs.connections.push(rte_crc);
                            }
                        }
                    }
                }
            }
            error!("STRUCT: {:#?}", &rte_crcs);
            rtes.push(rte_crcs);

        }
        error!("##################################################################");

        let mut stages: Vec<String> = Vec::new();
        let mut deploy_stages: Vec<String> = Vec::new();
        let mut destroy_stages: Vec<String> = Vec::new();
        let s_deploy = self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::StageDeploy);
        let s_destroy = self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::StageDestroy);

        for _stage in self.get_neighbour_objects_by_identifier(&s_deploy, VertexTypes::StageDeploy).iter() {
            let _p = &self.get_object_properties(&_stage).unwrap().props;
            let p = &_p.get(PropertyType::Base.index()).unwrap().value;
            deploy_stages.push(p.as_str().unwrap().to_string());
        }

        for _stage in self.get_neighbour_objects_by_identifier(&s_destroy, VertexTypes::StageDestroy).iter() {
            let _p = &self.get_object_properties(&_stage).unwrap().props;
            let p = &_p.get(PropertyType::Base.index()).unwrap().value;
            destroy_stages.push(p.as_str().unwrap().to_string());
        }

        stages.append(&mut deploy_stages);
        stages.append(&mut destroy_stages);

        let data = EutRenderContext {
            base: eut_p_base.clone(),
            module: eut_p_module.clone(),
            ci: eut_ci.clone(),
            provider: eut_provider,
        };

        let mut context = Context::new();
        context.insert(RTES, &rtes);
        context.insert(EUT, &data);
        context.insert("config", &self.config);
        context.insert("stages", &stages);
        context.insert(FEATURES, &features);
        context.insert(PROJECT, &project_p_base);

        //error!("{:#?}", context);
        info!("Build render context -> Done.");
        context
    }

    pub fn render(&self, context: &Context) -> String {
        info!("Render regression pipeline file first step...");
        let mut _tera = Tera::new(&self.config.project.templates).unwrap();
        let rendered = _tera.render(PIPELINE_TEMPLATE_FILE_NAME, &context).unwrap();
        info!("Render regression pipeline file first step -> Done.");
        rendered
    }

    #[allow(dead_code)]
    pub fn to_json(&self) -> String {
        let j = json!({
                "config": &self.config,
            });
        j.to_string()
    }

    pub fn to_file(&self, data: &str, file: &str) {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file)
            .expect("Couldn't open file");

        f.write_all(data.as_bytes()).expect("panic while writing to file");
    }

    pub fn to_gv(&self) -> String {
        let mut context = Context::new();

        // Nodes
        let q = AllVertexQuery;
        let result = self.regression.get(q);
        let _nodes = indradb::util::extract_vertices(result.unwrap());

        match &_nodes {
            Some(nodes) => {
                let mut items = Vec::new();

                for n in nodes.iter() {
                    let n_p = self.get_object_properties(&self.get_object(n.id));
                    match n_p {
                        Some(p) => {
                            let gv_id = p.props.get(PropertyType::Gv.index()).unwrap().value[GVID].as_str();
                            let gv_label = p.props.get(PropertyType::Gv.index()).unwrap().value[GV_LABEL].as_str();
                            match gv_id {
                                Some(_) => items.push(json!({"id": &gv_id.unwrap(), "label": &gv_label, "shape": "circle"})),
                                None => items.push(json!({"id": &n.t.as_str(), "label": &n.t.as_str(), "shape": "circle"}))
                            }
                        }
                        None => {
                            items.push(json!({"id": &n.t.as_str(), "name": &n.t.as_str(), "shape": "circle"}))
                        }
                    }
                };
                context.insert("nodes", &items);
            }
            None => {}
        }

        //Edges
        let q = AllVertexQuery.include().outbound();
        let result = self.regression.get(q.unwrap()).unwrap();
        let _edges = indradb::util::extract_edges(result);

        match &_edges {
            Some(edge) => {
                let mut items = Vec::new();

                for e in edge.iter() {
                    let o_a = self.get_object(e.outbound_id);
                    let o_b = self.get_object(e.inbound_id);
                    let a_id = format!("{}", self.get_object(e.outbound_id).t.to_string());
                    let b_id = format!("{}", self.get_object(e.inbound_id).t.to_string());
                    let a_p = self.get_object_properties(&self.get_object(o_a.id));
                    let b_p = self.get_object_properties(&self.get_object(o_b.id));

                    match a_p {
                        Some(ap) => {
                            match b_p {
                                Some(bp) => {
                                    let a_p_name = &ap.props.get(PropertyType::Gv.index()).unwrap().value[GVID].as_str();
                                    let b_p_name = &bp.props.get(PropertyType::Gv.index()).unwrap().value[GVID].as_str();

                                    match a_p_name {
                                        Some(ap) => {
                                            match b_p_name {
                                                Some(bp) => items.push(json!({"src": &ap, "dst": &bp})),
                                                None => items.push(json!({"src": &ap, "dst": &b_id}))
                                            }
                                        }
                                        None => {
                                            match b_p_name {
                                                Some(bp) => items.push(json!({"src": &a_id, "dst": &bp})),
                                                None => items.push(json!({"src": &a_id, "dst": &b_id}))
                                            }
                                        }
                                    }
                                }
                                None => {
                                    let a_p_name = &ap.props.get(PropertyType::Gv.index()).unwrap().value[GVID].as_str();
                                    match a_p_name {
                                        Some(ap) => items.push(json!({"src": &ap, "dst": &b_id})),
                                        None => items.push(json!({"src": &a_id, "dst": &b_id}))
                                    }
                                }
                            }
                        }
                        None => {
                            match b_p {
                                Some(bp) => {
                                    let b_p_name = &bp.props.get(PropertyType::Gv.index()).unwrap().value[GVID].as_str();
                                    match b_p_name {
                                        Some(bp) => items.push(json!({"src": &a_id, "dst": &bp})),
                                        None => items.push(json!({"src": &a_id, "dst": &b_id}))
                                    }
                                }
                                None => items.push(json!({"src": &a_id, "dst": &b_id}))
                            }
                        }
                    }
                };
                context.insert("edges", &items);
            }
            None => {}
        }

        let mut _tera = Tera::new(&self.config.project.templates).unwrap();
        let rendered = _tera.render("graph.tpl", &context).unwrap();

        rendered
    }
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let r = Regression::new(&cli.config);
    let p = r.init();
    r.to_file(&r.render(&r.build_context(p)), PIPELINE_FILE_NAME);
    r.to_file(&r.to_gv(), &"graph.gv");

    /*if cli.write {
        r.to_file(&PIPELINE_FILE_NAME.to_string());
    }
    if cli.json {
        r.to_json();
         info!("{}", r.to_json());
    }*/
    /*if cli.render {
         info!("{}", r.render());
    }*/

    /*if cli.debug {
         info!("{:#?}", r);
    }*/
}
