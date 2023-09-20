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

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::option::Option;
use std::string::ToString;
use std::vec;

use clap::Parser;
use indradb;
use indradb::{AllVertexQuery, BulkInsertItem, Edge, Identifier, Json, QueryExt, Vertex, VertexProperties};
use lazy_static::lazy_static;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, to_value, Value};
use serde_json::Value::Null;
use tera::{Context, Tera};
use uuid::Uuid;

const CONFIG_FILE_NAME: &str = "config.json";

// KEYS
const KEY_CI: &str = "ci";
const KEY_EUT: &str = "eut";
const KEY_RTE: &str = "rte";
const KEY_SRC: &str = "src";
const KEY_DST: &str = "dst";
const KEY_FILE: &str = "file";
const KEY_RTES: &str = "rtes";
const KEY_TEST: &str = "test";
const KEY_TESTS: &str = "tests";
const KEY_GVID: &str = "id";
const KEY_NAME: &str = "name";
const KEY_APPLY: &str = "apply";
const KEY_SITES: &str = "sites";
const KEY_SHARE: &str = "share";
const KEY_COUNT: &str = "count";
const KEY_CONFIG: &str = "config";
const KEY_STAGES: &str = "stages";
const KEY_MODULE: &str = "module";
const KEY_SCRIPT: &str = "script";
const KEY_RELEASE: &str = "release";
const KEY_SCRIPTS: &str = "scripts";
const KEY_SOURCE: &str = "source";
const KEY_PROJECT: &str = "project";
const KEY_FEATURE: &str = "feature";
const KEY_GV_LABEL: &str = "label";
const KEY_FEATURES: &str = "features";
const KEY_PROVIDER: &str = "provider";
const KEY_PROVIDERS: &str = "providers";
const KEY_COMPONENT: &str = "component";
const KEY_COMPONENTS: &str = "components";
const KEY_CONNECTION: &str = "connection";
const KEY_CONNECTIONS: &str = "connections";
const KEY_VERIFICATION: &str = "verification";
const KEY_SCRIPTS_PATH: &str = "scripts_path";
const KEY_VERIFICATIONS: &str = "verifications";

// miscellaneous
const PIPELINE_FILE_NAME: &str = ".gitlab-ci.yml";
const PIPELINE_TEMPLATE_FILE_NAME: &str = ".gitlab-ci.yml.tpl";

const PROPERTY_TYPE_GV: &str = "gv";
const PROPERTY_TYPE_BASE: &str = "base";
const PROPERTY_TYPE_MODULE: &str = "module";

// Key order
const EUT_KEY_ORDER: &[&str] = &["ci", "features", "name", "provider", "release", "sites", "rtes", "scripts", "scripts_path"];

//Objects types
const VERTEX_TYPE_CI: &str = "ci";
const VERTEX_TYPE_EUT: &str = "eut";
const VERTEX_TYPE_RTE: &str = "rte";
const VERTEX_TYPE_RTES: &str = "rtes";
const VERTEX_TYPE_TEST: &str = "test";
const VERTEX_TYPE_NONE: &str = "none";
const VERTEX_TYPE_SITE: &str = "site";
const VERTEX_TYPE_SITES: &str = "sites";
const VERTEX_TYPE_SHARE: &str = "share";
const VERTEX_TYPE_SCRIPT: &str = "script";
const VERTEX_TYPE_SCRIPTS: &str = "scripts";
const VERTEX_TYPE_PROJECT: &str = "project";
const VERTEX_TYPE_FEATURE: &str = "feature";
const VERTEX_TYPE_FEATURES: &str = "features";
const VERTEX_TYPE_PROVIDERS: &str = "providers";
const VERTEX_TYPE_COLLECTOR: &str = "collector";
const VERTEX_TYPE_COMPONENTS: &str = "components";
const VERTEX_TYPE_CONNECTION: &str = "connection";
const VERTEX_TYPE_CONNECTIONS: &str = "connections";
const VERTEX_TYPE_VERIFICATION: &str = "verification";
const VERTEX_TYPE_EUT_PROVIDER: &str = "eut_provider";
const VERTEX_TYPE_RTE_PROVIDER: &str = "rte_provider";
const VERTEX_TYPE_STAGE_DEPLOY: &str = "deploy";
const VERTEX_TYPE_STAGE_DESTROY: &str = "stage_destroy";
const VERTEX_TYPE_COMPONENT_SRC: &str = "component_src";
const VERTEX_TYPE_COMPONENT_DST: &str = "component_dst";
const VERTEX_TYPE_CONNECTION_SRC: &str = "connection_src";
const VERTEX_TYPE_CONNECTION_DST: &str = "connection_dst";

// Rel type
const EDGE_TYPE_HAS: &str = "has";
const EDGE_TYPE_RUNS: &str = "runs";
const EDGE_TYPE_NEEDS: &str = "needs";
const EDGE_TYPE_HAS_CI: &str = "has_ci";
const EDGE_TYPE_HAS_EUT: &str = "has_eut";
const EDGE_TYPE_HAS_SITE: &str = "has_site";
const EDGE_TYPE_HAS_SITES: &str = "has_sites";
const EDGE_TYPE_NEEDS_SHARE: &str = "needs_share";
const EDGE_TYPE_USES_RTES: &str = "uses_rtes";
const EDGE_TYPE_NEXT_STAGE: &str = "next_stage";
const EDGE_TYPE_REFERS_SITE: &str = "refers_site";
const EDGE_TYPE_HAS_FEATURE: &str = "has_feature";
const EDGE_TYPE_HAS_FEATURES: &str = "has_features";
const EDGE_TYPE_PROVIDES_RTE: &str = "provides_rte";
const EDGE_TYPE_HAS_PROVIDERS: &str = "has_providers";
const EDGE_TYPE_USES_PROVIDER: &str = "uses_provider";
const EDGE_TYPE_NEEDS_PROVIDER: &str = "needs_provider";
const EDGE_TYPE_HAS_COMPONENTS: &str = "has_components";
const EDGE_TYPE_HAS_CONNECTION: &str = "has_connection";
const EDGE_TYPE_HAS_CONNECTIONS: &str = "has_connections";
const EDGE_TYPE_PROVIDES_PROVIDER: &str = "provides_provider";
const EDGE_TYPE_HAS_COMPONENT_SRC: &str = "has_component_src";
const EDGE_TYPE_HAS_COMPONENT_DST: &str = "has_component_dst";
const EDGE_TYPE_HAS_CONNECTION_SRC: &str = "has_connection_src";
const EDGE_TYPE_HAS_CONNECTION_DST: &str = "has_connection_dst";
const EDGE_TYPE_HAS_DEPLOY_STAGES: &str = "has_deploy_stages";
const EDGE_TYPE_HAS_DESTROY_STAGES: &str = "has_destroy_stages";

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
    Site,
    Sites,
    Share,
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
    Runs,
    Needs,
    HasCi,
    HasEut,
    HasSite,
    HasSites,
    UsesRtes,
    NextStage,
    RefersSite,
    NeedsShare,
    HasFeature,
    HasFeatures,
    ProvidesRte,
    UsesProvider,
    HasProviders,
    NeedsProvider,
    HasComponents,
    HasConnection,
    HasConnections,
    HasDeployStages,
    HasComponentSrc,
    HasComponentDst,
    ProvidesProvider,
    HasConnectionSrc,
    HasConnectionDst,
    HasDestroyStages,
}

impl VertexTypes {
    fn name(&self) -> &'static str {
        match *self {
            VertexTypes::Ci => VERTEX_TYPE_CI,
            VertexTypes::Eut => VERTEX_TYPE_EUT,
            VertexTypes::Rte => VERTEX_TYPE_RTE,
            VertexTypes::Rtes => VERTEX_TYPE_RTES,
            VertexTypes::Test => VERTEX_TYPE_TEST,
            VertexTypes::Site => VERTEX_TYPE_SITE,
            VertexTypes::Sites => VERTEX_TYPE_SITES,
            VertexTypes::Share => VERTEX_TYPE_SHARE,
            VertexTypes::Script => VERTEX_TYPE_SCRIPT,
            VertexTypes::Scripts => VERTEX_TYPE_SCRIPTS,
            VertexTypes::Project => VERTEX_TYPE_PROJECT,
            VertexTypes::Feature => VERTEX_TYPE_FEATURE,
            VertexTypes::Features => VERTEX_TYPE_FEATURES,
            VertexTypes::Collector => VERTEX_TYPE_COLLECTOR,
            VertexTypes::Providers => VERTEX_TYPE_PROVIDERS,
            VertexTypes::Connection => VERTEX_TYPE_CONNECTION,
            VertexTypes::Components => VERTEX_TYPE_COMPONENTS,
            VertexTypes::Connections => VERTEX_TYPE_CONNECTIONS,
            VertexTypes::EutProvider => VERTEX_TYPE_EUT_PROVIDER,
            VertexTypes::RteProvider => VERTEX_TYPE_RTE_PROVIDER,
            VertexTypes::StageDeploy => VERTEX_TYPE_STAGE_DEPLOY,
            VertexTypes::StageDestroy => VERTEX_TYPE_STAGE_DESTROY,
            VertexTypes::Verification => VERTEX_TYPE_VERIFICATION,
            VertexTypes::ComponentSrc => VERTEX_TYPE_COMPONENT_SRC,
            VertexTypes::ComponentDst => VERTEX_TYPE_COMPONENT_DST,
            VertexTypes::ConnectionSrc => VERTEX_TYPE_CONNECTION_SRC,
            VertexTypes::ConnectionDst => VERTEX_TYPE_CONNECTION_DST,
            VertexTypes::None => VERTEX_TYPE_NONE,
        }
    }

    fn get_name_by_object(object: &Vertex) -> &'static str {
        match object.t.as_str() {
            VERTEX_TYPE_CI => VertexTypes::Ci.name(),
            VERTEX_TYPE_RTE => VertexTypes::Rte.name(),
            VERTEX_TYPE_EUT => VertexTypes::Eut.name(),
            VERTEX_TYPE_RTES => VertexTypes::Rtes.name(),
            VERTEX_TYPE_SITE => VertexTypes::Site.name(),
            VERTEX_TYPE_TEST => VertexTypes::Test.name(),
            VERTEX_TYPE_SITES => VertexTypes::Sites.name(),
            VERTEX_TYPE_SHARE => VertexTypes::Share.name(),
            VERTEX_TYPE_SCRIPT => VertexTypes::Script.name(),
            VERTEX_TYPE_SCRIPTS => VertexTypes::Scripts.name(),
            VERTEX_TYPE_PROJECT => VertexTypes::Project.name(),
            VERTEX_TYPE_FEATURE => VertexTypes::Feature.name(),
            VERTEX_TYPE_PROVIDERS => VertexTypes::Providers.name(),
            VERTEX_TYPE_COLLECTOR => VertexTypes::Collector.name(),
            VERTEX_TYPE_CONNECTION => VertexTypes::Connection.name(),
            VERTEX_TYPE_COMPONENTS => VertexTypes::Components.name(),
            VERTEX_TYPE_CONNECTIONS => VertexTypes::Connections.name(),
            VERTEX_TYPE_VERIFICATION => VertexTypes::Verification.name(),
            VERTEX_TYPE_STAGE_DEPLOY => VertexTypes::StageDeploy.name(),
            VERTEX_TYPE_EUT_PROVIDER => VertexTypes::EutProvider.name(),
            VERTEX_TYPE_RTE_PROVIDER => VertexTypes::RteProvider.name(),
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
            VERTEX_TYPE_SITE => VertexTypes::Site,
            VERTEX_TYPE_SITES => VertexTypes::Sites,
            VERTEX_TYPE_SHARE => VertexTypes::Share,
            VERTEX_TYPE_SCRIPT => VertexTypes::Script,
            VERTEX_TYPE_SCRIPTS => VertexTypes::Scripts,
            VERTEX_TYPE_PROJECT => VertexTypes::Project,
            VERTEX_TYPE_FEATURE => VertexTypes::Feature,
            VERTEX_TYPE_FEATURES => VertexTypes::Features,
            VERTEX_TYPE_PROVIDERS => VertexTypes::Providers,
            VERTEX_TYPE_COLLECTOR => VertexTypes::Collector,
            VERTEX_TYPE_CONNECTION => VertexTypes::Connection,
            VERTEX_TYPE_COMPONENTS => VertexTypes::Components,
            VERTEX_TYPE_CONNECTIONS => VertexTypes::Connections,
            VERTEX_TYPE_VERIFICATION => VertexTypes::Verification,
            VERTEX_TYPE_EUT_PROVIDER => VertexTypes::EutProvider,
            VERTEX_TYPE_RTE_PROVIDER => VertexTypes::RteProvider,
            VERTEX_TYPE_STAGE_DEPLOY => VertexTypes::StageDeploy,
            VERTEX_TYPE_STAGE_DESTROY => VertexTypes::StageDestroy,
            VERTEX_TYPE_COMPONENT_SRC => VertexTypes::ComponentSrc,
            VERTEX_TYPE_COMPONENT_DST => VertexTypes::ComponentDst,
            VERTEX_TYPE_CONNECTION_SRC => VertexTypes::ConnectionSrc,
            VERTEX_TYPE_CONNECTION_DST => VertexTypes::ConnectionDst,
            _ => VertexTypes::None
        }
    }
}

