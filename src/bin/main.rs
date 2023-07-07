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
use indradb::{BulkInsertItem, Edge, Identifier, QueryExt, QueryOutputValue, RangeVertexQuery, Vertex, VertexProperties};
use tera::{Context, Tera};
use lazy_static::lazy_static;
use std::option::Option;
use uuid::Uuid;
use serde_json::{json, Value};

const CONFIG_FILE_NAME: &str = "config.json";

// const SCRIPT_TYPE_APPLY: &str = "apply";
// const SCRIPT_TYPE_DESTROY: &str = "destroy";
// const SCRIPT_TYPE_ARTIFACTS: &str = "artifacts";
// const SCRIPT_TYPE_COLLECTOR_PATH: &str = "scripts";

const PIPELINE_FILE_NAME: &str = ".gitlab-ci.yml";
const PIPELINE_TEMPLATE_FILE_NAME: &str = ".gitlab-ci.yml.tpl";

const PROPERTY_TYPE_EUT: &str = "eut";
const PROPERTY_TYPE_MODULE: &str = "module";

const VERTEX_TYPE_CI: &str = "ci";
const VERTEX_TYPE_EUT: &str = "eut";
const VERTEX_TYPE_RTE: &str = "rte";
const VERTEX_TYPE_TEST: &str = "test";
const VERTEX_TYPE_SCRIPTS: &str = "scripts";
const VERTEX_TYPE_PROJECT: &str = "project";
const VERTEX_TYPE_FEATURE: &str = "feature";
const VERTEX_TYPE_PROVIDER: &str = "provider";
const VERTEX_TYPE_ENDPOINT: &str = "endpoint";
const VERTEX_TYPE_ENDPOINTS: &str = "endpoints";
const VERTEX_TYPE_COMPONENTS: &str = "components";
const VERTEX_TYPE_ENDPOINT_SRC: &str = "endpoint_src";
const VERTEX_TYPE_ENDPOINT_DST: &str = "endpoint_dst";
const VERTEX_TYPE_PROVIDER_AWS: &str = "provider_aws";
const VERTEX_TYPE_PROVIDER_GCP: &str = "provider_gcp";
const VERTEX_TYPE_VERIFICATION: &str = "verification";
const VERTEX_TYPE_SCRIPT_APPLY: &str = "script_apply";
const VERTEX_TYPE_STAGE_DEPLOY: &str = "stage_deploy";
const VERTEX_TYPE_STAGE_DESTROY: &str = "stage_destroy";
const VERTEX_TYPE_COMPONENT_SRC: &str = "component_src";
const VERTEX_TYPE_COMPONENT_DST: &str = "component_dst";
const VERTEX_TYPE_PROVIDER_AZURE: &str = "provider_azure";
const VERTEX_TYPE_STAGE_DEPLOY_ROOT: &str = "stage_deploy_root";
const VERTEX_TYPE_STAGE_DESTROY_ROOT: &str = "stage_destroy_root";

const EDGE_TYPE_HAS_CI: &str = "has_ci";
const EDGE_TYPE_HAS_EUT: &str = "has_eut";
const EDGE_TYPE_USES_RTE: &str = "uses_rte";
const EDGE_TYPE_RUNS_TEST: &str = "runs_test";
const EDGE_TYPE_NEXT_STAGE: &str = "next_stage";
const EDGE_TYPE_HAS_SCRIPTS: &str = "has_scripts";
const EDGE_TYPE_HAS_FEATURE: &str = "has_feature";
const EDGE_TYPE_HAS_PROVIDER: &str = "has_provider";
const EDGE_TYPE_HAS_ENDPOINT: &str = "has_endpoint";
const EDGE_TYPE_HAS_ENDPOINTS: &str = "has_endpoints";
const EDGE_TYPE_PROVIDES_TEST: &str = "provides_test";
const EDGE_TYPE_IS_APPLY_SCRIPT: &str = "is_apply_script";
const EDGE_TYPE_HAS_DEPLOY_STAGE: &str = "deploy_stage";
const EDGE_TYPE_HAS_COMPONENTS: &str = "has_components";
const EDGE_TYPE_HAS_PROVIDER_AWS: &str = "has_provider_aws";
const EDGE_TYPE_HAS_PROVIDER_GCP: &str = "has_provider_gcp";
const EDGE_TYPE_HAS_ENDPOINT_SRC: &str = "has_endpoint_src";
const EDGE_TYPE_HAS_ENDPOINT_DST: &str = "has_endpoint_dst";
const EDGE_TYPE_HAS_DESTROY_STAGE: &str = "destroy_stage";
const EDGE_TYPE_HAS_PROVIDER_AZURE: &str = "has_provider_azure";
const EDGE_TYPE_NEEDS_VERIFICATION: &str = "needs_verification";
const EDGE_TYPE_HAS_DEPLOY_STAGE_ROOT: &str = "has_deploy_stage_root";
const EDGE_TYPE_HAS_DESTROY_STAGE_ROOT: &str = "has_destroy_stage_root";
const EDGEP_TYPE_PROVIDES_SRC_COMPONENT: &str = "provides_src_component";
const EDGEP_TYPE_PROVIDES_DST_COMPONENT: &str = "provides_dst_component";

