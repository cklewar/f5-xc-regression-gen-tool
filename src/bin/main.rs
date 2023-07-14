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
use std::io::{stderr, Write};
use std::option::{Iter, Option};
use std::string::ToString;

use clap::error::ErrorKind::Format as clap_format;
use clap::Parser;
use graphviz_rust::as_item;
use indradb;
use indradb::{AllVertexQuery, BulkInsertItem, Edge, Identifier, QueryExt, RangeVertexQuery, Vertex, VertexProperties, VertexProperty};
use indradb::Query::AllVertex;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use phf::phf_map;
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use serde_json::Value::Null;
use tera::{Context, Tera};
use uuid::Uuid;

const CONFIG_FILE_NAME: &str = "config.json";

// const SCRIPT_TYPE_APPLY: &str = "apply";
// const SCRIPT_TYPE_DESTROY: &str = "destroy";
// const SCRIPT_TYPE_ARTIFACTS: &str = "artifacts";
// const SCRIPT_TYPE_COLLECTOR_PATH: &str = "scripts";

// KEY MAP TYPES
const DIRECT: &str = "direct";
const OBJECT: &str = "object";

// KEYS
const CI: &str = "ci";
const EUT: &str = "eut";
const RTE: &str = "rte";
const RTES: &str = "rtes";
const TEST: &str = "test";
const GVID: &str = "id";
const GV_LABEL: &str = "label";
const NAME: &str = "name";
const SCRIPTS: &str = "scripts";
const FEATURE: &str = "feature";
const FEATURES: &str = "features";
const PROVIDER: &str = "provider";
const ENDPOINTS: &str = "endpoints";
const VERIFICATION: &str = "verification";

const PIPELINE_FILE_NAME: &str = ".gitlab-ci.yml";
const PIPELINE_TEMPLATE_FILE_NAME: &str = ".gitlab-ci.yml.tpl";

const PROPERTY_TYPE_GV: &str = "gv";
const PROPERTY_TYPE_EUT: &str = "eut";
const PROPERTY_TYPE_BASE: &str = "base";
const PROPERTY_TYPE_MODULE: &str = "module";

const VERTEX_TYPE_CI: &str = "ci";
const VERTEX_TYPE_EUT: &str = "eut";
const VERTEX_TYPE_RTE: &str = "rte";
const VERTEX_TYPE_RTES: &str = "rtes";
const VERTEX_TYPE_TEST: &str = "test";
const VERTEX_TYPE_NONE: &str = "none";
const VERTEX_TYPE_NAME: &str = "name";
const VERTEX_TYPE_VALUE: &str = "value";
const VERTEX_TYPE_SCRIPT: &str = "script";
const VERTEX_TYPE_SCRIPTS: &str = "scripts";
const VERTEX_TYPE_PROJECT: &str = "project";
const VERTEX_TYPE_FEATURE: &str = "feature";
const VERTEX_TYPE_FEATURES: &str = "features";
const VERTEX_TYPE_PROVIDER: &str = "provider";
const VERTEX_TYPE_CONNECTION: &str = "connection";
const VERTEX_TYPE_CONNECTIONS: &str = "connections";
const VERTEX_TYPE_COLLECTOR: &str = "collector";
const VERTEX_TYPE_COMPONENTS: &str = "components";
const VERTEX_TYPE_CONNECTION_SRC: &str = "source";
const VERTEX_TYPE_CONNECTION_DST: &str = "destination";
const VERTEX_TYPE_PROVIDER_AWS: &str = "aws";
const VERTEX_TYPE_PROVIDER_GCP: &str = "gcp";
const VERTEX_TYPE_VERIFICATION: &str = "verification";
const VERTEX_TYPE_SCRIPT_APPLY: &str = "apply";
const VERTEX_TYPE_STAGE_DEPLOY: &str = "deploy";
const VERTEX_TYPE_STAGE_DESTROY: &str = "stage_destroy";
const VERTEX_TYPE_COMPONENT_SRC: &str = "component_src";
const VERTEX_TYPE_COMPONENT_DST: &str = "component_dst";
const VERTEX_TYPE_PROVIDER_AZURE: &str = "azure";
const VERTEX_TYPE_SCRIPT_ARTIFACTS: &str = "artifacts";

const EDGE_TYPE_IS: &str = "is";
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
    Name,
    Value,
    Script,
    Project,
    Scripts,
    Feature,
    Features,
    Provider,
    Collector,
    Components,
    Connection,
    Connections,
    ConnectionSrc,
    ConnectionDst,
    ProviderAws,
    ProviderGcp,
    StageDeploy,
    StageDestroy,
    Verification,
    ComponentSrc,
    ComponentDst,
    ProviderAzure,
    None,
}