impl EdgeTypes {
    fn name(&self) -> &'static str {
        match *self {
            EdgeTypes::Has => EDGE_TYPE_HAS,
            EdgeTypes::Runs => EDGE_TYPE_RUNS,
            EdgeTypes::Needs => EDGE_TYPE_NEEDS,
            EdgeTypes::HasCi => EDGE_TYPE_HAS_CI,
            EdgeTypes::HasEut => EDGE_TYPE_HAS_EUT,
            EdgeTypes::HasSite => EDGE_TYPE_HAS_SITE,
            EdgeTypes::HasSites => EDGE_TYPE_HAS_SITES,
            EdgeTypes::UsesRtes => EDGE_TYPE_USES_RTES,
            EdgeTypes::NextStage => EDGE_TYPE_NEXT_STAGE,
            EdgeTypes::RefersSite => EDGE_TYPE_REFERS_SITE,
            EdgeTypes::HasFeature => EDGE_TYPE_HAS_FEATURE,
            EdgeTypes::NeedsShare => EDGE_TYPE_NEEDS_SHARE,
            EdgeTypes::HasFeatures => EDGE_TYPE_HAS_FEATURES,
            EdgeTypes::ProvidesRte => EDGE_TYPE_PROVIDES_RTE,
            EdgeTypes::HasProviders => EDGE_TYPE_HAS_PROVIDERS,
            EdgeTypes::UsesProvider => EDGE_TYPE_USES_PROVIDER,
            EdgeTypes::NeedsProvider => EDGE_TYPE_NEEDS_PROVIDER,
            EdgeTypes::HasComponents => EDGE_TYPE_HAS_COMPONENTS,
            EdgeTypes::HasConnection => EDGE_TYPE_HAS_CONNECTION,
            EdgeTypes::HasConnections => EDGE_TYPE_HAS_CONNECTIONS,
            EdgeTypes::HasComponentSrc => EDGE_TYPE_HAS_COMPONENT_SRC,
            EdgeTypes::HasComponentDst => EDGE_TYPE_HAS_COMPONENT_DST,
            EdgeTypes::HasDeployStages => EDGE_TYPE_HAS_DEPLOY_STAGES,
            EdgeTypes::HasDestroyStages => EDGE_TYPE_HAS_DESTROY_STAGES,
            EdgeTypes::ProvidesProvider => EDGE_TYPE_PROVIDES_PROVIDER,
            EdgeTypes::HasConnectionSrc => EDGE_TYPE_HAS_CONNECTION_SRC,
            EdgeTypes::HasConnectionDst => EDGE_TYPE_HAS_CONNECTION_DST,
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
        map.insert(VertexTuple(VertexTypes::Project.name().to_string(), VertexTypes::Eut.name().to_string()), EdgeTypes::HasEut.name());
        map.insert(VertexTuple(VertexTypes::Project.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::HasCi.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Features.name().to_string()), EdgeTypes::HasFeatures.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Rtes.name().to_string()), EdgeTypes::UsesRtes.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::HasCi.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Providers.name().to_string()), EdgeTypes::HasProviders.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Scripts.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Sites.name().to_string()), EdgeTypes::HasSites.name());
        map.insert(VertexTuple(VertexTypes::Rtes.name().to_string(), VertexTypes::Rte.name().to_string()), EdgeTypes::ProvidesRte.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Providers.name().to_string()), EdgeTypes::NeedsProvider.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Connections.name().to_string()), EdgeTypes::HasConnections.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Collector.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Scripts.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Features.name().to_string()), EdgeTypes::Needs.name());
        map.insert(VertexTuple(VertexTypes::Sites.name().to_string(), VertexTypes::Site.name().to_string()), EdgeTypes::HasSite.name());
        map.insert(VertexTuple(VertexTypes::Site.name().to_string(), VertexTypes::EutProvider.name().to_string()), EdgeTypes::UsesProvider.name());
        map.insert(VertexTuple(VertexTypes::Providers.name().to_string(), VertexTypes::EutProvider.name().to_string()), EdgeTypes::ProvidesProvider.name());
        map.insert(VertexTuple(VertexTypes::Providers.name().to_string(), VertexTypes::RteProvider.name().to_string()), EdgeTypes::ProvidesProvider.name());
        map.insert(VertexTuple(VertexTypes::RteProvider.name().to_string(), VertexTypes::Components.name().to_string()), EdgeTypes::HasComponents.name());
        map.insert(VertexTuple(VertexTypes::RteProvider.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::HasCi.name());
        map.insert(VertexTuple(VertexTypes::RteProvider.name().to_string(), VertexTypes::Share.name().to_string()), EdgeTypes::NeedsShare.name());
        map.insert(VertexTuple(VertexTypes::Components.name().to_string(), VertexTypes::ComponentSrc.name().to_string()), EdgeTypes::HasComponentSrc.name());
        map.insert(VertexTuple(VertexTypes::Components.name().to_string(), VertexTypes::ComponentDst.name().to_string()), EdgeTypes::HasComponentDst.name());
        map.insert(VertexTuple(VertexTypes::Connections.name().to_string(), VertexTypes::Connection.name().to_string()), EdgeTypes::HasConnection.name());
        map.insert(VertexTuple(VertexTypes::Connection.name().to_string(), VertexTypes::ConnectionSrc.name().to_string()), EdgeTypes::HasConnectionSrc.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::ConnectionDst.name().to_string()), EdgeTypes::HasConnectionDst.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::Test.name().to_string()), EdgeTypes::Runs.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::ComponentSrc.name().to_string()), EdgeTypes::HasComponentSrc.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::Site.name().to_string()), EdgeTypes::RefersSite.name());
        map.insert(VertexTuple(VertexTypes::ConnectionDst.name().to_string(), VertexTypes::ComponentDst.name().to_string()), EdgeTypes::HasComponentDst.name());
        map.insert(VertexTuple(VertexTypes::ConnectionDst.name().to_string(), VertexTypes::Site.name().to_string()), EdgeTypes::RefersSite.name());
        map.insert(VertexTuple(VertexTypes::Test.name().to_string(), VertexTypes::Verification.name().to_string()), EdgeTypes::Needs.name());
        map.insert(VertexTuple(VertexTypes::Test.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::HasCi.name());
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::HasDeployStages.name());
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::HasDestroyStages.name());
        map.insert(VertexTuple(VertexTypes::StageDeploy.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::NextStage.name());
        map.insert(VertexTuple(VertexTypes::StageDestroy.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::NextStage.name());
        map.insert(VertexTuple(VertexTypes::Features.name().to_string(), VertexTypes::Feature.name().to_string()), EdgeTypes::HasFeature.name());
        map.insert(VertexTuple(VertexTypes::Scripts.name().to_string(), VertexTypes::Script.name().to_string()), EdgeTypes::Has.name());
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
    #[arg(long)]
    write_ci: bool,
    /// Export data to json file
    #[arg(long)]
    write_json: bool,
    /// Render CI pipline file
    #[arg(short, long)]
    render_ci: bool,
    /// Write to GraphViz file
    #[arg(long)]
    write_gv: bool,
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

#[derive(Deserialize, Serialize, Clone, Debug)]
struct RegressionConfigProjectVars {
    file: String,
    path: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct RegressionConfigProject {
    name: String,
    templates: String,
    root_path: String,
    vars: RegressionConfigProjectVars,
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
struct FeatureRenderContext {
    ci: Map<String, Value>,
    eut: String,
    name: String,
    release: String,
    scripts: Vec<HashMap<String, Vec<String>>>,
}

#[derive(Serialize, Debug)]
struct EutSiteRenderContext {
    job: String,
    name: String,
    index: usize,
    scripts: Vec<HashMap<String, Vec<String>>>,
    provider: String,
}

#[derive(Serialize, Debug)]
struct EutRenderContext {
    base: Map<String, Value>,
    sites: Vec<EutSiteRenderContext>,
    module: Map<String, Value>,
    project: RegressionConfigProject,
    provider: Vec<String>,
}

#[derive(Serialize, Debug)]
struct RteProviderShareRenderContext {
    job: String,
    rte: String,
    eut: String,
    provider: String,
    scripts: Vec<HashMap<String, Vec<String>>>,
}

#[derive(Serialize, Debug)]
struct RteCiRenderContext {
    timeout: Value,
    variables: Value,
}

#[derive(Serialize, Debug)]
struct RteVerificationRenderContext {
    ci: Map<String, Value>,
    test: String,
    rte: String,
    job: String,
    name: String,
    module: String,
    scripts: Vec<HashMap<String, Vec<String>>>,
}

#[derive(Serialize, Debug)]
struct RteTestRenderContext {
    ci: Map<String, Value>,
    rte: String,
    job: String,
    name: String,
    module: String,
    provider: String,
    scripts: Vec<HashMap<String, Vec<String>>>,
    verifications: Vec<RteVerificationRenderContext>,
}

#[derive(Serialize, Debug)]
struct RteComponentRenderContext {
    job: String,
    rte: String,
    name: String,
    site: String,
    scripts: Vec<HashMap<String, Vec<String>>>,
    provider: String,
}

#[derive(Serialize, Debug)]
struct RteRenderContext {
    ci: HashMap<String, RteCiRenderContext>,
    tests: Vec<RteTestRenderContext>,
    shares: Vec<RteProviderShareRenderContext>,
    components: Vec<RteComponentRenderContext>,
}

#[derive(Serialize, Debug)]
struct EutFeatureCiRenderContext {
    timeout: String,
}

#[derive(Serialize, Debug)]
struct EutFeatureRenderContext {
    ci: EutFeatureCiRenderContext,
    name: String,
    release: String,
    scripts: HashMap<String, String>,
}

#[derive(Serialize, Debug)]
struct ScriptEutRenderContext {
    name: Option<String>,
    site: Option<String>,
    index: Option<usize>,
    project: RegressionConfigProject,
    release: Option<String>,
    provider: Option<String>,
}

impl ScriptEutRenderContext {
    pub fn new(project: RegressionConfigProject) -> Self {
        Self {
            project,
            name: None,
            site: None,
            index: None,
            release: None,
            provider: None,
        }
    }

    pub fn render_script(&self, context: &ScriptEutRenderContext, input: &String) -> String {
        info!("Render regression pipeline file eut script section...");
        let ctx = Context::from_serialize(context);
        let rendered = Tera::one_off(input, &ctx.unwrap(), true).unwrap();
        info!("Render regression pipeline file eut script section -> Done.");
        rendered
    }
}

#[derive(Serialize, Debug)]
struct ScriptFeatureRenderContext {
    eut: String,
    name: Option<String>,
    release: Option<String>,
    provider: Option<Vec<String>>,
}

impl ScriptFeatureRenderContext {
    pub fn new(eut: String) -> Self {
        Self {
            eut,
            name: None,
            release: None,
            provider: None,
        }
    }

    pub fn render_script(&self, context: &ScriptFeatureRenderContext, input: &String) -> String {
        info!("Render regression pipeline file feature script section...");
        let ctx = Context::from_serialize(context);
        let rendered = Tera::one_off(input, &ctx.unwrap(), true).unwrap();
        info!("Render regression pipeline file feature script section -> Done.");
        rendered
    }
}

#[derive(Serialize, Debug)]
struct ScriptVerificationRenderContext {
    rte: Option<String>,
    name: Option<String>,
    module: Option<String>,
    provider: String,
    test_name: Option<String>,
    test_module: Option<String>,
}

impl ScriptVerificationRenderContext {
    pub fn new(provider: String) -> Self {
        Self {
            provider,
            rte: None,
            name: None,
            module: None,
            test_name: None,
            test_module: None,
        }
    }

    pub fn render_script(&self, context: &ScriptVerificationRenderContext, input: &String) -> String {
        info!("Render regression pipeline file verification script section...");
        let ctx = Context::from_serialize(context);
        let rendered = Tera::one_off(input, &ctx.unwrap(), true).unwrap();
        info!("Render regression pipeline file verification script section -> Done.");
        rendered
    }
}

#[derive(Serialize, Debug)]
struct ScriptTestRenderContext {
    rte: Option<String>,
    name: Option<String>,
    module: Option<String>,
    provider: String,
}

impl ScriptTestRenderContext {
    pub fn new(provider: String) -> Self {
        Self {
            provider,
            rte: None,
            name: None,
            module: None,
        }
    }

    pub fn render_script(&self, context: &ScriptTestRenderContext, input: &String) -> String {
        info!("Render regression pipeline file test script section...");
        let ctx = Context::from_serialize(context);
        let rendered = Tera::one_off(input, &ctx.unwrap(), true).unwrap();
        info!("Render regression pipeline file test script section -> Done.");
        rendered
    }
}

#[derive(Serialize, Debug)]
struct ScriptRteRenderContext {
    eut: Option<String>,
    rte: Option<String>,
    site: Option<String>,
    release: Option<String>,
    project: Option<RegressionConfigProject>,
    provider: String,
    destinations: Option<HashSet<String>>,
}

impl ScriptRteRenderContext {
    pub fn new(provider: String) -> Self {
        Self {
            provider,
            eut: None,
            rte: None,
            site: None,
            release: None,
            project: None,
            destinations: None,
        }
    }

    pub fn render_script(&self, context: &ScriptRteRenderContext, input: &String) -> String {
        info!("Render regression pipeline file rte script section...");
        let ctx = Context::from_serialize(context);
        let rendered = Tera::one_off(input, &ctx.unwrap(), true).unwrap();
        info!("Render regression pipeline file rte script section -> Done.");
        rendered
    }
}

#[derive(Serialize, Debug)]
struct ScriptRteProviderShareDataRenderContext {
    name: String,
    index: usize,
    has_client: bool,
    has_server: bool,
}

#[derive(Serialize, Debug)]
struct ScriptRteProviderShareRenderContext {
    eut: Option<String>,
    rte: Option<String>,
    data: Option<String>,
    vars: Option<RegressionConfigProjectVars>,
    counter: Option<usize>,
    project: String,
    provider: Option<String>,
}

impl ScriptRteProviderShareRenderContext {
    pub fn new(project: String) -> Self {
        Self {
            project,
            eut: None,
            rte: None,
            vars: None,
            data: None,
            counter: None,
            provider: None,
        }
    }

    pub fn render_script(&self, context: &ScriptRteProviderShareRenderContext, input: &String) -> String {
        info!("Render regression pipeline file rte provider share script section...");
        let ctx = Context::from_serialize(context);
        let rendered = Tera::one_off(input, &ctx.unwrap(), false).unwrap();
        info!("Render regression pipeline file rte provider share script section -> Done.");
        rendered
    }
}

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
        context.insert(KEY_EUT, &cfg.eut);
        context.insert(KEY_RTE, &cfg.rte);
        context.insert(KEY_TESTS, &cfg.tests);
        context.insert(KEY_PROJECT, &cfg.project);
        context.insert(KEY_FEATURES, &cfg.features);
        context.insert("collector", &cfg.collector);
        context.insert(KEY_VERIFICATIONS, &cfg.verifications);

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
            KEY_EUT => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.eut.path, module, CONFIG_FILE_NAME);
            }
            KEY_RTE => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.rte.path, module, CONFIG_FILE_NAME);
            }
            KEY_FEATURE => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.features.path, module, CONFIG_FILE_NAME);
            }
            KEY_TEST => {
                file = format!("{}/{}/{}/{}", self.config.project.root_path, self.config.tests.path, module, CONFIG_FILE_NAME);
            }
            KEY_VERIFICATION => {
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

    fn add_ci_stages(&self, ancestor: &Vertex, stages: &Vec<String>, object_type: &VertexTypes) -> Option<Vertex> {
        let mut curr = Vertex { id: Default::default(), t: Default::default() };

        for (i, stage) in stages.iter().enumerate() {
            let new = self.create_object(object_type.clone());
            self.add_object_properties(&new, &stage, PropertyType::Base);
            self.add_object_properties(&new, &json!({
                KEY_GVID: stage.replace("-", "_"),
                KEY_GV_LABEL: stage,
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
        // self.regression.index_property(indradb::Identifier::new("name").unwrap()).expect("panic message");
        // Project
        let project = self.create_object(VertexTypes::Project);
        self.add_object_properties(&project, &self.config.project, PropertyType::Base);
        self.add_object_properties(&project, &json!({
            KEY_GVID: self.config.project.name.replace("-", "_"),
            KEY_GV_LABEL: self.config.project.name.replace("-", "_"),
        }), PropertyType::Gv);

        // Ci
        let ci = self.create_object(VertexTypes::Ci);
        self.add_object_properties(&ci, &self.config.ci, PropertyType::Base);
        self.add_object_properties(&ci, &json!({
            KEY_GVID: format!("{}_{}", self.config.project.name.replace("-", "_"), KEY_CI),
            KEY_GV_LABEL: KEY_CI,
        }), PropertyType::Gv);
        self.create_relationship(&project, &ci);

        // Eut
        let eut = self.create_object(VertexTypes::Eut);
        self.add_object_properties(&eut, &self.config.eut, PropertyType::Base);
        self.add_object_properties(&eut, &json!({
            KEY_GVID: self.config.eut.module.replace("-", "_"),
            KEY_GV_LABEL: self.config.eut.module,
        }), PropertyType::Gv);
        let module = self.load_object_config(&VertexTypes::get_name_by_object(&eut), &self.config.eut.module);
        let v = to_value(module).unwrap();
        self.create_relationship(&project, &eut);

        let eut_providers = self.create_object(VertexTypes::Providers);
        self.create_relationship(&eut, &eut_providers);

        for k in EUT_KEY_ORDER.iter() {
            let obj = v.as_object().unwrap().get(*k).unwrap();
            match *k {
                k if k == KEY_NAME => {
                    let eut_o_p = self.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.to_string(), obj.clone());
                    self.add_object_properties(&eut, &p, PropertyType::Module);
                    self.add_object_properties(&eut_providers, &json!({
                        KEY_GVID: format!("{}_{}_{}", eut.t.as_str(), KEY_PROVIDERS, &obj.as_str().unwrap()),
                        KEY_GV_LABEL: KEY_PROVIDERS
                    }), PropertyType::Gv);
                }
                k if k == KEY_RELEASE => {
                    let eut_o_p = self.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.to_string(), obj.clone());
                    self.add_object_properties(&eut, &p, PropertyType::Module);
                }
                k if k == KEY_PROVIDER => {
                    for p in obj.as_array().unwrap().iter() {
                        let p_o = self.create_object(VertexTypes::EutProvider);
                        self.create_relationship(&eut_providers, &p_o);
                        self.add_object_properties(&p_o, &json!({KEY_NAME: &p.as_str().unwrap()}), PropertyType::Base);
                        self.add_object_properties(&p_o, &json!({
                            KEY_GVID: format!("{}_{}_{}", eut.t.as_str(), KEY_PROVIDER, &p.as_str().unwrap()),
                            KEY_GV_LABEL: &p.as_str().unwrap()
                        }), PropertyType::Gv);
                    }
                }
                k if k == KEY_CI => {
                    let eut_o_p = self.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.to_string(), obj.clone());
                    self.add_object_properties(&eut, &p, PropertyType::Module);
                }
                k if k == KEY_SITES => {
                    let o = self.create_object(VertexTypes::get_type_by_key(k));
                    self.create_relationship(&eut, &o);
                    self.add_object_properties(&o, &json!({
                        KEY_GVID: format!("{}_{}", self.config.eut.module, k),
                        KEY_GV_LABEL: k
                    }), PropertyType::Gv);
                    let _p = self.get_object_neighbour_out(&eut.id, EdgeTypes::HasProviders);
                    let provider = self.get_object_neighbours_with_properties_out(&_p.id, EdgeTypes::ProvidesProvider);
                    let mut id_name_map: HashMap<&str, Uuid> = HashMap::new();

                    //Generate provider name to vertex id map
                    for p in provider.iter() {
                        id_name_map.insert(&p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().
                            get(KEY_NAME).unwrap().as_str().unwrap(), p.vertex.id);
                    }

                    for (site_name, site_attr) in obj.as_object().unwrap().iter() {
                        let site_count = site_attr.as_object().unwrap().get(KEY_COUNT).unwrap().as_i64().unwrap();

                        if site_count == 1 {
                            let s_o = self.create_object(VertexTypes::Site);
                            self.create_relationship(&o, &s_o);
                            let provider = &site_attr.as_object().unwrap().get(KEY_PROVIDER).unwrap().as_str().unwrap();
                            let p_o = self.get_object(&id_name_map.get(provider).unwrap());
                            self.create_relationship(&s_o, &p_o);
                            self.add_object_properties(&s_o, &json!({KEY_NAME: site_name}), PropertyType::Base);
                            self.add_object_properties(&s_o, &json!({
                                KEY_GVID: format!("{}_{}_{}", self.config.eut.module, k, site_name),
                                KEY_GV_LABEL: site_name
                            }), PropertyType::Gv);
                        } else if site_count > 1 {
                            for c in 1..=site_count {
                                let s_o = self.create_object(VertexTypes::Site);
                                self.create_relationship(&o, &s_o);
                                let provider = &site_attr.as_object().unwrap().get(KEY_PROVIDER).unwrap().as_str().unwrap();
                                let p_o = self.get_object(&id_name_map.get(provider).unwrap());
                                self.create_relationship(&s_o, &p_o);
                                self.add_object_properties(&s_o, &json!({KEY_NAME: format!("{}_{}", site_name, c)}),
                                                           PropertyType::Base);
                                self.add_object_properties(&s_o, &json!({
                                    KEY_GVID: format!("{}_{}_{}_{}", self.config.eut.module, k, site_name, c),
                                    KEY_GV_LABEL: format!("{}_{}", site_name, c)
                                }), PropertyType::Gv);
                            }
                        }
                    }
                }
                k if k == KEY_FEATURES => {
                    let o = self.create_object(VertexTypes::get_type_by_key(k));
                    self.create_relationship(&eut, &o);
                    self.add_object_properties(&o, &json!({
                        KEY_GVID: format!("{}_{}", self.config.eut.module, k),
                        KEY_GV_LABEL: k
                    }), PropertyType::Gv);

                    for f in obj.as_array().unwrap().iter() {
                        for (k, v) in f.as_object().unwrap().iter() {
                            let f_o = self.create_object(VertexTypes::get_type_by_key(k));
                            self.create_relationship(&o, &f_o);
                            self.add_object_properties(&f_o, &json!({KEY_NAME: &v.as_str().unwrap()}), PropertyType::Base);
                            self.add_object_properties(&f_o, &json!({
                                    KEY_GVID: format!("{}_{}_{}", self.config.eut.module, k, v.as_str().unwrap()),
                                    KEY_GV_LABEL: &v.as_str().unwrap()
                                }), PropertyType::Gv);

                            let f_p = self.get_object_properties(&f_o).unwrap().props;
                            let name = f_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                            // FEATURE MODULE CFG
                            let cfg = self.load_object_config(&VertexTypes::get_name_by_object(&f_o), &name);
                            for (k, v) in cfg.as_object().unwrap().iter() {
                                match k {
                                    k if k == KEY_SCRIPTS_PATH => {
                                        let f_o_p = self.get_object_properties(&f_o).unwrap().props;
                                        let mut p = f_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                        self.add_object_properties(&f_o, &p, PropertyType::Module);
                                    }
                                    k if k == KEY_RELEASE => {
                                        let f_o_p = self.get_object_properties(&f_o).unwrap().props;
                                        let mut p = f_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                        self.add_object_properties(&f_o, &p, PropertyType::Module);
                                    }
                                    k if k == KEY_NAME => {
                                        let f_o_p = self.get_object_properties(&f_o).unwrap().props;
                                        let mut p = f_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                        self.add_object_properties(&f_o, &p, PropertyType::Module);
                                    }
                                    k if k == KEY_CI => {
                                        let f_o_p = self.get_object_properties(&f_o).unwrap().props;
                                        let mut p = f_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                        self.add_object_properties(&f_o, &p, PropertyType::Module);
                                    }
                                    k if k == KEY_SCRIPTS => {
                                        let f_o_p = self.get_object_properties(&f_o).unwrap().props;
                                        let mut p = f_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                        self.add_object_properties(&f_o, &p, PropertyType::Module);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                k if k == KEY_SCRIPTS_PATH => {
                    let eut_o_p = self.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.to_string(), obj.clone());
                    self.add_object_properties(&eut, &p, PropertyType::Module);
                }
                k if k == KEY_SCRIPTS => {
                    let eut_o_p = self.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.to_string(), obj.clone());
                    self.add_object_properties(&eut, &p, PropertyType::Module);
                }
                k if k == KEY_RTES => {
                    let o = self.create_object(VertexTypes::get_type_by_key(k));
                    self.create_relationship(&eut, &o);
                    //Rte
                    for r in obj.as_array().unwrap().iter() {
                        let r_o = self.create_object(VertexTypes::Rte);
                        self.create_relationship(&o, &r_o);
                        self.add_object_properties(&r_o, &json!({
                            KEY_GVID: format!("{}_{}", KEY_RTE, &r.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap()),
                            KEY_GV_LABEL: &r.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap()
                        }), PropertyType::Gv);

                        let rte_p_o = self.create_object(VertexTypes::Providers);
                        self.create_relationship(&r_o, &rte_p_o);
                        self.add_object_properties(&rte_p_o, &json!({
                            KEY_GVID: format!("{}_{}_{}", KEY_RTE, KEY_PROVIDERS, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                            KEY_GV_LABEL: KEY_PROVIDERS
                        }), PropertyType::Gv);
                        // REL: RTE -> Features
                        let eut_f_o = self.get_object_neighbour_out(&eut.id, EdgeTypes::HasFeatures);
                        self.create_relationship(&r_o, &eut_f_o);

                        for (k, v) in r.as_object().unwrap().iter() {
                            match k {
                                k if k == KEY_MODULE => {
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
                                                KEY_GVID: format!("{}_{}", &k, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                KEY_GV_LABEL: &c_o.t.as_str()
                                            }), PropertyType::Gv);
                                }
                                //Connections
                                k if k == KEY_CONNECTIONS => {
                                    let cs_o = self.create_object(VertexTypes::get_type_by_key(k));
                                    self.add_object_properties(&cs_o, &json!({
                                                KEY_GVID: format!("{}_{}", &k, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                KEY_GV_LABEL: &cs_o.t.as_str()
                                            }), PropertyType::Gv);
                                    self.create_relationship(&r_o, &cs_o);

                                    for item in v.as_array().unwrap().iter() {
                                        //Connection
                                        let c_o = self.create_object(VertexTypes::Connection);
                                        self.create_relationship(&cs_o, &c_o);
                                        let c_name = item.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                                        self.add_object_properties(&c_o, &json!({KEY_NAME: c_name}), PropertyType::Base);
                                        self.add_object_properties(&c_o, &json!({
                                                KEY_GVID: format!("{}_{}_{}", KEY_CONNECTION, c_name.replace("-", "_"), &r.as_object().
                                                    unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                KEY_GV_LABEL: c_name
                                            }), PropertyType::Gv);

                                        //Connection Source
                                        let source = item.as_object().unwrap().get(KEY_SOURCE).unwrap().as_str().unwrap();
                                        let src_o = self.create_object(VertexTypes::ConnectionSrc);
                                        self.create_relationship(&c_o, &src_o);
                                        self.add_object_properties(&src_o, &json!({KEY_NAME: &source, KEY_RTE: &r.as_object().
                                            unwrap().get(KEY_MODULE).unwrap().as_str().unwrap()}), PropertyType::Base);
                                        self.add_object_properties(&src_o, &json!({
                                            KEY_GVID: format!("{}_{}_{}_{}", "connection_src", &c_name.replace("-","_"), &source, &r.as_object().unwrap().
                                                get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                            KEY_GV_LABEL: source
                                        }), PropertyType::Gv);

                                        let _sites = self.get_object_neighbour_out(&eut.id, EdgeTypes::HasSites);
                                        let sites = self.get_object_neighbours_with_properties_out(&_sites.id, EdgeTypes::HasSite);

                                        //Connection Source -> Site
                                        for s in sites.iter() {
                                            let site_name = s.props.get(PropertyType::Base.index()).unwrap().value.as_object().
                                                unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                                            if site_name == source {
                                                self.create_relationship(&src_o, &s.vertex);
                                            }
                                        }

                                        //Connection Destinations
                                        let destinations = item.as_object().unwrap().get("destinations").unwrap().as_array().unwrap();
                                        for d in destinations.iter() {
                                            let dst_o = self.create_object(VertexTypes::ConnectionDst);
                                            self.create_relationship(&src_o, &dst_o);
                                            self.add_object_properties(&dst_o, &json!({KEY_NAME: &d, KEY_RTE: &r.as_object().
                                                unwrap().get(KEY_MODULE).unwrap().as_str().unwrap()}), PropertyType::Base);
                                            self.add_object_properties(&dst_o, &json!({
                                                         KEY_GVID: format!("{}_{}_{}_{}", "connection_dst", &c_name.replace("-","_"), d.as_str().unwrap(), &r.as_object().
                                                            unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                         KEY_GV_LABEL: d.as_str().unwrap()
                                                     }), PropertyType::Gv);

                                            //Connection Destination -> Site
                                            for s in sites.iter() {
                                                let site_name = s.props.get(PropertyType::Base.index()).unwrap().value.as_object().
                                                    unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                                                if site_name == d {
                                                    self.create_relationship(&dst_o, &s.vertex);
                                                }
                                            }
                                        }

                                        //Tests
                                        let tests = item.as_object().unwrap().get(KEY_TESTS).unwrap().as_array().unwrap();
                                        for test in tests.iter() {
                                            let t_o = self.create_object(VertexTypes::Test);
                                            self.create_relationship(&src_o, &t_o);

                                            for (k, v) in test.as_object().unwrap().iter() {
                                                match k {
                                                    k if k == KEY_NAME => {
                                                        let t_o_p = self.get_object_properties(&t_o).unwrap().props;
                                                        let mut p = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                        p.insert(k.clone(), v.clone());
                                                        self.add_object_properties(&t_o, &p, PropertyType::Base);
                                                    }
                                                    k if k == KEY_MODULE => {
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
                                                        let t_name = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                                                        let t_module = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                                                        self.add_object_properties(&t_o, &json!({
                                                                             KEY_GVID: format!("{}_{}_{}", t_o.t.as_str(), t_name.replace("-", "_"), &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                             KEY_GV_LABEL: format!("t_{}", t_module)
                                                                         }), PropertyType::Gv);
                                                    }
                                                    k if k == KEY_CI => {
                                                        let t_o_p = self.get_object_properties(&t_o).unwrap().props;
                                                        let mut p = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                        p.append(&mut json!({k: v.as_object().unwrap().clone()}).as_object().unwrap().clone());
                                                        self.add_object_properties(&t_o, &p, PropertyType::Base);
                                                    }
                                                    k if k == KEY_VERIFICATIONS => {
                                                        for v in v.as_array().unwrap().iter() {
                                                            let v_o = self.create_object(VertexTypes::Verification);
                                                            self.create_relationship(&t_o, &v_o);
                                                            let v_name = v.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                                                            let v_module = v.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                                                            self.add_object_properties(&v_o, v, PropertyType::Base);
                                                            self.add_object_properties(&v_o, &json!({
                                                                                 KEY_GVID: format!("{}_{}_{}", v_o.t.as_str(), v_name.replace("-", "_"), &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                                 KEY_GV_LABEL: format!("v_{}", v_module)
                                                                             }), PropertyType::Gv);
                                                            // Verification module cfg
                                                            let cfg = self.load_object_config(&VertexTypes::get_name_by_object(&v_o), &v_module);
                                                            for (k, v) in cfg.as_object().unwrap().iter() {
                                                                match k {
                                                                    k if k == KEY_NAME => {
                                                                        let v_o_p = self.get_object_properties(&v_o).unwrap().props;
                                                                        let mut p = v_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                        self.add_object_properties(&v_o, &p, PropertyType::Module);
                                                                    }
                                                                    k if k == KEY_SCRIPTS => {
                                                                        let v_o_p = self.get_object_properties(&v_o).unwrap().props;
                                                                        let mut p = v_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                        self.add_object_properties(&v_o, &p, PropertyType::Module);
                                                                    }
                                                                    k if k == KEY_SCRIPTS_PATH => {
                                                                        let v_o_p = self.get_object_properties(&v_o).unwrap().props;
                                                                        let mut p = v_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                        self.add_object_properties(&v_o, &p, PropertyType::Module);
                                                                    }
                                                                    _ => {}
                                                                }
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            //Test module cfg
                                            let t_p = self.get_object_properties(&t_o).unwrap().props;
                                            let module = t_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                                            let cfg = self.load_object_config(&VertexTypes::get_name_by_object(&t_o), &module);
                                            for (k, v) in cfg.as_object().unwrap().iter() {
                                                match k {
                                                    k if k == KEY_NAME => {
                                                        let t_o_p = self.get_object_properties(&t_o).unwrap().props;
                                                        let mut p = t_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                        self.add_object_properties(&t_o, &p, PropertyType::Module);
                                                    }
                                                    k if k == KEY_SCRIPTS => {
                                                        let t_o_p = self.get_object_properties(&t_o).unwrap().props;
                                                        let mut p = t_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                        self.add_object_properties(&t_o, &p, PropertyType::Module);
                                                    }
                                                    k if k == KEY_SCRIPTS_PATH => {
                                                        let t_o_p = self.get_object_properties(&t_o).unwrap().props;
                                                        let mut p = t_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                        self.add_object_properties(&t_o, &p, PropertyType::Module);
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
                        //Rte module cfg
                        let r_p = self.get_object_properties(&r_o).unwrap().props;
                        let module = r_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                        let cfg = self.load_object_config(&VertexTypes::get_name_by_object(&r_o), &module);

                        for (k, v) in cfg.as_object().unwrap().iter() {
                            match k {
                                k if k == KEY_NAME => {
                                    let r_o_p = self.get_object_properties(&r_o).unwrap().props;
                                    let mut p = r_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                    p.append(&mut json!({k: v}).as_object().unwrap().clone());
                                    self.add_object_properties(&r_o, &p, PropertyType::Module);
                                }
                                k if k == KEY_PROVIDER => {
                                    for (p, v) in v.as_object().unwrap().iter() {
                                        let o = self.create_object(VertexTypes::RteProvider);
                                        self.create_relationship(&rte_p_o, &o);
                                        self.add_object_properties(&o, &json!({KEY_NAME: p}), PropertyType::Module);
                                        self.add_object_properties(&o, &json!({
                                                KEY_GVID: format!("{}_{}_{}", KEY_PROVIDER, p, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                KEY_GV_LABEL: p
                                            }), PropertyType::Gv);

                                        for (k, v) in v.as_object().unwrap().iter() {
                                            match k {
                                                k if k == KEY_CI => {
                                                    let p_ci_o = self.create_object(VertexTypes::Ci);
                                                    self.create_relationship(&o, &p_ci_o);
                                                    self.add_object_properties(&p_ci_o, &json!({
                                                            KEY_GVID: format!("{}_{}_{}_{}", KEY_PROVIDER, k, p, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                            KEY_GV_LABEL: k
                                                        }), PropertyType::Gv);
                                                    self.add_object_properties(&p_ci_o, &v.as_object().unwrap(), PropertyType::Base);
                                                }
                                                k if k == KEY_SHARE => {
                                                    let s_o = self.create_object(VertexTypes::Share);
                                                    self.create_relationship(&o, &s_o);
                                                    self.add_object_properties(&s_o, &json!({
                                                        KEY_GVID: format!("{}_{}_{}_{}", KEY_PROVIDER, k, p, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                        KEY_GV_LABEL: k
                                                    }), PropertyType::Gv);
                                                    self.add_object_properties(&s_o, &v.as_object().unwrap(), PropertyType::Base);
                                                }
                                                k if k == KEY_COMPONENTS => {
                                                    let c_o = self.create_object(VertexTypes::Components);
                                                    self.create_relationship(&o, &c_o);
                                                    self.add_object_properties(&c_o, &json!({
                                                            KEY_GVID: format!("{}_{}_{}_{}", KEY_PROVIDER, p, k, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                            KEY_GV_LABEL: k
                                                        }), PropertyType::Gv);

                                                    for (k, v) in v.as_object().unwrap().iter() {
                                                        match k {
                                                            k if k == KEY_SRC => {
                                                                let c_src_o = self.create_object(VertexTypes::ComponentSrc);
                                                                self.create_relationship(&c_o, &c_src_o);
                                                                self.add_object_properties(&c_src_o, &json!({
                                                                        KEY_GVID: format!("{}_{}_{}_{}_{}", KEY_RTE, p, KEY_COMPONENT, k, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                        KEY_GV_LABEL: k
                                                                    }), PropertyType::Gv);

                                                                for (k, v) in v.as_object().unwrap().iter() {
                                                                    let c_src_o_p = self.get_object_properties(&c_src_o).unwrap().props;
                                                                    match k {
                                                                        k if k == KEY_NAME => {
                                                                            let mut p = c_src_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                            p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                            self.add_object_properties(&c_src_o, &p, PropertyType::Base);
                                                                        }
                                                                        k if k == KEY_SCRIPTS => {
                                                                            let mut p = c_src_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                            p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                            self.add_object_properties(&c_src_o, &p, PropertyType::Base);
                                                                        }
                                                                        k if k == KEY_SCRIPTS_PATH => {
                                                                            let mut p = c_src_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                            p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                            self.add_object_properties(&c_src_o, &p, PropertyType::Base);
                                                                        }
                                                                        _ => {}
                                                                    }
                                                                }
                                                            }
                                                            k if k == KEY_DST => {
                                                                let c_dst_o = self.create_object(VertexTypes::ComponentDst);
                                                                self.create_relationship(&c_o, &c_dst_o);
                                                                self.add_object_properties(&c_dst_o, &json!({
                                                                        KEY_GVID: format!("{}_{}_{}_{}_{}", KEY_RTE, p, KEY_COMPONENT, k, &r.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                        KEY_GV_LABEL: k
                                                                    }), PropertyType::Gv);

                                                                for (k, v) in v.as_object().unwrap().iter() {
                                                                    let c_dst_o_p = self.get_object_properties(&c_dst_o).unwrap().props;
                                                                    match k {
                                                                        k if k == KEY_NAME => {
                                                                            let mut p = c_dst_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                            p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                            self.add_object_properties(&c_dst_o, &p, PropertyType::Base);
                                                                        }
                                                                        k if k == KEY_SCRIPTS => {
                                                                            let mut p = c_dst_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                            p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                            self.add_object_properties(&c_dst_o, &p, PropertyType::Base);
                                                                        }
                                                                        k if k == KEY_SCRIPTS_PATH => {
                                                                            let mut p = c_dst_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                            p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                            self.add_object_properties(&c_dst_o, &p, PropertyType::Base);
                                                                        }
                                                                        _ => {}
                                                                    }
                                                                }
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
                                _ => {}
                            }
                        }
                        // Connection -> Component
                        let _c = self.get_object_neighbour_out(&r_o.id, EdgeTypes::HasConnections);
                        let connections = self.get_object_neighbours_out(&_c.id, EdgeTypes::HasConnection);
                        let _p = self.get_object_neighbour_out(&r_o.id, EdgeTypes::NeedsProvider);
                        let rte_provider = self.get_object_neighbours_with_properties_out(&_p.id, EdgeTypes::ProvidesProvider);

                        for c in connections.iter() {
                            let c_s = self.get_object_neighbour_with_properties_out(&c.id, EdgeTypes::HasConnectionSrc);
                            let site = self.get_object_neighbour_out(&c_s.vertex.id, EdgeTypes::RefersSite);
                            let site_provider = self.get_object_neighbour_with_properties_out(&site.id, EdgeTypes::UsesProvider);
                            let s_p_name = site_provider.props.get(PropertyType::Base.index()).unwrap().value.as_object().
                                unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                            let _c_d_s: Vec<VertexProperties> = self.get_object_neighbours_with_properties_out(&c_s.vertex.id, EdgeTypes::HasConnectionDst);
                            for p in rte_provider.iter() {
                                let _components = self.get_object_neighbour_out(&p.vertex.id, EdgeTypes::HasComponents);
                                let component_src = self.get_object_neighbour_out(&_components.id, EdgeTypes::HasComponentSrc);
                                let r_p_name = p.props.get(PropertyType::Module.index()).unwrap().value.as_object().
                                    unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                                if s_p_name == r_p_name {
                                    self.create_relationship(&c_s.vertex, &component_src);
                                }
                            }
                            //CONNECTION DSTs
                            for c_d in _c_d_s.iter() {
                                for p in rte_provider.iter() {
                                    let _components = self.get_object_neighbour_out(&p.vertex.id, EdgeTypes::HasComponents);
                                    let component_dst = self.get_object_neighbour_out(&_components.id, EdgeTypes::HasComponentDst);
                                    self.create_relationship(&c_d.vertex, &component_dst);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        //Rte Stages Deploy
        let rte_stage_deploy = self.add_ci_stages(&ci, &self.config.rte.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Eut Stages Deploy
        let eut_stage_deploy = self.add_ci_stages(&rte_stage_deploy.unwrap(), &self.config.eut.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Feature Stages Deploy
        let feature_stage_deploy = self.add_ci_stages(&eut_stage_deploy.unwrap(), &self.config.features.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Test Stages Deploy
        let mut test_stage_deploy = self.add_ci_stages(&feature_stage_deploy.unwrap(), &self.config.tests.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Verification Stages Deploy
        let verification_stage_deploy = self.add_ci_stages(&test_stage_deploy.unwrap(), &self.config.verifications.ci.stages.deploy, &VertexTypes::StageDeploy);

        //Test and Verification single job stages
        let _rtes = self.get_object_neighbour_out(&eut.id, EdgeTypes::UsesRtes);
        let rtes = self.get_object_neighbours_with_properties_out(&_rtes.id, EdgeTypes::ProvidesRte);
        let mut _test_stages: Vec<String> = Vec::new();
        let mut _verification_stages: Vec<String> = Vec::new();

        for rte in rtes.iter() {
            let _c = self.get_object_neighbour_out(&rte.vertex.id, EdgeTypes::HasConnections);
            let _conns = self.get_object_neighbours_out(&_c.id, EdgeTypes::HasConnection);
            for conn in _conns.iter() {
                let c_src = self.get_object_neighbour_with_properties_out(&conn.id, EdgeTypes::HasConnectionSrc);
                let tests = self.get_object_neighbours_with_properties_out(&c_src.vertex.id, EdgeTypes::Runs);
                for t in tests.iter() {
                    let t_stage_name = format!("{}-{}-{}-{}-{}",
                                               KEY_TEST,
                                               rte.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                               c_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                               &t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                               KEY_APPLY
                    ).replace("_", "-");
                    _test_stages.push(t_stage_name);

                    //Verification stages
                    let verifications = self.get_object_neighbours_with_properties_out(&t.vertex.id, EdgeTypes::Needs);
                    for v in verifications.iter() {
                        let v_stage_name = format!("{}-{}-{}-{}-{}",
                                                   KEY_VERIFICATION,
                                                   rte.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                                   c_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                                   &v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                                   KEY_APPLY
                        ).replace("_", "-");
                        _verification_stages.push(v_stage_name);
                    }
                }
            }
        }

        test_stage_deploy = self.add_ci_stages(&verification_stage_deploy.unwrap(), &_test_stages, &VertexTypes::StageDeploy);
        self.add_ci_stages(&test_stage_deploy.unwrap(), &_verification_stages, &VertexTypes::StageDeploy);

        //Feature Stages Destroy
        let mut stage_destroy: Option<Vertex> = None;
        let _features = self.get_object_neighbour_out(&eut.id, EdgeTypes::HasFeatures);
        let features = self.get_object_neighbours_out(&_features.id, EdgeTypes::HasFeature);

        if features.len() > 0 {
            stage_destroy = self.add_ci_stages(&ci, &self.config.features.ci.stages.destroy, &VertexTypes::StageDestroy);
        }

        //Eut Stages Destroy
        match stage_destroy {
            Some(f) => stage_destroy = self.add_ci_stages(&f, &self.config.eut.ci.stages.destroy, &VertexTypes::StageDestroy),
            None => stage_destroy = self.add_ci_stages(&ci, &self.config.eut.ci.stages.destroy, &VertexTypes::StageDestroy)
        }

        //Rte Stages Destroy
        self.add_ci_stages(&stage_destroy.unwrap(), &self.config.rte.ci.stages.destroy, &VertexTypes::StageDestroy);

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
        let v = to_value(value).unwrap();
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
        let v = to_value(value).unwrap();
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

    fn get_object(&self, id: &Uuid) -> Vertex {
        let q = self.regression.get(indradb::SpecificVertexQuery::single(*id));
        let _objs = indradb::util::extract_vertices(q.unwrap());
        let objs = _objs.unwrap();
        let o = objs.get(0).unwrap();
        o.clone()
    }

    fn get_object_with_properties(&self, id: &Uuid) -> VertexProperties {
        let obj = self.regression.get(indradb::SpecificVertexQuery::single(*id).properties().unwrap());
        let a = indradb::util::extract_vertex_properties(obj.unwrap()).unwrap();
        a.get(0).unwrap().clone()
    }

    fn get_object_neighbour_out(&self, id: &Uuid, identifier: EdgeTypes) -> Vertex {
        let i = Identifier::new(identifier.name().to_string()).unwrap();
        let o = self.regression.get(indradb::SpecificVertexQuery::single(*id).outbound().unwrap().t(i));
        let id = indradb::util::extract_edges(o.unwrap()).unwrap().get(0).unwrap().inbound_id;
        self.get_object(&id)
    }

    fn get_object_neighbours_out(&self, id: &Uuid, identifier: EdgeTypes) -> Vec<Vertex> {
        let i = Identifier::new(identifier.name().to_string()).unwrap();
        let o = self.regression.get(indradb::SpecificVertexQuery::single(*id).outbound().unwrap().t(i));
        let mut objs: Vec<Vertex> = Vec::new();

        for item in indradb::util::extract_edges(o.unwrap()).unwrap().iter() {
            objs.push(self.get_object(&item.inbound_id));
        }
        objs
    }

    fn get_object_neighbour_with_properties_out(&self, id: &Uuid, identifier: EdgeTypes) -> VertexProperties {
        let i = Identifier::new(identifier.name().to_string()).unwrap();
        let o = self.regression.get(indradb::SpecificVertexQuery::single(*id).outbound().unwrap().t(i));
        let id = indradb::util::extract_edges(o.unwrap()).unwrap().get(0).unwrap().inbound_id;
        self.get_object_with_properties(&id)
    }

    fn get_object_neighbours_with_properties_out(&self, id: &Uuid, identifier: EdgeTypes) -> Vec<VertexProperties> {
        let i = Identifier::new(identifier.name().to_string()).unwrap();
        let o = self.regression.get(indradb::SpecificVertexQuery::single(*id).outbound().unwrap().t(i));
        let mut objs: Vec<VertexProperties> = Vec::new();

        for item in indradb::util::extract_edges(o.unwrap()).unwrap().iter() {
            objs.push(self.get_object_with_properties(&item.inbound_id));
        }
        objs
    }

    fn get_object_neighbours_with_properties_in(&self, id: &Uuid, identifier: EdgeTypes) -> Vec<VertexProperties> {
        let i = Identifier::new(identifier.name().to_string()).unwrap();
        let o = self.regression.get(indradb::SpecificVertexQuery::single(*id).inbound().unwrap().t(i));
        let mut objs: Vec<VertexProperties> = Vec::new();

        for item in indradb::util::extract_edges(o.unwrap()).unwrap().iter() {
            //error!("ITEM: {:?}", &item);
            objs.push(self.get_object_with_properties(&item.outbound_id));
        }
        objs
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

    fn build_context(&self, id: Uuid) -> Context {
        info!("Build render context...");
        let project = self.get_object_with_properties(&id);
        let project_p_base = project.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap();

        let eut = self.get_object_neighbour_with_properties_out(&project.vertex.id, EdgeTypes::HasEut);
        let eut_p_base = eut.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap();
        let eut_p_module = eut.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();
        let eut_name = eut_p_base.get(KEY_MODULE).unwrap().as_str().unwrap().to_string();

        let _eut_providers = self.get_object_neighbour_out(&eut.vertex.id, EdgeTypes::HasProviders);
        let eut_provider = self.get_object_neighbours_with_properties_out(&_eut_providers.id, EdgeTypes::ProvidesProvider);

        let mut eut_provider_p_base = Vec::new();
        for p in eut_provider.iter() {
            let name = p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            eut_provider_p_base.push(String::from(name));
        }
        //Process eut sites
        let mut eut_sites: Vec<EutSiteRenderContext> = vec![];
        let _sites = self.get_object_neighbour_out(&eut.vertex.id, EdgeTypes::HasSites);
        let sites = self.get_object_neighbours_with_properties_out(&_sites.id, EdgeTypes::HasSite);

        for (i, s) in sites.iter().enumerate() {
            let site_name = s.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            let s_p = self.get_object_neighbour_with_properties_out(&s.vertex.id, EdgeTypes::UsesProvider);
            let provider_name = s_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

            //Process eut site scripts
            let scripts_path = eut.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
            let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
            for script in eut.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                let path = format!("{}/{}/{}/{}/{}", self.config.project.root_path, self.config.eut.path, eut_name.to_string(), scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                let contents = std::fs::read_to_string(path).expect("panic while opening eut site script file");
                let mut ctx: ScriptEutRenderContext = ScriptEutRenderContext::new(self.config.project.clone());
                ctx.name = Option::from(eut_name.to_string());
                ctx.site = Option::from(site_name.to_string());
                ctx.index = Option::from(i);
                ctx.release = Option::from(eut_p_module.get(KEY_RELEASE).unwrap().as_str().unwrap().to_string());
                ctx.provider = Option::from(provider_name.to_string());
                let mut commands: Vec<String> = Vec::new();

                for command in ctx.render_script(&ctx, &contents).lines() {
                    commands.push(format!("{:indent$}{}", "", command, indent = 0));
                }

                let data: HashMap<String, Vec<String>> = [
                    (script.as_object().unwrap().get(KEY_SCRIPT).unwrap().as_str().unwrap().to_string(), commands),
                ].into_iter().collect();
                scripts.push(data);
            }
            let eut_s_rc = EutSiteRenderContext {
                job: format!("{}_{}_{}", KEY_EUT, &eut_name, &site_name),
                name: site_name.to_string(),
                index: i,
                scripts,
                provider: provider_name.to_string(),
            };

            eut_sites.push(eut_s_rc);
        }

        let eut_rc = EutRenderContext {
            base: eut_p_base.clone(),
            module: eut_p_module.clone(),
            provider: eut_provider_p_base.clone(),
            project: self.config.project.clone(),
            sites: eut_sites,
        };

        let _features = self.get_object_neighbour_out(&eut.vertex.id, EdgeTypes::HasFeatures);
        let features = self.get_object_neighbours_with_properties_out(&_features.id, EdgeTypes::HasFeature);
        let mut features_rc: Vec<FeatureRenderContext> = Vec::new();

        for feature in features.iter() {
            let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
            let f_name = feature.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string();
            let scripts_path = feature.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();

            //Process feature scripts
            for script in feature.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                let path = format!("{}/{}/{}/{}/{}", self.config.project.root_path, self.config.features.path, f_name, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                let contents = std::fs::read_to_string(path).expect("panic while opening feature script file");
                let mut ctx: ScriptFeatureRenderContext = ScriptFeatureRenderContext::new(eut_name.to_string());
                ctx.name = Option::from(f_name.to_string());
                ctx.release = Option::from(feature.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_RELEASE).unwrap().as_str().unwrap().to_string());
                ctx.provider = Option::from(eut_provider_p_base.clone());
                let mut commands: Vec<String> = Vec::new();

                for command in ctx.render_script(&ctx, &contents).lines() {
                    commands.push(format!("{:indent$}{}", "", command, indent = 0));
                }

                let data: HashMap<String, Vec<String>> = [
                    (script.as_object().unwrap().get(KEY_SCRIPT).unwrap().as_str().unwrap().to_string(), commands),
                ].into_iter().collect();
                scripts.push(data);
            }

            let frc = FeatureRenderContext {
                ci: feature.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_CI).unwrap().as_object().unwrap().clone(),
                eut: eut_name.to_string(),
                name: f_name,
                release: feature.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_RELEASE).unwrap().as_str().unwrap().to_string(),
                scripts,
            };
            features_rc.push(frc);
        }

        let _rtes = self.get_object_neighbour_out(&eut.vertex.id, EdgeTypes::UsesRtes);
        let rtes = self.get_object_neighbours_with_properties_out(&_rtes.id, EdgeTypes::ProvidesRte);
        let mut rtes_rc: Vec<RteRenderContext> = Vec::new();

        for rte in rtes.iter() {
            let rte_name = rte.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
            let _c = self.get_object_neighbour_out(&rte.vertex.id, EdgeTypes::HasConnections);
            let connections = self.get_object_neighbours_with_properties_out(&_c.id, EdgeTypes::HasConnection);
            let mut rte_crcs = RteRenderContext {
                ci: HashMap::new(),
                tests: vec![],
                shares: vec![],
                components: Default::default(),
            };

            //Process provider share data render context
            let mut data_rc: Vec<ScriptRteProviderShareDataRenderContext> = Vec::new();
            for (i, site) in sites.iter().enumerate() {
                let components = self.get_object_neighbours_with_properties_in(&site.vertex.id, EdgeTypes::RefersSite);
                let mut srpsd_rc = ScriptRteProviderShareDataRenderContext {
                    name: site.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                    index: i,
                    has_client: false,
                    has_server: false,
                };

                for c in components.iter() {
                    match VertexTypes::get_type_by_key(c.vertex.t.as_str()) {
                        VertexTypes::ConnectionSrc => {
                            srpsd_rc.has_client = true
                        }
                        VertexTypes::ConnectionDst => {
                            srpsd_rc.has_server = true
                        }
                        _ => {}
                    }
                }
                data_rc.push(srpsd_rc);
            }

            let _provider = self.get_object_neighbour_out(&rte.vertex.id, EdgeTypes::NeedsProvider);
            let provider = self.get_object_neighbours_with_properties_out(&_provider.id, EdgeTypes::ProvidesProvider);

            for p in provider.iter() {
                let ci_p = self.get_object_neighbour_with_properties_out(&p.vertex.id, EdgeTypes::HasCi);
                let p_name = p.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                rte_crcs.ci.insert(p_name.to_string(),
                                   RteCiRenderContext {
                                       timeout: ci_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("timeout").unwrap().clone(),
                                       variables: ci_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("variables").unwrap().clone(),
                                   },
                );

                //Process provider share scripts render context
                let share_p = self.get_object_neighbour_with_properties_out(&p.vertex.id, EdgeTypes::NeedsShare);
                let scripts_path = share_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();

                for script in share_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                    let path = format!("{}/{}/{}/{}/{}/{}/{}", self.config.project.root_path, self.config.rte.path, rte_name, scripts_path, p_name, KEY_SHARE, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                    let contents = std::fs::read_to_string(path).expect("panic while opening feature script file");
                    let mut ctx: ScriptRteProviderShareRenderContext = ScriptRteProviderShareRenderContext::new(self.config.project.name.to_string());

                    ctx.rte = Option::from(rte_name.to_string());
                    ctx.eut = Option::from(eut_name.to_string());
                    ctx.vars = Option::from(self.config.project.vars.clone());
                    ctx.data = Option::from(serde_json::to_string(&data_rc).unwrap());
                    ctx.counter = Option::from(data_rc.len());
                    ctx.provider = Option::from(p_name.to_string());

                    let mut commands: Vec<String> = Vec::new();
                    for command in ctx.render_script(&ctx, &contents).lines() {
                        commands.push(format!("{:indent$}{}", "", command, indent = 0));
                    }

                    let data: HashMap<String, Vec<String>> = [
                        (script.as_object().unwrap().get(KEY_SCRIPT).unwrap().as_str().unwrap().to_string(), commands),
                    ].into_iter().collect();

                    scripts.push(data);
                }

                rte_crcs.shares.push(RteProviderShareRenderContext {
                    job: format!("{}_{}_{}_{}", KEY_RTE, &rte_name, p_name, KEY_SHARE),
                    rte: rte_name.to_string(),
                    eut: eut_name.to_string(),
                    provider: p_name.to_string(),
                    scripts,
                });
            }

            //Connection DST rt set
            let mut server_destinations: HashSet<String> = HashSet::new();
            //Process connections
            for conn in connections.iter() {
                let connection_name = conn.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let src = self.get_object_neighbour_with_properties_out(&conn.vertex.id, EdgeTypes::HasConnectionSrc);
                let src_name = src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let src_site = self.get_object_neighbour_with_properties_out(&src.vertex.id, EdgeTypes::RefersSite);
                let src_site_name = src_site.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let src_provider = self.get_object_neighbour_with_properties_out(&src_site.vertex.id, EdgeTypes::UsesProvider);
                let src_p_name = src_provider.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let comp_src = self.get_object_neighbour_with_properties_out(&src.vertex.id, EdgeTypes::HasComponentSrc);
                let comp_src_name = &comp_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let rte_job_name = format!("{}_{}_{}_{}_{}", KEY_RTE, &rte_name, &connection_name, &src_name, &comp_src_name);

                //Process rte src component scripts
                let scripts_path = comp_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();

                //Process client destination list
                let mut client_destinations: HashSet<String> = HashSet::new();
                let dsts = self.get_object_neighbours_with_properties_out(&src.vertex.id, EdgeTypes::HasConnectionDst);
                for dst in dsts.iter() {
                    client_destinations.insert(dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string());
                }

                for p in provider.iter() {
                    let p_name = p.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                    for script in comp_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                        if src_p_name == p_name {
                            let path = format!("{}/{}/{}/{}/{}/{}/{}", self.config.project.root_path, self.config.rte.path, rte_name, scripts_path, p_name, comp_src_name, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                            let contents = std::fs::read_to_string(&path).expect("panic while opening rte apply.script file");
                            let mut ctx: ScriptRteRenderContext = ScriptRteRenderContext::new(p_name.to_string());

                            ctx.rte = Option::from(rte_name.to_string());
                            ctx.eut = Option::from(eut_name.to_string());
                            ctx.site = Option::from(src_site_name.to_string());
                            ctx.project = Option::from(self.config.project.clone());
                            ctx.destinations = Option::from(client_destinations.clone());

                            let mut commands: Vec<String> = Vec::new();
                            for command in ctx.render_script(&ctx, &contents).lines() {
                                commands.push(format!("{:indent$}{}", "", command, indent = 0));
                            }

                            let data: HashMap<String, Vec<String>> = [
                                (script.as_object().unwrap().get(KEY_SCRIPT).unwrap().as_str().unwrap().to_string(), commands),
                            ].into_iter().collect();
                            scripts.push(data);
                        }
                    }
                }

                let rte_crc = RteComponentRenderContext {
                    job: rte_job_name.clone(),
                    rte: rte_name.to_string(),
                    name: comp_src_name.to_string(),
                    site: src_site_name.to_string(),
                    provider: src_p_name.to_string(),
                    scripts,
                };
                rte_crcs.components.push(rte_crc);

                //Process connection destinations
                for dst in dsts.iter() {
                    let dst_name = dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                    let dst_site = self.get_object_neighbour_with_properties_out(&dst.vertex.id, EdgeTypes::RefersSite);
                    let dst_site_name = dst_site.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                    let dst_provider = self.get_object_neighbour_with_properties_out(&dst_site.vertex.id, EdgeTypes::UsesProvider);
                    let dst_p_name = dst_provider.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                    let comp_dst = self.get_object_neighbour_with_properties_out(&dst.vertex.id, EdgeTypes::HasComponentDst);
                    let comp_dst_name = &comp_dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                    let rte_job_name = format!("{}_{}_{}_{}_{}", KEY_RTE, &rte_name, &connection_name, &dst_name, &comp_dst_name);

                    //Process server destination list
                    let rt_dsts = self.get_object_neighbours_with_properties_in(&dst.vertex.id, EdgeTypes::HasConnectionDst);
                    for dst in rt_dsts.iter() {
                        server_destinations.insert(dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string());
                    }

                    //Process rte dst component scripts
                    let scripts_path = comp_dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                    let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();

                    for p in provider.iter() {
                        let p_name = p.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                        for script in comp_dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                            if dst_p_name == p_name {
                                let path = format!("{}/{}/{}/{}/{}/{}/{}", self.config.project.root_path, self.config.rte.path, rte_name, scripts_path, p_name, comp_dst_name, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                                let contents = std::fs::read_to_string(path).expect("panic while opening rte apply.script file");
                                let mut ctx: ScriptRteRenderContext = ScriptRteRenderContext::new(p_name.to_string());

                                ctx.rte = Option::from(rte_name.to_string());
                                ctx.eut = Option::from(eut_name.to_string());
                                ctx.project = Option::from(self.config.project.clone());
                                ctx.site = Option::from(dst_site_name.to_string());
                                ctx.destinations = Option::from(server_destinations.clone());

                                let mut commands: Vec<String> = Vec::new();
                                for command in ctx.render_script(&ctx, &contents).lines() {
                                    commands.push(format!("{:indent$}{}", "", command, indent = 0));
                                }

                                let data: HashMap<String, Vec<String>> = [
                                    (script.as_object().unwrap().get(KEY_SCRIPT).unwrap().as_str().unwrap().to_string(), commands),
                                ].into_iter().collect();
                                scripts.push(data);
                            }
                        }
                    }

                    let rte_crc = RteComponentRenderContext {
                        job: rte_job_name.clone(),
                        rte: rte_name.to_string(),
                        site: dst_site_name.to_string(),
                        name: comp_dst_name.to_string(),
                        provider: dst_p_name.to_string(),
                        scripts,
                    };
                    rte_crcs.components.push(rte_crc);
                }

                //Tests
                let tests_p = self.get_object_neighbours_with_properties_out(&src.vertex.id, EdgeTypes::Runs);
                for t in tests_p.iter() {
                    let t_job_name = format!("{}_{}_{}_{}",
                                             KEY_TEST,
                                             rte_name,
                                             src_name,
                                             t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap()
                    ).replace("_", "-");

                    //Process test scripts
                    let t_name = t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                    let t_module = t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                    let scripts_path = t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                    let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
                    for script in t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                        let path = format!("{}/{}/{}/{}/{}", self.config.project.root_path, self.config.tests.path, t_module, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                        let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
                        let mut ctx: ScriptTestRenderContext = ScriptTestRenderContext::new(src_name.to_string());

                        ctx.rte = Option::from(rte_name.to_string());
                        ctx.name = Option::from(t_name.to_string());
                        ctx.module = Option::from(t_module.to_string());

                        let mut commands: Vec<String> = Vec::new();
                        for command in ctx.render_script(&ctx, &contents).lines() {
                            commands.push(format!("{:indent$}{}", "", command, indent = 0));
                        }

                        let data: HashMap<String, Vec<String>> = [
                            (script.as_object().unwrap().get(KEY_SCRIPT).unwrap().as_str().unwrap().to_string(), commands),
                        ].into_iter().collect();
                        scripts.push(data);
                    }

                    //Verifications
                    let verifications_p = self.get_object_neighbours_with_properties_out(&t.vertex.id, EdgeTypes::Needs);
                    let mut verifications: Vec<RteVerificationRenderContext> = Vec::new();
                    for v in verifications_p.iter() {
                        let v_job_name = format!("{}_{}_{}_{}_{}",
                                                 KEY_VERIFICATION,
                                                 rte_name,
                                                 src_name,
                                                 &t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                                 v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                        ).replace("_", "-");
                        //Process test scripts
                        let v_name = v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                        let v_module = v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                        let scripts_path = v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                        let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
                        for script in v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                            let path = format!("{}/{}/{}/{}/{}", self.config.project.root_path, self.config.verifications.path, v_module, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                            let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
                            let mut ctx: ScriptVerificationRenderContext = ScriptVerificationRenderContext::new(src_name.to_string());

                            ctx.rte = Option::from(rte_name.to_string());
                            ctx.name = Option::from(v_name.to_string());
                            ctx.module = Option::from(v_module.to_string());
                            ctx.test_name = Option::from(t_name.to_string());
                            ctx.test_module = Option::from(t_module.to_string());

                            let mut commands: Vec<String> = Vec::new();
                            for command in ctx.render_script(&ctx, &contents).lines() {
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
                            rte: rte_name.to_string(),
                            job: v_job_name,
                            name: v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                            module: v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                            scripts,
                        };
                        verifications.push(rte_vrc);
                    }

                    let rterc = RteTestRenderContext {
                        ci: t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_CI).unwrap().as_object().unwrap().clone(),
                        rte: rte_name.to_string(),
                        job: t_job_name,
                        name: t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                        module: t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                        provider: src_name.to_string(),
                        scripts,
                        verifications,
                    };
                    rte_crcs.tests.push(rterc);
                }
            }
            //error!("RTE: {:?} -- SERVER_DESTINATIONS_RT: {:?}", &rte_name, &server_destinations);
            rtes_rc.push(rte_crcs);
        }

        let mut stages: Vec<String> = Vec::new();
        let mut deploy_stages: Vec<String> = Vec::new();
        let mut destroy_stages: Vec<String> = Vec::new();

        let project_ci = self.get_object_neighbour_out(&project.vertex.id, EdgeTypes::HasCi);
        let s_deploy = self.get_object_neighbour_with_properties_out(&project_ci.id, EdgeTypes::HasDeployStages);
        let s_destroy = self.get_object_neighbour_with_properties_out(&project_ci.id, EdgeTypes::HasDestroyStages);
        deploy_stages.push(s_deploy.props.get(PropertyType::Base.index()).unwrap().value.as_str().unwrap().to_string());
        self.get_next_stage(&s_deploy.vertex.id, &mut deploy_stages);
        deploy_stages.push(s_destroy.props.get(PropertyType::Base.index()).unwrap().value.as_str().unwrap().to_string());
        self.get_next_stage(&s_destroy.vertex.id, &mut destroy_stages);

        stages.append(&mut deploy_stages);
        stages.append(&mut destroy_stages);

        let mut context = Context::new();
        context.insert(KEY_RTES, &rtes_rc);
        context.insert(KEY_EUT, &eut_rc);
        context.insert(KEY_CONFIG, &self.config);
        context.insert(KEY_STAGES, &stages);
        context.insert(KEY_FEATURES, &features_rc);
        context.insert(KEY_PROJECT, &project_p_base);

        //error!("{:#?}", context);
        info!("Build render context -> Done.");
        context
    }

    fn get_next_stage(&self, id: &Uuid, data: &mut Vec<String>) {
        for stage in self.get_object_neighbours_with_properties_out(&id, EdgeTypes::NextStage).iter() {
            data.push(stage.props.get(PropertyType::Base.index()).unwrap().value.as_str().unwrap().to_string());
            self.get_next_stage(&stage.vertex.id, data);
        }
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
                KEY_CONFIG: &self.config,
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
                    let n_p = self.get_object_properties(&self.get_object(&n.id));
                    match n_p {
                        Some(p) => {
                            let gv_id = p.props.get(PropertyType::Gv.index()).unwrap().value[KEY_GVID].as_str();
                            let gv_label = p.props.get(PropertyType::Gv.index()).unwrap().value[KEY_GV_LABEL].as_str();
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
                    let o_a = self.get_object(&e.outbound_id);
                    let o_b = self.get_object(&e.inbound_id);
                    let a_id = format!("{}", self.get_object(&e.outbound_id).t.to_string());
                    let b_id = format!("{}", self.get_object(&e.inbound_id).t.to_string());
                    let a_p = self.get_object_properties(&self.get_object(&o_a.id));
                    let b_p = self.get_object_properties(&self.get_object(&o_b.id));

                    match a_p {
                        Some(ap) => {
                            match b_p {
                                Some(bp) => {
                                    let a_p_name = &ap.props.get(PropertyType::Gv.index()).unwrap().value[KEY_GVID].as_str();
                                    let b_p_name = &bp.props.get(PropertyType::Gv.index()).unwrap().value[KEY_GVID].as_str();

                                    match a_p_name {
                                        Some(ap) => {
                                            match b_p_name {
                                                Some(bp) => items.push(json!({KEY_SRC: &ap, KEY_DST: &bp})),
                                                None => items.push(json!({KEY_SRC: &ap, KEY_DST: &b_id}))
                                            }
                                        }
                                        None => {
                                            match b_p_name {
                                                Some(bp) => items.push(json!({KEY_SRC: &a_id, KEY_DST: &bp})),
                                                None => items.push(json!({KEY_SRC: &a_id, KEY_DST: &b_id}))
                                            }
                                        }
                                    }
                                }
                                None => {
                                    let a_p_name = &ap.props.get(PropertyType::Gv.index()).unwrap().value[KEY_GVID].as_str();
                                    match a_p_name {
                                        Some(ap) => items.push(json!({KEY_SRC: &ap, KEY_DST: &b_id})),
                                        None => items.push(json!({KEY_SRC: &a_id, KEY_DST: &b_id}))
                                    }
                                }
                            }
                        }
                        None => {
                            match b_p {
                                Some(bp) => {
                                    let b_p_name = &bp.props.get(PropertyType::Gv.index()).unwrap().value[KEY_GVID].as_str();
                                    match b_p_name {
                                        Some(bp) => items.push(json!({KEY_SRC: &a_id, KEY_DST: &bp})),
                                        None => items.push(json!({KEY_SRC: &a_id, KEY_DST: &b_id}))
                                    }
                                }
                                None => items.push(json!({KEY_SRC: &a_id, KEY_DST: &b_id}))
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

    if cli.write_ci {
        r.to_file(&r.render(&r.build_context(p)), PIPELINE_FILE_NAME);
    }
    if cli.write_json {
        r.to_json();
        info!("{}", r.to_json());
    }
    if cli.render_ci {
        info!("{}", r.render(&r.build_context(p)));
    }
    if cli.write_gv {
        r.to_file(&r.to_gv(), &"graph.gv");
    }
}