enum PropertyType {
    Module,
    Eut,
}

enum VertexTypes {
    Ci,
    Eut,
    Rte,
    Test,
    Scripts,
    Feature,
    Project,
    Provider,
    Endpoint,
    Endpoints,
    Components,
    EndpointSrc,
    EndpointDst,
    ProviderAws,
    ProviderGcp,
    ScriptApply,
    StageDeploy,
    StageDestroy,
    Verification,
    ComponentSrc,
    ComponentDst,
    ProviderAzure,
    StageDeployRoot,
    StageDestroyRoot,
}

enum EdgeTypes {
    HasCi,
    HasEut,
    UsesRte,
    RunsTest,
    NextStage,
    HasFeature,
    HasScripts,
    HasProvider,
    HasEndpoint,
    HasEndpoints,
    ProvidesTest,
    HasComponents,
    IsApplyScript,
    HasProviderAws,
    HasProviderGcp,
    HasDeployStage,
    HasEndpointSrc,
    HasEndpointDst,
    HasDestroyStage,
    HasProviderAzure,
    NeedsVerification,
    HasDeployStageRoot,
    HasDestroyStageRoot,
    ProvidesSrcComponent,
    ProvidesDstComponent,
}

impl VertexTypes {
    fn name(&self) -> &'static str {
        match *self {
            VertexTypes::Ci => VERTEX_TYPE_CI,
            VertexTypes::Eut => VERTEX_TYPE_EUT,
            VertexTypes::Rte => VERTEX_TYPE_RTE,
            VertexTypes::Test => VERTEX_TYPE_TEST,
            VertexTypes::Scripts => VERTEX_TYPE_SCRIPTS,
            VertexTypes::Project => VERTEX_TYPE_PROJECT,
            VertexTypes::Feature => VERTEX_TYPE_FEATURE,
            VertexTypes::Provider => VERTEX_TYPE_PROVIDER,
            VertexTypes::Endpoint => VERTEX_TYPE_ENDPOINT,
            VertexTypes::Endpoints => VERTEX_TYPE_ENDPOINTS,
            VertexTypes::Components => VERTEX_TYPE_COMPONENTS,
            VertexTypes::EndpointSrc => VERTEX_TYPE_ENDPOINT_SRC,
            VertexTypes::EndpointDst => VERTEX_TYPE_ENDPOINT_DST,
            VertexTypes::ProviderAws => VERTEX_TYPE_PROVIDER_AWS,
            VertexTypes::ProviderGcp => VERTEX_TYPE_PROVIDER_GCP,
            VertexTypes::ProviderAzure => VERTEX_TYPE_PROVIDER_AZURE,
            VertexTypes::ScriptApply => VERTEX_TYPE_SCRIPT_APPLY,
            VertexTypes::StageDeploy => VERTEX_TYPE_STAGE_DEPLOY,
            VertexTypes::StageDestroy => VERTEX_TYPE_STAGE_DESTROY,
            VertexTypes::Verification => VERTEX_TYPE_VERIFICATION,
            VertexTypes::ComponentSrc => VERTEX_TYPE_COMPONENT_SRC,
            VertexTypes::ComponentDst => VERTEX_TYPE_COMPONENT_DST,
            VertexTypes::StageDeployRoot => VERTEX_TYPE_STAGE_DEPLOY_ROOT,
            VertexTypes::StageDestroyRoot => VERTEX_TYPE_STAGE_DESTROY_ROOT,
        }
    }

    fn get(object: &Vertex) -> &'static str {
        match object.t.as_str() {
            VERTEX_TYPE_CI => VertexTypes::Ci.name(),
            VERTEX_TYPE_RTE => VertexTypes::Rte.name(),
            VERTEX_TYPE_EUT => VertexTypes::Eut.name(),
            VERTEX_TYPE_TEST =>VertexTypes::Test.name(),
            VERTEX_TYPE_SCRIPTS => VertexTypes::Scripts.name(),
            VERTEX_TYPE_PROJECT => VertexTypes::Project.name(),
            VERTEX_TYPE_FEATURE=> VertexTypes::Feature.name(),
            VERTEX_TYPE_PROVIDER => VertexTypes::Provider.name(),
            VERTEX_TYPE_ENDPOINT => VertexTypes::Endpoint.name(),
            VERTEX_TYPE_ENDPOINTS => VertexTypes::Endpoints.name(),
            VERTEX_TYPE_COMPONENTS => VertexTypes::Components.name(),
            VERTEX_TYPE_ENDPOINT_SRC => VertexTypes::EndpointSrc.name(),
            VERTEX_TYPE_ENDPOINT_DST => VertexTypes::EndpointDst.name(),
            VERTEX_TYPE_PROVIDER_AWS => VertexTypes::ProviderAws.name(),
            VERTEX_TYPE_PROVIDER_GCP => VertexTypes::ProviderGcp.name(),
            VERTEX_TYPE_VERIFICATION => VertexTypes::Verification.name(),
            VERTEX_TYPE_SCRIPT_APPLY => VertexTypes::ScriptApply.name(),
            VERTEX_TYPE_STAGE_DEPLOY => VertexTypes::StageDeploy.name(),
            VERTEX_TYPE_STAGE_DESTROY => VertexTypes::StageDestroy.name(),
            VERTEX_TYPE_COMPONENT_SRC => VertexTypes::ComponentSrc.name(),
            VERTEX_TYPE_COMPONENT_DST => VertexTypes::ComponentDst.name(),
            VERTEX_TYPE_PROVIDER_AZURE => VertexTypes::ProviderAzure.name(),
            VERTEX_TYPE_STAGE_DEPLOY_ROOT => VertexTypes::StageDeployRoot.name(),
            VERTEX_TYPE_STAGE_DESTROY_ROOT => VertexTypes::StageDestroyRoot.name(),
            _ => "empty"
        }
    }
}