enum EdgeTypes {
    Is,
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
            VertexTypes::Name => VERTEX_TYPE_NAME,
            VertexTypes::Value => VERTEX_TYPE_VALUE,
            VertexTypes::Script => VERTEX_TYPE_SCRIPT,
            VertexTypes::Scripts => VERTEX_TYPE_SCRIPTS,
            VertexTypes::Project => VERTEX_TYPE_PROJECT,
            VertexTypes::Feature => VERTEX_TYPE_FEATURE,
            VertexTypes::Features => VERTEX_TYPE_FEATURES,
            VertexTypes::Provider => VERTEX_TYPE_PROVIDER,
            VertexTypes::Connection => VERTEX_TYPE_CONNECTION,
            VertexTypes::Connections => VERTEX_TYPE_CONNECTIONS,
            VertexTypes::Collector => VERTEX_TYPE_COLLECTOR,
            VertexTypes::Components => VERTEX_TYPE_COMPONENTS,
            VertexTypes::ConnectionSrc => VERTEX_TYPE_CONNECTION_SRC,
            VertexTypes::ConnectionDst => VERTEX_TYPE_CONNECTION_DST,
            VertexTypes::ProviderAws => VERTEX_TYPE_PROVIDER_AWS,
            VertexTypes::ProviderGcp => VERTEX_TYPE_PROVIDER_GCP,
            VertexTypes::StageDeploy => VERTEX_TYPE_STAGE_DEPLOY,
            VertexTypes::StageDestroy => VERTEX_TYPE_STAGE_DESTROY,
            VertexTypes::Verification => VERTEX_TYPE_VERIFICATION,
            VertexTypes::ComponentSrc => VERTEX_TYPE_COMPONENT_SRC,
            VertexTypes::ComponentDst => VERTEX_TYPE_COMPONENT_DST,
            VertexTypes::ProviderAzure => VERTEX_TYPE_PROVIDER_AZURE,
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
            VERTEX_TYPE_PROVIDER => VertexTypes::Provider.name(),
            VERTEX_TYPE_COLLECTOR => VertexTypes::Collector.name(),
            VERTEX_TYPE_CONNECTION => VertexTypes::Connection.name(),
            VERTEX_TYPE_CONNECTIONS => VertexTypes::Connections.name(),
            VERTEX_TYPE_COMPONENTS => VertexTypes::Components.name(),
            VERTEX_TYPE_PROVIDER_AWS => VertexTypes::ProviderAws.name(),
            VERTEX_TYPE_PROVIDER_GCP => VertexTypes::ProviderGcp.name(),
            VERTEX_TYPE_VERIFICATION => VertexTypes::Verification.name(),
            VERTEX_TYPE_STAGE_DEPLOY => VertexTypes::StageDeploy.name(),
            VERTEX_TYPE_STAGE_DESTROY => VertexTypes::StageDestroy.name(),
            VERTEX_TYPE_COMPONENT_SRC => VertexTypes::ComponentSrc.name(),
            VERTEX_TYPE_COMPONENT_DST => VertexTypes::ComponentDst.name(),
            VERTEX_TYPE_PROVIDER_AZURE => VertexTypes::ProviderAzure.name(),
            VERTEX_TYPE_CONNECTION_SRC => VertexTypes::ConnectionSrc.name(),
            VERTEX_TYPE_CONNECTION_DST => VertexTypes::ConnectionDst.name(),
            _ => "None"
        }
    }
    fn get_name_by_key(key: &str) -> &'static str {
        match key {
            VERTEX_TYPE_CI => VertexTypes::Ci.name(),
            VERTEX_TYPE_RTE => VertexTypes::Rte.name(),
            VERTEX_TYPE_EUT => VertexTypes::Eut.name(),
            VERTEX_TYPE_RTES => VertexTypes::Rtes.name(),
            VERTEX_TYPE_TEST => VertexTypes::Test.name(),
            VERTEX_TYPE_SCRIPTS => VertexTypes::Scripts.name(),
            VERTEX_TYPE_PROJECT => VertexTypes::Project.name(),
            VERTEX_TYPE_FEATURE => VertexTypes::Feature.name(),
            VERTEX_TYPE_PROVIDER => VertexTypes::Provider.name(),
            VERTEX_TYPE_CONNECTION => VertexTypes::Connection.name(),
            VERTEX_TYPE_CONNECTIONS => VertexTypes::Connections.name(),
            VERTEX_TYPE_COLLECTOR => VertexTypes::Collector.name(),
            VERTEX_TYPE_COMPONENTS => VertexTypes::Components.name(),
            VERTEX_TYPE_CONNECTION_SRC => VertexTypes::ConnectionSrc.name(),
            VERTEX_TYPE_CONNECTION_DST => VertexTypes::ConnectionDst.name(),
            VERTEX_TYPE_PROVIDER_AWS => VertexTypes::ProviderAws.name(),
            VERTEX_TYPE_PROVIDER_GCP => VertexTypes::ProviderGcp.name(),
            VERTEX_TYPE_VERIFICATION => VertexTypes::Verification.name(),
            VERTEX_TYPE_STAGE_DEPLOY => VertexTypes::StageDeploy.name(),
            VERTEX_TYPE_STAGE_DESTROY => VertexTypes::StageDestroy.name(),
            VERTEX_TYPE_COMPONENT_SRC => VertexTypes::ComponentSrc.name(),
            VERTEX_TYPE_COMPONENT_DST => VertexTypes::ComponentDst.name(),
            VERTEX_TYPE_PROVIDER_AZURE => VertexTypes::ProviderAzure.name(),
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
            VERTEX_TYPE_PROVIDER => VertexTypes::Provider,
            VERTEX_TYPE_CONNECTION => VertexTypes::Connection,
            VERTEX_TYPE_CONNECTIONS => VertexTypes::Connections,
            VERTEX_TYPE_COMPONENTS => VertexTypes::Components,
            VERTEX_TYPE_COLLECTOR => VertexTypes::Collector,
            VERTEX_TYPE_CONNECTION_SRC => VertexTypes::ConnectionSrc,
            VERTEX_TYPE_CONNECTION_DST => VertexTypes::ConnectionDst,
            VERTEX_TYPE_PROVIDER_AWS => VertexTypes::ProviderAws,
            VERTEX_TYPE_PROVIDER_GCP => VertexTypes::ProviderGcp,
            VERTEX_TYPE_VERIFICATION => VertexTypes::Verification,
            VERTEX_TYPE_STAGE_DEPLOY => VertexTypes::StageDeploy,
            VERTEX_TYPE_STAGE_DESTROY => VertexTypes::StageDestroy,
            VERTEX_TYPE_COMPONENT_SRC => VertexTypes::ComponentSrc,
            VERTEX_TYPE_COMPONENT_DST => VertexTypes::ComponentDst,
            VERTEX_TYPE_PROVIDER_AZURE => VertexTypes::ProviderAzure,
            _ => VertexTypes::None
        }
    }
}