impl EdgeTypes {
    fn name(&self) -> &'static str {
        match *self {
            EdgeTypes::HasCi => EDGE_TYPE_HAS_CI,
            EdgeTypes::HasEut => EDGE_TYPE_HAS_EUT,
            EdgeTypes::UsesRte => EDGE_TYPE_USES_RTE,
            EdgeTypes::RunsTest => EDGE_TYPE_RUNS_TEST,
            EdgeTypes::NextStage => EDGE_TYPE_NEXT_STAGE,
            EdgeTypes::HasScripts => EDGE_TYPE_HAS_SCRIPTS,
            EdgeTypes::HasFeature => EDGE_TYPE_HAS_FEATURE,
            EdgeTypes::HasProvider => EDGE_TYPE_HAS_PROVIDER,
            EdgeTypes::HasEndpoint => EDGE_TYPE_HAS_ENDPOINT,
            EdgeTypes::HasEndpoints => EDGE_TYPE_HAS_ENDPOINTS,
            EdgeTypes::ProvidesTest => EDGE_TYPE_PROVIDES_TEST,
            EdgeTypes::HasComponents => EDGE_TYPE_HAS_COMPONENTS,
            EdgeTypes::IsApplyScript => EDGE_TYPE_IS_APPLY_SCRIPT,
            EdgeTypes::HasProviderAws => EDGE_TYPE_HAS_PROVIDER_AWS,
            EdgeTypes::HasProviderGcp => EDGE_TYPE_HAS_PROVIDER_GCP,
            EdgeTypes::HasEndpointSrc => EDGE_TYPE_HAS_ENDPOINT_SRC,
            EdgeTypes::HasEndpointDst => EDGE_TYPE_HAS_ENDPOINT_DST,
            EdgeTypes::HasDeployStage => EDGE_TYPE_HAS_DEPLOY_STAGE,
            EdgeTypes::HasDestroyStage => EDGE_TYPE_HAS_DESTROY_STAGE,
            EdgeTypes::HasProviderAzure => EDGE_TYPE_HAS_PROVIDER_AZURE,
            EdgeTypes::NeedsVerification => EDGE_TYPE_NEEDS_VERIFICATION,
            EdgeTypes::HasDeployStageRoot => EDGE_TYPE_HAS_DEPLOY_STAGE_ROOT,
            EdgeTypes::HasDestroyStageRoot => EDGE_TYPE_HAS_DESTROY_STAGE_ROOT,
            EdgeTypes::ProvidesSrcComponent => EDGEP_TYPE_PROVIDES_SRC_COMPONENT,
            EdgeTypes::ProvidesDstComponent => EDGEP_TYPE_PROVIDES_DST_COMPONENT,
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
        map.insert(VertexTuple(VertexTypes::Project.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::HasCi.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Feature.name().to_string()), EdgeTypes::HasFeature.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Rte.name().to_string()), EdgeTypes::UsesRte.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Provider.name().to_string()), EdgeTypes::HasProvider.name());
        map.insert(VertexTuple(VertexTypes::Provider.name().to_string(), VertexTypes::ProviderAws.name().to_string()), EdgeTypes::HasProviderAws.name());
        map.insert(VertexTuple(VertexTypes::Provider.name().to_string(), VertexTypes::ProviderGcp.name().to_string()), EdgeTypes::HasProviderGcp.name());
        map.insert(VertexTuple(VertexTypes::Provider.name().to_string(), VertexTypes::ProviderAzure.name().to_string()), EdgeTypes::HasProviderAzure.name());
        map.insert(VertexTuple(VertexTypes::ProviderAws.name().to_string(), VertexTypes::Scripts.name().to_string()), EdgeTypes::HasScripts.name());
        map.insert(VertexTuple(VertexTypes::ProviderGcp.name().to_string(), VertexTypes::Scripts.name().to_string()), EdgeTypes::HasScripts.name());
        map.insert(VertexTuple(VertexTypes::ProviderAzure.name().to_string(), VertexTypes::Scripts.name().to_string()), EdgeTypes::HasScripts.name());
        map.insert(VertexTuple(VertexTypes::ProviderAws.name().to_string(), VertexTypes::Components.name().to_string()), EdgeTypes::HasComponents.name());
        map.insert(VertexTuple(VertexTypes::ProviderGcp.name().to_string(), VertexTypes::Components.name().to_string()), EdgeTypes::HasComponents.name());
        map.insert(VertexTuple(VertexTypes::ProviderAzure.name().to_string(), VertexTypes::Components.name().to_string()), EdgeTypes::HasComponents.name());
        map.insert(VertexTuple(VertexTypes::Components.name().to_string(), VertexTypes::ComponentSrc.name().to_string()), EdgeTypes::ProvidesSrcComponent.name());
        map.insert(VertexTuple(VertexTypes::Components.name().to_string(), VertexTypes::ComponentDst.name().to_string()), EdgeTypes::ProvidesDstComponent.name());
        map.insert(VertexTuple(VertexTypes::Scripts.name().to_string(), VertexTypes::ScriptApply.name().to_string()), EdgeTypes::IsApplyScript.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Endpoints.name().to_string()), EdgeTypes::HasEndpoints.name());
        map.insert(VertexTuple(VertexTypes::Endpoints.name().to_string(), VertexTypes::Endpoint.name().to_string()), EdgeTypes::HasEndpoint.name());
        map.insert(VertexTuple(VertexTypes::Endpoint.name().to_string(), VertexTypes::EndpointSrc.name().to_string()), EdgeTypes::HasEndpointSrc.name());
        map.insert(VertexTuple(VertexTypes::EndpointSrc.name().to_string(), VertexTypes::EndpointDst.name().to_string()), EdgeTypes::HasEndpointDst.name());
        map.insert(VertexTuple(VertexTypes::EndpointSrc.name().to_string(), VertexTypes::Test.name().to_string()), EdgeTypes::RunsTest.name());
        map.insert(VertexTuple(VertexTypes::EndpointDst.name().to_string(), VertexTypes::Test.name().to_string()), EdgeTypes::ProvidesTest.name());
        map.insert(VertexTuple(VertexTypes::EndpointSrc.name().to_string(), VertexTypes::ComponentSrc.name().to_string()), EdgeTypes::HasEndpointDst.name());
        map.insert(VertexTuple(VertexTypes::Test.name().to_string(), VertexTypes::Verification.name().to_string()), EdgeTypes::NeedsVerification.name());
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

#[derive(Serialize, Deserialize, Debug)]
struct JsonData {
    data: serde_json::Map<String, Value>,
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

    fn load_object_config(&self, _type: &str, module: &str) -> Option<JsonData> {
        println!("Loading module <{module}> configuration data...");
        let file: String;
        match _type {
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

    #[allow(dead_code)]
    fn render_script(context: &ScriptRenderContext, input: &str) -> String {
        println!("Render regression pipeline file script section...");
        let ctx = Context::from_serialize(context);
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

    fn init(&self) -> Uuid {
        // Project
        let project = self.create_object(VertexTypes::Project);
        self.add_object_properties(&project, &self.config.project, PropertyType::Eut);

        // Ci
        let ci = self.create_object(VertexTypes::Ci);
        self.add_object_properties(&ci, &self.config.ci, PropertyType::Eut);
        self.create_relationship(&project, &ci);

        // Ci stages
        let stage_deploy = self.create_object(VertexTypes::StageDeployRoot);
        self.create_relationship(&ci, &stage_deploy);
        let stage_destroy = self.create_object(VertexTypes::StageDestroyRoot);
        self.create_relationship(&ci, &stage_destroy);

        // Eut
        let eut = self.create_object(VertexTypes::Eut);
        let cfg = self.load_object_config(&VertexTypes::get(&eut), &self.config.eut.name);
        self.add_object_properties(&eut, &cfg.unwrap().data, PropertyType::Eut);
        self.create_relationship(&project, &eut);
        let eut_p = self.get_object_properties(&eut);

        for feature in eut_p.props.get(0).unwrap().value.get("eut").unwrap().get("features").unwrap().as_array().unwrap().iter() {
            let f_o = self.create_object(VertexTypes::Feature);
            let cfg = self.load_object_config(&VertexTypes::get(&f_o), &String::from(feature["module"].as_str().unwrap()));
            self.add_object_properties(&f_o, &cfg.unwrap().data, PropertyType::Module);
            self.create_relationship(&eut, &f_o);
        }

        //Stages Deploy
        let mut deploy_stage: Option<Vertex> = self.add_ci_stage(&stage_deploy, &json!(self.config.eut.ci.stages.deploy), VertexTypes::StageDeploy);
        if eut_p.props.get(0).unwrap().value.get("eut").unwrap().get("features").iter().len() > 0 {
            deploy_stage = self.add_ci_stage(&deploy_stage.unwrap(), &json!(self.config.features.ci.stages.deploy), VertexTypes::StageDeploy);
        }
        let rte_deploy_stage = self.add_ci_stage(deploy_stage.as_ref().unwrap(), &json!(&self.config.rte.ci.stages.deploy), VertexTypes::StageDeploy);

        //Stages Destroy
        let mut destroy_stage: Option<Vertex> = self.add_ci_stage(&stage_destroy, &json!(&self.config.rte.ci.stages.destroy), VertexTypes::StageDestroy);
        if eut_p.props.get(0).unwrap().value.get("eut").unwrap().get("features").iter().len() > 0 {
            destroy_stage = self.add_ci_stage(&destroy_stage.unwrap(), &json!(self.config.features.ci.stages.destroy), VertexTypes::StageDestroy);
        }
        self.add_ci_stage(&destroy_stage.unwrap(), &json!(self.config.eut.ci.stages.destroy), VertexTypes::StageDestroy);

        // Rtes
        for rte in eut_p.props.get(0).unwrap().value.get("eut").unwrap().get("rtes").unwrap().as_array().unwrap().iter() {
            let r_o = self.create_object(VertexTypes::Rte);
            self.add_object_properties(&r_o, &rte, PropertyType::Eut);
            let cfg = self.load_object_config(&VertexTypes::get(&r_o), &String::from(rte["module"].as_str().unwrap()));
            self.add_object_properties(&r_o, &cfg.unwrap().data, PropertyType::Module);
            self.create_relationship(&eut, &r_o);

            //Rte - Endpoints
            self.create_object(VertexTypes::Endpoints);
            for _ in rte["endpoints"].as_array().unwrap().iter() {
                let ep_o = self.create_object(VertexTypes::Endpoint);

                //Rte - Endpoint - Src
                for _ in rte["endpoints"].get(0).unwrap()["sources"].as_array().unwrap().iter() {
                    let ep_src_o = self.create_object(VertexTypes::EndpointSrc);
                    self.create_relationship(&ep_o, &ep_src_o);

                    //Rte - Tests
                    for test in rte["endpoints"].get(0).unwrap()["tests"].as_array().unwrap().iter() {
                        let t_o = self.create_object(VertexTypes::Test);
                        self.add_object_properties(&t_o, &test, PropertyType::Eut);
                        let t_p = self.get_object_properties(&t_o);
                        let name = format!("{}-{}-{}", self.config.tests.ci.stages.deploy[0], &rte["module"].as_str().unwrap().replace("_", "-"), &t_p.props.get(0).unwrap().value["name"].as_str().unwrap());
                        let test_deploy_stage = self.add_ci_stage(rte_deploy_stage.as_ref().unwrap(), &json!([name]), VertexTypes::StageDeploy);

                        //Rte - Verifications
                        for verification in test["verifications"].as_array().unwrap() {
                            let v_o = self.create_object(VertexTypes::Verification);
                            self.add_object_properties(&v_o, &verification, PropertyType::Eut);
                            // let verification_p = self.get_object_properties(&v_o);
                            self.create_relationship(&t_o, &v_o);
                            let name = format!("{}-{}-{}", self.config.verifications.ci.stages.deploy[0], &rte["module"].as_str().unwrap().replace("_", "-"), &t_p.props.get(0).unwrap().value["name"].as_str().unwrap());
                            self.add_ci_stage(test_deploy_stage.as_ref().unwrap(), &json!([name]), VertexTypes::StageDeploy);
                        }
                    }

                    //Rte - Endpoint - Dst
                    for _ in rte["endpoints"].get(0).unwrap()["destinations"].as_array().unwrap().iter() {
                        let ep_dst_o = self.create_object(VertexTypes::EndpointDst);
                        self.create_relationship(&ep_src_o, &ep_dst_o);
                    }
                }
            }


            // Rte - Provider
            let p_o = self.create_object(VertexTypes::Provider);
            self.create_relationship(&r_o, &p_o);
            let r_p = self.get_object_properties(&r_o);
            for p in eut_p.props.get(PropertyType::Eut.index()).unwrap().value[PropertyType::Eut.name()]["provider"].as_array().unwrap().iter() {
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
                for p in eut_p.props.get(PropertyType::Eut.index()).unwrap().value[PropertyType::Eut.name()]["provider"].as_array().unwrap().iter() {
                    let cs_o = self.create_object(VertexTypes::ComponentSrc);
                    let ds_o = self.create_object(VertexTypes::ComponentDst);
                    self.add_object_properties(&cs_o, &rte_module_cfg.props.get(PropertyType::Module.index()).unwrap().value["provider"][p.as_str().unwrap()]["components"]["src"], PropertyType::Module);
                    self.add_object_properties(&ds_o, &rte_module_cfg.props.get(PropertyType::Module.index()).unwrap().value["provider"][p.as_str().unwrap()]["components"]["dst"], PropertyType::Module);
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
                            let file = format!("{}/{}/{}/{}/{}/{}", self.config.project.root_path, self.config.rte.path, rte["module"].as_str().unwrap(), s_path.as_str().unwrap(), p.as_str().unwrap(), script["value"].as_str().unwrap());
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
        }


        project.id
    }

    fn create_object(&self, object_type: VertexTypes) -> Vertex {
        println!("Create new object of type <{}>...", object_type.name());
        let o = Vertex::new(Identifier::new(object_type.name()).unwrap());
        self.regression.create_vertex(&o).expect("panic while creating project db entry");
        println!("Create new object of type <{}> -> Done", object_type.name());
        o
    }

    fn create_relationship_identifier(&self, a: &Vertex, b: &Vertex) -> Identifier {
        println!("Create relationship identifier for <{}> and <{}>...", a.t.as_str(), b.t.as_str());
        let i = Identifier::new(self.get_relationship_type(&a, &b)).unwrap();
        println!("Create relationship identifier for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        i
    }

    fn create_relationship(&self, a: &Vertex, b: &Vertex) -> bool {
        println!("Create relationship for <{}> and <{}>...", a.t.as_str(), b.t.as_str());
        let i = self.create_relationship_identifier(&a, &b);
        let e = Edge::new(a.id, i, b.id);
        let status = self.regression.create_edge(&e).expect(&format!("panic build relationship between {} and {}", a.t.as_str(), b.t.as_str()));
        println!("Create relationship for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        status
    }

    fn add_object_properties<T: serde::Serialize>(&self, object: &Vertex, value: &T, property_type: PropertyType) {
        println!("Add new property to object <{}>...", object.t.to_string());
        let v = serde_json::to_value(value).unwrap();
        let p: BulkInsertItem;

        match property_type {
            PropertyType::Eut => {
                p = BulkInsertItem::VertexProperty(object.id, Identifier::new(PROPERTY_TYPE_EUT).unwrap(), indradb::Json::new(v.clone()));
            }
            PropertyType::Module => {
                p = BulkInsertItem::VertexProperty(object.id, Identifier::new(PROPERTY_TYPE_MODULE).unwrap(), indradb::Json::new(v.clone()));
            }
        }

        self.regression.bulk_insert(vec![p]).unwrap();
        println!("Add new property to object <{}> -> Done", object.t.to_string());
    }

    #[allow(dead_code)]
    fn add_relationship_properties<T: serde::Serialize>(&self, object: Edge, value: &T, property_type: PropertyType) {

        //println!("Add new property to relationship <{}>...", object.t.to_string());
        let v = serde_json::to_value(value).unwrap();
        let p: BulkInsertItem;

        match property_type {
            PropertyType::Eut => {
                p = BulkInsertItem::EdgeProperty(object, Identifier::new(PROPERTY_TYPE_EUT).unwrap(), indradb::Json::new(v.clone()));
            }
            PropertyType::Module => {
                p = BulkInsertItem::EdgeProperty(object, Identifier::new(PROPERTY_TYPE_MODULE).unwrap(), indradb::Json::new(v.clone()));
            }
        }

        self.regression.bulk_insert(vec![p]).unwrap();
        //println!("Add new property to relationship <{}> -> Done", object.t.to_string());
    }

    #[allow(dead_code)]
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

    fn get_object(&self, id: Uuid) -> Vertex {
        let q = self.regression.get(indradb::SpecificVertexQuery::single(id));
        let objs = indradb::util::extract_vertices(q.unwrap());
        let obj = objs.unwrap();
        let o = obj.get(0).unwrap();
        o.clone()
    }

    fn get_direct_neighbour_object_by_identifier(&self, object: &Vertex, identifier: VertexTypes) -> Vertex {
        println!("Get direct neighbor of <{}>...", object.t.as_str());
        let mut rvq = RangeVertexQuery::new();
        rvq.t = Option::from(Identifier::new(identifier.name()).unwrap());
        rvq.limit = 1;
        rvq.start_id = Option::from(object.id);
        let result = indradb::util::extract_vertices(self.regression.get(rvq).unwrap()).unwrap();
        println!("Get direct neighbor of <{}> -> Done.", object.t.as_str());
        result.get(0).unwrap().clone()
    }

    fn get_direct_neighbour_objects_by_identifier(&self, start: &Vertex, identifier: VertexTypes) -> Vec<Vertex> {
        println!("Get direct neighbors of <{}>...", start.t.as_str());
        let mut rvq = RangeVertexQuery::new();
        rvq.t = Option::from(Identifier::new(identifier.name()).unwrap());
        rvq.limit = 10;
        rvq.start_id = Option::from(start.id);
        let result = indradb::util::extract_vertices(self.regression.get(rvq).unwrap()).unwrap();
        println!("Get direct neighbors of <{}> -> Done.", start.t.as_str());
        result
    }

    fn get_object_properties(&self, object: &Vertex) -> VertexProperties {
        println!("Get object <{}> properties...", object.t.as_str());
        let b = indradb::SpecificVertexQuery::new(vec!(object.id)).properties().unwrap();
        let _r = self.regression.get(b);
        let r = indradb::util::extract_vertex_properties(_r.unwrap()).unwrap();
        println!("Get object <{}> properties -> Done.", object.t.as_str());
        r.get(0).unwrap().clone()
    }

    #[allow(dead_code)]
    fn get_relationship(&self, a: &Vertex, b: &Vertex) -> Vec<Edge> {
        println!("Get relationship for <{}> and <{}>...", &a.t.as_str(), &b.t.as_str());
        let i = self.create_relationship_identifier(&a, &b);
        let e = Edge::new(a.id, i, b.id);
        let r: Vec<indradb::QueryOutputValue> = self.regression.get(indradb::SpecificEdgeQuery::single(e.clone())).unwrap();
        let e = indradb::util::extract_edges(r).unwrap();
        println!("Get relationship for <{}> and <{}> -> Done.", a.t.as_str(), b.t.as_str());
        e
    }

    fn build_context(&self, id: Uuid) -> Context {
        println!("Build render context...");
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
        println!("##################################################################");
        for rte in _rtes.iter() {
            let rte_p = self.get_object_properties(&rte);

            for ep in rte_p.props.get(PropertyType::Eut.index()).unwrap().value["endpoints"].as_array().unwrap().iter() {
                for source in ep["sources"].as_array().unwrap().iter() {
                    println!("{} CLIENT: {:?}", &source.as_str().unwrap(), &rte_p.props.get(PropertyType::Module.index()).unwrap().value["provider"][source.as_str().unwrap()]["components"]["src"]);
                }
                for destination in ep["destinations"].as_array().unwrap().iter() {
                    println!("{} SERVER: {:?}", &destination.as_str().unwrap(), &rte_p.props.get(PropertyType::Module.index()).unwrap().value["provider"][destination.as_str().unwrap()]["components"]["dst"]);
                }
            }
            let data = json!({
                "cfg": &rte_p.props.get(PropertyType::Eut.index()).unwrap().value.clone(),
                "module": &rte_p.props.get(PropertyType::Module.index()).unwrap().value.clone()
            });
            rtes.push(data);
        }
        println!("##################################################################");
        let mut stages: Vec<String> = Vec::new();
        let mut deploy_stages: Vec<String> = Vec::new();
        let mut destroy_stages: Vec<String> = Vec::new();
        let s_deploy = self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::StageDeployRoot);
        let s_destroy = self.get_direct_neighbour_object_by_identifier(&project, VertexTypes::StageDestroyRoot);

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
        context.insert("eut", &eut_p.props[0].value["eut"]);
        context.insert("rtes", &rtes);
        context.insert("config", &self.config);
        context.insert("stages", &stages);
        context.insert("features", &features);
        context.insert("project", &project_p.props[0].value);

        //println!("{:#?}", context);
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