impl EdgeTypes {
    fn name(&self) -> &'static str {
        match *self {
            EdgeTypes::Is => EDGE_TYPE_IS,
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
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Provider.name().to_string()), EdgeTypes::Uses.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Scripts.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Scripts.name().to_string(), VertexTypes::Script.name().to_string()), EdgeTypes::Uses.name());
        map.insert(VertexTuple(VertexTypes::Script.name().to_string(), VertexTypes::Value.name().to_string()), EdgeTypes::Uses.name());
        map.insert(VertexTuple(VertexTypes::Features.name().to_string(), VertexTypes::Feature.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Rtes.name().to_string(), VertexTypes::Rte.name().to_string()), EdgeTypes::Uses.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Provider.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Connections.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Collector.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Provider.name().to_string(), VertexTypes::ProviderAws.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Provider.name().to_string(), VertexTypes::ProviderGcp.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Provider.name().to_string(), VertexTypes::ProviderAzure.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::ProviderAws.name().to_string(), VertexTypes::Components.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::ProviderGcp.name().to_string(), VertexTypes::Components.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::ProviderAzure.name().to_string(), VertexTypes::Components.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Components.name().to_string(), VertexTypes::ComponentSrc.name().to_string()), EdgeTypes::Provides.name());
        map.insert(VertexTuple(VertexTypes::Components.name().to_string(), VertexTypes::ComponentDst.name().to_string()), EdgeTypes::Provides.name());
        map.insert(VertexTuple(VertexTypes::Connections.name().to_string(), VertexTypes::Connection.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Connection.name().to_string(), VertexTypes::ConnectionSrc.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::ConnectionDst.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::Test.name().to_string()), EdgeTypes::Runs.name());
        map.insert(VertexTuple(VertexTypes::ConnectionDst.name().to_string(), VertexTypes::Test.name().to_string()), EdgeTypes::Provides.name());
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
                return Value::Null;
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

        for (k, v) in v.as_object().unwrap().iter() {
            if v.is_string() {
                self.add_object_properties(&eut, &v, PropertyType::Module);
            } else {
                match k {
                    k if k == PROVIDER => {
                        let o = self.create_object(VertexTypes::get_type_by_key(k));
                        self.create_relationship(&eut, &o);
                        for p in v.as_array().unwrap().iter() {
                            let p_o = self.create_object(VertexTypes::get_type_by_key(p.as_str().unwrap()));
                            self.create_relationship(&o, &p_o);
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
                                self.add_object_properties(&f_o, &json!({
                                    GVID: format!("{}_{}", &k, &v.as_str().unwrap()),
                                    GV_LABEL: &v.as_str().unwrap()
                                }), PropertyType::Gv);
                                self.create_relationship(&o, &f_o);
                            }
                        }
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
                            self.add_object_properties(&r_o, &json!({
                                                GVID: format!("{}_{}", &k, &r.as_object().unwrap().get("module").unwrap().as_str().unwrap()),
                                                GV_LABEL: &r.as_object().unwrap().get("module").unwrap().as_str().unwrap()
                                            }), PropertyType::Gv);
                            self.create_relationship(&o, &r_o);
                            for (k, v) in r.as_object().unwrap().iter() {
                                match k {
                                    k if k == "module" => {
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
                                    k if k == "connections" => {
                                        let eps_o = self.create_object(VertexTypes::get_type_by_key(k));
                                        self.add_object_properties(&eps_o, &json!({
                                                GVID: format!("{}_{}", &k, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                GV_LABEL: &eps_o.t.as_str()
                                            }), PropertyType::Gv);
                                        self.create_relationship(&r_o, &eps_o);

                                        for item in v.as_array().unwrap().iter() {
                                            //Connection
                                            let ep_o = self.create_object(VertexTypes::Connection);
                                            self.create_relationship(&eps_o, &ep_o);
                                            let mut dst_objs: Vec<Uuid> = Vec::new();
                                            let mut test_objs: Vec<Uuid> = Vec::new();

                                            for (k, v) in item.as_object().unwrap().iter() {
                                                match k {
                                                    //Connection Name
                                                    k if k == NAME => {
                                                        self.add_object_properties(&ep_o, &json!({k:  v.as_str().unwrap()}), PropertyType::Base);
                                                        self.add_object_properties(&ep_o, &json!({
                                                            GVID: format!("{}_{}_{}", ep_o.t.as_str(), v.as_str().unwrap().replace("-", "_"), &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                            GV_LABEL: v.as_str().unwrap()
                                                        }), PropertyType::Gv);
                                                    }
                                                    k if k == "sources" => {
                                                        for s in v.as_array().unwrap() {
                                                            let src_o = self.create_object(VertexTypes::ConnectionSrc);
                                                            self.create_relationship(&ep_o, &src_o);
                                                            self.add_object_properties(&src_o, &json!({
                                                                GVID: format!("{}_{}_{}", &k, s.as_str().unwrap(), &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                GV_LABEL: s.as_str().unwrap()
                                                            }), PropertyType::Gv);
                                                        }
                                                    }
                                                    k if k == "destinations" => {
                                                        // Create destination object only. Relationship needs created later when all src objects created
                                                        for d in v.as_array().unwrap() {
                                                            let dst_o = self.create_object(VertexTypes::ConnectionDst);
                                                            self.add_object_properties(&dst_o, &json!({
                                                                GVID: format!("{}_{}_{}", &k, d.as_str().unwrap(), &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                GV_LABEL: d.as_str().unwrap()
                                                            }), PropertyType::Gv);
                                                            dst_objs.push(dst_o.id);
                                                        }
                                                    }
                                                    k if k == "tests" => {
                                                        for test in v.as_array().unwrap().iter() {
                                                            let t_o = self.create_object(VertexTypes::Test);
                                                            test_objs.push(t_o.id);

                                                            for (k, v) in test.as_object().unwrap().iter() {
                                                                match k {
                                                                    k if k == NAME => {
                                                                        let t_o_p = self.get_object_properties(&t_o).unwrap().props;
                                                                        let mut p = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                        p.insert(k.clone(), v.clone());
                                                                        self.add_object_properties(&t_o, &p, PropertyType::Base);
                                                                    }
                                                                    k if k == "module" => {
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
                                                                        let t_module = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("module").unwrap().as_str().unwrap();
                                                                        self.add_object_properties(&t_o, &json!({
                                                                            GVID: format!("{}_{}_{}", t_o.t.as_str(), t_name.replace("-", "_"), &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                            GV_LABEL: t_module
                                                                        }), PropertyType::Gv);
                                                                    }
                                                                    k if k == CI => {
                                                                        let mut t_o_p = self.get_object_properties(&t_o).unwrap().props;
                                                                        let mut p = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                        p.append(&mut json!({k: v.as_object().unwrap().clone()}).as_object().unwrap().clone());
                                                                        self.add_object_properties(&t_o, &p, PropertyType::Base);
                                                                    }
                                                                    k if k == "verifications" => {
                                                                        for v in v.as_array().unwrap().iter() {
                                                                            let v_o = self.create_object(VertexTypes::Verification);
                                                                            let v_name = v.as_object().unwrap().get(NAME).unwrap().as_str().unwrap();
                                                                            let v_module = v.as_object().unwrap().get("module").unwrap().as_str().unwrap();
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
                                                    _ => {}
                                                }
                                            }
                                            let src_objs = self.get_direct_neighbour_objects_by_identifier(&eps_o, VertexTypes::ConnectionSrc);
                                            for src in src_objs.iter() {
                                                //SRC -> DST REL
                                                for dst in dst_objs.iter() {
                                                    self.create_relationship(src, &self.get_object(*dst));
                                                }
                                                //SRC -> TEST REL
                                                for test in test_objs.iter() {
                                                    self.create_relationship(src,&self.get_object(*test));
                                                }
                                            }
                                            //DST -> TEST REL
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            //Rte module cfg
                            let r_p = self.get_object_properties(&r_o).unwrap().props;
                            let module = r_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("module").unwrap().as_str().unwrap();
                            let cfg = self.load_object_config(&VertexTypes::get_name_by_object(&r_o), &module);
                            error!("RTE MODULE CFG: {:#?}", &cfg);
                        }
                    }
                    &_ => {}
                }
            }
        }

        //Eut Stages Deploy
        let eut_stage_deploy = self.add_ci_stages(&ci, &self.config.eut.ci.stages.deploy, &VertexTypes::StageDeploy);

        //Rte Stages Deploy
        let rte_stage_deploy = self.add_ci_stages(&eut_stage_deploy.unwrap(), &self.config.rte.ci.stages.deploy, &VertexTypes::StageDeploy);

        //Feature Stages Deploy
        let features_stage_deploy = self.add_ci_stages(&rte_stage_deploy.unwrap(), &self.config.features.ci.stages.deploy, &VertexTypes::StageDeploy);

        //Feature Stages Destroy
        let mut stage_destroy: Option<Vertex> = None;
        let features = self.get_direct_neighbour_objects_by_identifier(&eut, VertexTypes::Feature);
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

        // Rtes
        /*let rtes = self.get_direct_neighbour_objects_by_identifier(&eut, VertexTypes::Rte);
        for rte in rtes.iter() {
            error!("RTE: {:?}", &rte);
            /*let rte_p = self.get_object_properties(&rte);
            error!("{:?} --> {:?}", &rte.t.as_str(), &rte_p);*/
        }*/


        /*for rte in eut_p.props.get(0).unwrap().value.get(EUT).unwrap().get(RTES).unwrap().as_array().unwrap().iter() {
            let r_o = self.create_object(VertexTypes::Rte);
            self.add_object_properties(&r_o, &rte, PropertyType::Base);
            let cfg = self.load_object_config(&VertexTypes::get(&r_o), &String::from(rte["module"].as_str()
                .unwrap()));
            self.add_object_properties(&r_o, &cfg, PropertyType::Module);
            self.create_relationship(&eut, &r_o);

            //Rte - Endpoints
            self.create_object(VertexTypes::Connections);
            for _ in rte[ENDPOINTS].as_array().unwrap().iter() {
                let ep_o = self.create_object(VertexTypes::Connection);

                //Rte - Endpoint - Src
                for _ in rte[ENDPOINTS].get(0).unwrap()["sources"].as_array().unwrap().iter() {
                    let ep_src_o = self.create_object(VertexTypes::ConnectionSrc);
                    self.create_relationship(&ep_o, &ep_src_o);

                    //Rte - Tests
                    for test in rte[ENDPOINTS].get(0).unwrap()["tests"].as_array().unwrap().iter() {
                        let t_o = self.create_object(VertexTypes::Test);
                        self.add_object_properties(&t_o, &test, PropertyType::Base);
                        let t_p = self.get_object_properties(&t_o);
                        let name = format!("{}-{}-{}", self.config.tests.ci.stages.deploy[0], &rte["module"].as_str()
                            .unwrap().replace("_", "-"), &t_p.props.get(0)
                            .unwrap().value["name"].as_str()
                            .unwrap());
                        let test_deploy_stage = self.add_ci_stage(rte_deploy_stage.as_ref()
                                                                      .unwrap(), &json!([name]), VertexTypes::StageDeploy);

                        //Rte - Verifications
                        for verification in test["verifications"].as_array().unwrap() {
                            let v_o = self.create_object(VertexTypes::Verification);
                            self.add_object_properties(&v_o, &verification, PropertyType::Base);
                            // let verification_p = self.get_object_properties(&v_o);
                            self.create_relationship(&t_o, &v_o);
                            let name = format!("{}-{}-{}", self.config.verifications.ci.stages.deploy[0], &rte["module"].as_str()
                                .unwrap().replace("_", "-"), &t_p.props.get(0)
                                .unwrap().value["name"].as_str().unwrap());
                            self.add_ci_stage(test_deploy_stage.as_ref().unwrap(), &json!([name]), VertexTypes::StageDeploy);
                        }
                    }

                    //Rte - Endpoint - Dst
                    for _ in rte[ENDPOINTS].get(0).unwrap()["destinations"].as_array().unwrap().iter() {
                        let ep_dst_o = self.create_object(VertexTypes::ConnectionDst);
                        self.create_relationship(&ep_src_o, &ep_dst_o);
                    }
                }
            }

            // Rte - Provider
            let p_o = self.create_object(VertexTypes::Provider);
            self.create_relationship(&r_o, &p_o);
            let r_p = self.get_object_properties(&r_o);
            for p in eut_p.props.get(PropertyType::Base.index()).unwrap().value[PropertyType::Base.name()][PROVIDER].as_array().unwrap().iter() {
                let mut o: Vertex = Vertex { id: Default::default(), t: Default::default() };
                match p.as_str().unwrap() {
                    "aws" => {
                        o = self.create_object(VertexTypes::ProviderAws)
                    }
                    "gcp" => {
                        o = self.create_object(VertexTypes::ProviderGcp)
                    }
                    "azure" => {
                        o = self.create_object(VertexTypes::ProviderAzure)
                    }
                    _ => {}
                }
                self.create_relationship(&p_o, &o);

                // Rte - Components
                let c_o = self.create_object(VertexTypes::Components);
                self.create_relationship(&o, &c_o);
                let rte_module_cfg = self.get_object_properties(&r_o);
                for p in eut_p.props.get(PropertyType::Base.index()).unwrap().value[PropertyType::Base.name()][PROVIDER].as_array().unwrap().iter() {
                    let cs_o = self.create_object(VertexTypes::ComponentSrc);
                    let ds_o = self.create_object(VertexTypes::ComponentDst);
                    self.add_object_properties(&cs_o, &rte_module_cfg.props.get(PropertyType::Module.index())
                        .unwrap().value[PROVIDER][p.as_str()
                        .unwrap()]["components"]["src"], PropertyType::Module);
                    self.add_object_properties(&ds_o, &rte_module_cfg.props.get(PropertyType::Module.index())
                        .unwrap().value[PROVIDER][p.as_str()
                        .unwrap()]["components"]["dst"], PropertyType::Module);
                    self.create_relationship(&c_o, &cs_o);
                    self.create_relationship(&c_o, &ds_o);
                }

                // Rte - Scripts
                let s_o = self.create_object(VertexTypes::Scripts);
                self.create_relationship(&o, &s_o);

                for script in r_p.props.get(PropertyType::Module.index()).unwrap().value["scripts"].as_array().unwrap().iter() {
                    match script["name"].as_str().unwrap() {
                        "apply" => {
                            let sa_o = self.create_object(VertexTypes::ScriptApply);
                            self.create_relationship(&s_o, &sa_o);
                            let s_path = &r_p.props.get(PropertyType::Module.index()).unwrap().value["scripts_path"];
                            let file = format!("{}/{}/{}/{}/{}/{}", self.config.project.root_path, self.config.rte.path, rte["module"].as_str()
                                .unwrap(), s_path.as_str()
                                                   .unwrap(), p.as_str()
                                                   .unwrap(), script["value"].as_str()
                                                   .unwrap());
                            let contents = std::fs::read_to_string(file).expect("panic reading script file");
                            let value = json!({
                                "apply": contents
                            });
                            self.add_object_properties(&sa_o, &value, PropertyType::Module);
                        }
                        _ => {}
                    }
                }
            }
        }*/

        project.id
    }

    fn object_map(key: &str) -> Value {
        let mut a = HashMap::new();
        a.insert(String::from("scripts"), json!({"name": {"o": "script", "ancestor": "scripts"}, "value": {"o": "value", "ancestor": "script"}}));
        a.insert(String::from("eut"), json!({"name": {"o": "name", "ancestor": "eut"}}));
        error!("{:?}", &a.get(key));
        a.get(key).unwrap().clone()
    }

    fn map(&self, key: &str, val: &Value, ancestor: &Vertex) {
        error!("KEY: {:?} -- VALUE: {:?}", &key, &val);

        match val {
            Null => {}
            Value::Bool(_) => {}
            Value::Number(_) => {}
            Value::String(_) => {
                let v_type = VertexTypes::get_type_by_key(key);
                let a_p = self.get_object_properties(&ancestor).unwrap().props;
                let r_str = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

                /*error!("################################# STRING BEFORE ###################################################");
                error!("KEY: {:?}", &key);
                error!("VALUE: {:?}", &val);
                error!("ANCESTOR: {:?}", &ancestor);
                error!("V_TYPE: {:?}", &v_type);
                error!("RANDOM: {}", r_str);
                error!("A_P: {:?}", &a_p);
                error!("################################# STRING BEFORE ###################################################\n");*/
                let mut y = a_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                y.insert(String::from(key), val.clone());
                self.add_object_properties(&ancestor, &y, PropertyType::Module);
                self.add_object_properties(&ancestor, &json!({GVID: format!("{}_{}", &ancestor.t.as_str(), &r_str)}), PropertyType::Gv);

                let a_p = self.get_object_properties(&ancestor).unwrap().props;
                error!("################################# STRING AFTER ###################################################");
                error!("KEY: {:?}", &key);
                error!("VALUE: {:?}", &val);
                error!("ANCESTOR: {:?}", &ancestor);
                error!("V_TYPE: {:?}", &v_type);
                error!("A_P: {:?}", &a_p);
                error!("################################# STRING AFTER ###################################################\n");
            }
            Value::Array(_) => {
                /*error!("################################# ARRAY ###################################################");
                error!("KEY: {:?}", key);
                error!("VALUE: {:?}", val);
                error!("ANCESTOR: {:?}", ancestor);
                error!("################################# ARRAY ###################################################\n");*/

                let mut it = val.as_array().unwrap().iter().peekable();
                while let Some(item) = it.next() {
                    error!("ITEM: {:?}", item);
                    error!("######################## ARRAY WHILE #######################################");
                    error!("KEY: {:?}", key);
                    error!("ITEM: {:?}", item);
                    error!("ANCESTOR: {:?}", ancestor);
                    error!("######################## ARRAY WHILE #######################################\n");
                    /*let o = self.create_object(VertexTypes::get_type_by_key(item.as_str().unwrap()));
                    self.create_relationship(&ancestor, &o);*/
                    self.map(key, item, ancestor);
                }
            }
            Value::Object(_) => {
                /*error!("################################# OBJECT ###################################################");
                error!("KEY: {:?}", key);
                error!("VALUE: {:?}", &val);
                error!("ANCESTOR: {:?}", &ancestor);
                error!("################################# OBJECT ###################################################\n");*/

                let mut it = val.as_object().unwrap().iter().peekable();
                while let Some(item) = it.next() {
                    let key = item.0;
                    let value = item.1;

                    /*error!("######################## OBJECT WHILE BEFORE #######################################");
                    error!("KEY: {:?}", key);
                    error!("VALUE: {:?}", value);
                    error!("ANCESTOR: {:?}", ancestor);
                    error!("VALUE_IS_STRING: {:?}", value.is_string());
                    error!("######################## OBJECT WHILE BEFORE #######################################\n");*/

                    if value.is_string() {
                        self.map(key, value, &ancestor);
                    } else {
                        let v_type = VertexTypes::get_type_by_key(&key);
                        error!("######################## OBJECT WHILE AFTER #######################################");
                        error!("KEY: {:?}", key);
                        error!("VALUE: {:?}", value);
                        error!("ANCESTOR: {:?}", ancestor);
                        error!("V_TYPE: {:?}", &v_type);
                        error!("######################## OBJECT WHILE AFTER #######################################\n");

                        if v_type != VertexTypes::None {
                            let o = self.create_object(v_type);
                            self.create_relationship(&ancestor, &o);
                            self.map(key, value, &o);
                        }
                    }
                }
            }
        }

        /*error!("####################################################################################");
        //error!("Map key <{:?}> with value <{:?}> and ancestor <{:?}>", &key, &val, &ancestor);
        let ancestor_p = self.get_object_properties(&ancestor).unwrap().props;
        //let name = &ancestor_p.get(PropertyType::Base.index()).unwrap().value[NAME].as_str().unwrap().replace("-", "_");
        match val {
            Null => {}
            Value::Bool(_) => {
                error!("WE GOT A BOOL");
            }
            Value::Number(_) => {}
            Value::String(_) => {
                //error!("String --> Key: {:?} -> value: {:?} --> Adding property...", &key, &val);
                let mut y = ancestor_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                y.insert(String::from(key), val.clone());
                self.add_object_properties(&ancestor, &y, PropertyType::Module);
                //error!("####################################################################################");
            }
            Value::Array(_) => {
                for item in val.as_array().unwrap().iter() {
                    match item {
                        Value::Object(_) => {
                            error!("Array --> Key: {:?} -> value: {:?} --> push to next level...", &key, &item);
                            self.map(&VertexTypes::get_by_key(key).name(), &item, &ancestor);
                        }
                        Value::String(_) => {
                            //error!("Array --> Key: {:?} -> value: {:?} --> Creating new object...", &key, &item);
                            let o = self.create_object(VertexTypes::get_by_key(key));
                            self.add_object_properties(&o, &json!({GVID: format!("{}_{}", key, item.as_str().unwrap())}), PropertyType::Gv);
                            self.create_relationship(&ancestor, &o);
                        }
                        _ => {}
                    }
                }
                // error!("####################################################################################");
            }
            Value::Object(_) => {
                error!("Object --> Key: {:?} -> value: {:?} --> Creating new object...", &key, &val);
                let o = self.create_object(VertexTypes::get_by_key(key));
                self.add_object_properties(&o, &val, PropertyType::Base);
                self.create_relationship(&ancestor, &o);
                error!("NEW OBJECT: {:?}", &o);
                error!("KEY: {:?}", &VertexTypes::get_by_key(key).name());
                error!("VALUE: {:?}", &val);
                error!("ANCESTOR: {:?}", &ancestor);
                for (k, v) in val.as_object().unwrap().iter() {
                    error!("FOUND INNER OBJECT WITH KEY: {:?} => VALUE: {:?}", &k, &v);
                    self.map(&VertexTypes::get_by_key(k).name(), &v, &o);
                }

                match val.as_object().unwrap().get("module") {
                    Some(p) => {
                        self.add_object_properties(&o, &json!({GVID: format!("{}_{}", key, p.as_str().unwrap())}), PropertyType::Gv);
                    }
                    None => {
                        match val.as_object().unwrap().get("name") {
                            Some(p) => {
                                self.add_object_properties(&o, &json!({GVID: format!("{}_{}", key, p.as_str().unwrap())}), PropertyType::Gv);
                            }
                            None => {}
                        }
                    }
                }
                error!("####################################################################################");
            }
        }*/
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
                    .unwrap(), indradb::Json::new(v.clone()));
            }
            PropertyType::Base => {
                p = BulkInsertItem::VertexProperty(object.id, Identifier::new(PROPERTY_TYPE_BASE)
                    .unwrap(), indradb::Json::new(v.clone()));
            }
            PropertyType::Module => {
                p = BulkInsertItem::VertexProperty(object.id, Identifier::new(PROPERTY_TYPE_MODULE)
                    .unwrap(), indradb::Json::new(v.clone()));
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
                    .unwrap(), indradb::Json::new(v.clone()));
            }
            PropertyType::Base => {
                p = BulkInsertItem::EdgeProperty(object.clone(), Identifier::new(PROPERTY_TYPE_BASE)
                    .unwrap(), indradb::Json::new(v.clone()));
            }
            PropertyType::Module => {
                p = BulkInsertItem::EdgeProperty(object.clone(), Identifier::new(PROPERTY_TYPE_MODULE)
                    .unwrap(), indradb::Json::new(v.clone()));
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

    fn get_direct_neighbour_objects_by_identifier(&self, start: &Vertex, identifier: VertexTypes) -> Vec<Vertex> {
        info!("Get direct neighbors of <{}>...", start.t.as_str());
        let mut rvq = RangeVertexQuery::new();
        rvq.t = Option::from(Identifier::new(identifier.name()).unwrap());
        rvq.limit = 20;
        rvq.start_id = Option::from(start.id);
        let result = indradb::util::extract_vertices(self.regression.get(rvq).unwrap()).unwrap();
        info!("Get direct neighbors of <{}> -> Done.", start.t.as_str());
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
                error!("Error in properties query");
                None
            }
        };
    }

    #[allow(dead_code)]
    fn get_relationship(&self, a: &Vertex, b: &Vertex) -> Vec<Edge> {
        info!("Get relationship for <{}> and <{}>...", &a.t.as_str(), &b.t.as_str());
        let i = self.create_relationship_identifier(&a, &b);
        let e = Edge::new(a.id, i, b.id);
        let r: Vec<indradb::QueryOutputValue> = self.regression.get(indradb::SpecificEdgeQuery::single(e.clone())).unwrap();
        let e = indradb::util::extract_edges(r).unwrap();
        info!("Get relationship for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        e
    }

    /*fn build_context(&self, id: Uuid) -> Context {
        info!("Build render context...");
        let project = self.get_object(id);
        let project_p = self.get_object_properties(&project);

        let eut = self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::Eut);
        let eut_p = self.get_object_properties(&eut);

        let _features = self.get_direct_neighbour_objects_by_identifier(&eut, VertexTypes::Feature);
        let mut features = Vec::new();
        for feature in _features.iter() {
            let feature_p = self.get_object_properties(&feature);
            features.push(feature_p.props.get(0).unwrap().value.clone())
        }

        let _rtes = self.get_direct_neighbour_objects_by_identifier(&eut, VertexTypes::Rte);
        let mut rtes = Vec::new();
        error!("##################################################################");
        for rte in _rtes.iter() {
            let rte_p = self.get_object_properties(&rte);

            for ep in rte_p.props.get(PropertyType::Base.index()).unwrap().value[ENDPOINTS].as_array().unwrap().iter() {
                for source in ep["sources"].as_array().unwrap().iter() {
                    error!("{} CLIENT: {:?}", &source.as_str().unwrap(), &rte_p.props.get(PropertyType::Module.index())
                        .unwrap().value[PROVIDER][source.as_str().unwrap()]["components"]["src"]);
                }
                for destination in ep["destinations"].as_array().unwrap().iter() {
                    error!("{} SERVER: {:?}", &destination.as_str().unwrap(), &rte_p.props.get(PropertyType::Module.index())
                        .unwrap().value[PROVIDER][destination.as_str()
                            .unwrap()]["components"]["dst"]);
                }
            }
            let data = json!({
                "cfg": &rte_p.props.get(PropertyType::Base.index()).unwrap().value.clone(),
                "module": &rte_p.props.get(PropertyType::Module.index()).unwrap().value.clone()
            });
            rtes.push(data);
        }
        error!("##################################################################");
        let mut stages: Vec<String> = Vec::new();
        let mut deploy_stages: Vec<String> = Vec::new();
        let mut destroy_stages: Vec<String> = Vec::new();
        let s_deploy = self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::StageDeploy);
        let s_destroy = self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::StageDestroy);

        for _stage in self.get_direct_neighbour_objects_by_identifier(&s_deploy, VertexTypes::StageDeploy).iter() {
            let _p = &self.get_object_properties(&_stage);
            let p = &_p.props.get(0).unwrap().value;

            for stage in p.as_array().unwrap().iter() {
                deploy_stages.push(stage.as_str().unwrap().to_string());
            }
        }

        for _stage in self.get_direct_neighbour_objects_by_identifier(&s_destroy, VertexTypes::StageDestroy).iter() {
            let _p = &self.get_object_properties(&_stage);
            let p = &_p.props.get(0).unwrap().value;

            for stage in p.as_array().unwrap().iter() {
                destroy_stages.push(stage.as_str().unwrap().to_string());
            }
        }

        stages.append(&mut deploy_stages);
        stages.append(&mut destroy_stages);

        let mut context = Context::new();
        context.insert(EUT, &eut_p.props[0].value[EUT]);
        context.insert(RTES, &rtes);
        context.insert("config", &self.config);
        context.insert("stages", &stages);
        context.insert(FEATURES, &features);
        context.insert("project", &project_p.props[0].value);

        // info!("{:#?}", context);
        info!("Build render context -> Done.");
        context
    }*/

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
            Some(n) => {
                let nodes = &_nodes.unwrap();
                let mut items = Vec::new();

                for n in nodes.iter() {
                    let n_p = self.get_object_properties(&self.get_object(n.id));
                    match n_p {
                        Some(p) => {
                            let gv_id = p.props.get(PropertyType::Gv.index()).unwrap().value[GVID].as_str();
                            let gv_label = p.props.get(PropertyType::Gv.index()).unwrap().value[GV_LABEL].as_str();
                            match gv_id {
                                Some(p) => items.push(json!({"id": &gv_id.unwrap(), "label": &gv_label, "shape": "circle"})),
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
            Some(e) => {
                let edges = &_edges.unwrap();
                let mut items = Vec::new();

                for (i, e) in edges.iter().enumerate() {
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
    let root = r.init();
    // r.to_file(&r.render(&r.build_context(root)), PIPELINE_FILE_NAME);
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
