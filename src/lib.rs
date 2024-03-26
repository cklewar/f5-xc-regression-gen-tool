use std::any::Any;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Debug};
use std::format;
use std::io::{Write};

use indradb::{Vertex, VertexProperties};
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Map, to_value, Value};
use serde_json::Value::Null;
use tera::{Context, Tera};
use uuid::Uuid;

use objects::{Ci, Eut, Features, Feature, Project, Providers, EutProvider, Rtes, Rte, Sites, Site,
              Dashboard, Application, Applications, Connections, Connection, ConnectionSource,
              ConnectionDestination, Test, Verification};

use crate::constants::*;
use crate::db::Db;

pub mod constants;
pub mod db;
pub mod objects;

pub enum PropertyType {
    Gv,
    Base,
    Module,
}

#[derive(Clone, PartialEq, Debug)]
pub enum VertexTypes {
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
    Summary,
    Feature,
    Features,
    Providers,
    Collector,
    Dashboard,
    Components,
    Connection,
    Connections,
    Application,
    RteProvider,
    EutProvider,
    StageDeploy,
    StageDestroy,
    Applications,
    Verification,
    ComponentSrc,
    ComponentDst,
    ConnectionSrc,
    ConnectionDst,
    DashboardProvider,
    None,
}

pub enum EdgeTypes {
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
    HasSummaries,
    HasProviders,
    NeedsProvider,
    HasComponents,
    HasConnection,
    SiteRefersRte,
    RefersFeature,
    HasConnections,
    HasDeployStages,
    HasApplications,
    HasComponentSrc,
    HasComponentDst,
    ProvidesProvider,
    HasConnectionSrc,
    HasConnectionDst,
    HasDestroyStages,
    FeatureRefersSite,
    ProvidesApplication,
    TestRefersApplication,
}

impl VertexTypes {
    pub fn name(&self) -> &'static str {
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
            VertexTypes::Dashboard => VERTEX_TYPE_DASHBOARD,
            VertexTypes::Collector => VERTEX_TYPE_COLLECTOR,
            VertexTypes::Providers => VERTEX_TYPE_PROVIDERS,
            VertexTypes::Connection => VERTEX_TYPE_CONNECTION,
            VertexTypes::Components => VERTEX_TYPE_COMPONENTS,
            VertexTypes::Connections => VERTEX_TYPE_CONNECTIONS,
            VertexTypes::Application => VERTEX_TYPE_APPLICATION,
            VertexTypes::Applications => VERTEX_TYPE_APPLICATIONS,
            VertexTypes::EutProvider => VERTEX_TYPE_EUT_PROVIDER,
            VertexTypes::RteProvider => VERTEX_TYPE_RTE_PROVIDER,
            VertexTypes::StageDeploy => VERTEX_TYPE_STAGE_DEPLOY,
            VertexTypes::Summary => VERTEX_TYPE_STAGE_DEPLOY,
            VertexTypes::StageDestroy => VERTEX_TYPE_STAGE_DESTROY,
            VertexTypes::Verification => VERTEX_TYPE_VERIFICATION,
            VertexTypes::ComponentSrc => VERTEX_TYPE_COMPONENT_SRC,
            VertexTypes::ComponentDst => VERTEX_TYPE_COMPONENT_DST,
            VertexTypes::ConnectionSrc => VERTEX_TYPE_CONNECTION_SRC,
            VertexTypes::ConnectionDst => VERTEX_TYPE_CONNECTION_DST,
            VertexTypes::DashboardProvider => VERTEX_TYPE_DASHBOARD_PROVIDER,
            VertexTypes::None => VERTEX_TYPE_NONE,
        }
    }

    pub(crate) fn get_name_by_object(object: &Vertex) -> &'static str {
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
            VERTEX_TYPE_SUMMARY => VertexTypes::Summary.name(),
            VERTEX_TYPE_PROVIDERS => VertexTypes::Providers.name(),
            VERTEX_TYPE_DASHBOARD => VertexTypes::Dashboard.name(),
            VERTEX_TYPE_COLLECTOR => VertexTypes::Collector.name(),
            VERTEX_TYPE_CONNECTION => VertexTypes::Connection.name(),
            VERTEX_TYPE_COMPONENTS => VertexTypes::Components.name(),
            VERTEX_TYPE_CONNECTIONS => VertexTypes::Connections.name(),
            VERTEX_TYPE_APPLICATION => VertexTypes::Application.name(),
            VERTEX_TYPE_APPLICATIONS => VertexTypes::Applications.name(),
            VERTEX_TYPE_VERIFICATION => VertexTypes::Verification.name(),
            VERTEX_TYPE_STAGE_DEPLOY => VertexTypes::StageDeploy.name(),
            VERTEX_TYPE_EUT_PROVIDER => VertexTypes::EutProvider.name(),
            VERTEX_TYPE_RTE_PROVIDER => VertexTypes::RteProvider.name(),
            VERTEX_TYPE_STAGE_DESTROY => VertexTypes::StageDestroy.name(),
            VERTEX_TYPE_COMPONENT_SRC => VertexTypes::ComponentSrc.name(),
            VERTEX_TYPE_COMPONENT_DST => VertexTypes::ComponentDst.name(),
            VERTEX_TYPE_CONNECTION_SRC => VertexTypes::ConnectionSrc.name(),
            VERTEX_TYPE_CONNECTION_DST => VertexTypes::ConnectionDst.name(),
            VERTEX_TYPE_DASHBOARD_PROVIDER => VertexTypes::DashboardProvider.name(),
            _ => "None"
        }
    }

    pub(crate) fn get_type_by_key(key: &str) -> VertexTypes {
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
            VERTEX_TYPE_SUMMARY => VertexTypes::Summary,
            VERTEX_TYPE_FEATURE => VertexTypes::Feature,
            VERTEX_TYPE_FEATURES => VertexTypes::Features,
            VERTEX_TYPE_PROVIDERS => VertexTypes::Providers,
            VERTEX_TYPE_DASHBOARD => VertexTypes::Dashboard,
            VERTEX_TYPE_COLLECTOR => VertexTypes::Collector,
            VERTEX_TYPE_CONNECTION => VertexTypes::Connection,
            VERTEX_TYPE_COMPONENTS => VertexTypes::Components,
            VERTEX_TYPE_CONNECTIONS => VertexTypes::Connections,
            VERTEX_TYPE_APPLICATION => VertexTypes::Application,
            VERTEX_TYPE_APPLICATIONS => VertexTypes::Applications,
            VERTEX_TYPE_VERIFICATION => VertexTypes::Verification,
            VERTEX_TYPE_EUT_PROVIDER => VertexTypes::EutProvider,
            VERTEX_TYPE_RTE_PROVIDER => VertexTypes::RteProvider,
            VERTEX_TYPE_STAGE_DEPLOY => VertexTypes::StageDeploy,
            VERTEX_TYPE_STAGE_DESTROY => VertexTypes::StageDestroy,
            VERTEX_TYPE_COMPONENT_SRC => VertexTypes::ComponentSrc,
            VERTEX_TYPE_COMPONENT_DST => VertexTypes::ComponentDst,
            VERTEX_TYPE_CONNECTION_SRC => VertexTypes::ConnectionSrc,
            VERTEX_TYPE_CONNECTION_DST => VertexTypes::ConnectionDst,
            VERTEX_TYPE_DASHBOARD_PROVIDER => VertexTypes::DashboardProvider,
            _ => VertexTypes::None
        }
    }
}

impl EdgeTypes {
    pub(crate) fn name(&self) -> &'static str {
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
            EdgeTypes::HasSummaries => EDGE_TYPE_HAS_SUMMARIES,
            EdgeTypes::NeedsShare => EDGE_TYPE_NEEDS_SHARE,
            EdgeTypes::HasFeatures => EDGE_TYPE_HAS_FEATURES,
            EdgeTypes::ProvidesRte => EDGE_TYPE_PROVIDES_RTE,
            EdgeTypes::HasProviders => EDGE_TYPE_HAS_PROVIDERS,
            EdgeTypes::UsesProvider => EDGE_TYPE_USES_PROVIDER,
            EdgeTypes::NeedsProvider => EDGE_TYPE_NEEDS_PROVIDER,
            EdgeTypes::HasComponents => EDGE_TYPE_HAS_COMPONENTS,
            EdgeTypes::RefersFeature => EDGE_TYPE_SITE_REFERS_RTE,
            EdgeTypes::SiteRefersRte => EDGE_TYPE_APPLICATION_REFERS_FEATURE,
            EdgeTypes::HasConnection => EDGE_TYPE_HAS_CONNECTION,
            EdgeTypes::HasConnections => EDGE_TYPE_HAS_CONNECTIONS,
            EdgeTypes::HasComponentSrc => EDGE_TYPE_HAS_COMPONENT_SRC,
            EdgeTypes::HasComponentDst => EDGE_TYPE_HAS_COMPONENT_DST,
            EdgeTypes::HasApplications => EDGE_TYPE_HAS_APPLICATIONS,
            EdgeTypes::HasDeployStages => EDGE_TYPE_HAS_DEPLOY_STAGES,
            EdgeTypes::HasDestroyStages => EDGE_TYPE_HAS_DESTROY_STAGES,
            EdgeTypes::ProvidesProvider => EDGE_TYPE_PROVIDES_PROVIDER,
            EdgeTypes::HasConnectionSrc => EDGE_TYPE_HAS_CONNECTION_SRC,
            EdgeTypes::HasConnectionDst => EDGE_TYPE_HAS_CONNECTION_DST,
            EdgeTypes::FeatureRefersSite => EDGE_TYPE_FEATURE_REFERS_SITE,
            EdgeTypes::ProvidesApplication => EDGE_TYPE_PROVIDES_APPLICATION,
            EdgeTypes::TestRefersApplication => EDGE_TYPE_TEST_REFERS_APPLICATION,
        }
    }
}

impl PropertyType {
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
        map.insert(VertexTuple(VertexTypes::Project.name().to_string(), VertexTypes::Dashboard.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Dashboard.name().to_string(), VertexTypes::DashboardProvider.name().to_string()), EdgeTypes::UsesProvider.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Applications.name().to_string()), EdgeTypes::HasApplications.name());
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
        map.insert(VertexTuple(VertexTypes::Site.name().to_string(), VertexTypes::Rte.name().to_string()), EdgeTypes::SiteRefersRte.name());
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
        map.insert(VertexTuple(VertexTypes::Test.name().to_string(), VertexTypes::Application.name().to_string()), EdgeTypes::TestRefersApplication.name());
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::HasDeployStages.name());
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::HasDestroyStages.name());
        map.insert(VertexTuple(VertexTypes::StageDeploy.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::NextStage.name());
        map.insert(VertexTuple(VertexTypes::StageDestroy.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::NextStage.name());
        map.insert(VertexTuple(VertexTypes::Applications.name().to_string(), VertexTypes::Application.name().to_string()), EdgeTypes::ProvidesApplication.name());
        map.insert(VertexTuple(VertexTypes::Application.name().to_string(), VertexTypes::Feature.name().to_string()), EdgeTypes::RefersFeature.name());
        map.insert(VertexTuple(VertexTypes::Features.name().to_string(), VertexTypes::Feature.name().to_string()), EdgeTypes::HasFeature.name());
        map.insert(VertexTuple(VertexTypes::Feature.name().to_string(), VertexTypes::Site.name().to_string()), EdgeTypes::FeatureRefersSite.name());
        map.insert(VertexTuple(VertexTypes::Scripts.name().to_string(), VertexTypes::Script.name().to_string()), EdgeTypes::Has.name());
        map
    };
    static ref EDGES_COUNT: usize = EDGE_TYPES.len();
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct VertexTuple(String, String);

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
struct RegressionConfigGenericCiStages {
    deploy: Vec<String>,
    destroy: Vec<String>,
}

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
struct RegressionConfigGenericCi {
    stages: RegressionConfigGenericCiStages,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigCiVariables {
    name: String,
    value: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigJobTemplates {
    name: String,
    variables: Vec<RegressionConfigCiVariables>,
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
    job_templates: Vec<RegressionConfigJobTemplates>,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigApplications {
    ci: RegressionConfigGenericCi,
    path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigEut {
    ci: RegressionConfigGenericCi,
    path: String,
    module: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigCollector {
    path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigFeatures {
    ci: RegressionConfigGenericCi,
    path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigRte {
    ci: RegressionConfigGenericCi,
    path: String,
    data_vars_path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigTests {
    ci: RegressionConfigGenericCi,
    path: String,
    data_vars_path: String,
    data_scripts_path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigVerificationsSummaries {
    path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigVerifications {
    ci: RegressionConfigGenericCi,
    path: String,
    data_vars_path: String,
    data_scripts_path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigDashboard {
    ci: RegressionConfigGenericCi,
    path: String,
    module: String,
    provider: String,
}

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
struct RegressionConfigProject {
    ci: RegressionConfigGenericCi,
    path: String,
    module: String,
    templates: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RegressionConfig {
    ci: RegressionConfigCi,
    eut: RegressionConfigEut,
    rte: RegressionConfigRte,
    tests: RegressionConfigTests,
    project: RegressionConfigProject,
    features: RegressionConfigFeatures,
    root_path: String,
    dashboard: RegressionConfigDashboard,
    collector: RegressionConfigCollector,
    applications: RegressionConfigApplications,
    verifications: RegressionConfigVerifications,
}

#[derive(Serialize, Debug)]
struct ActionsRenderContext {
    rtes: Vec<String>,
    sites: Vec<String>,
    tests: Vec<String>,
    features: Vec<String>,
    applications: Vec<String>,
    verifications: Vec<String>,
}

#[derive(Serialize, Debug)]
struct ProjectRenderContext {
    job: String,
    base: Map<String, Value>,
    module: Map<String, Value>,
    scripts: Vec<HashMap<String, Vec<String>>>,
}

#[derive(Serialize, Debug)]
struct ApplicationRenderContext {
    job: String,
    base: Map<String, Value>,
    module: Map<String, Value>,
    project: RegressionConfigProject,
    scripts: Vec<HashMap<String, Vec<String>>>,
}

#[derive(Default, Serialize, Debug)]
struct DashboardRenderContext {
    base: Map<String, Value>,
    module: Map<String, Value>,
    project: RegressionConfigProject,
    provider: Map<String, Value>,
    scripts: Vec<HashMap<String, Vec<String>>>,
}

#[derive(Serialize, Debug)]
struct CollectorRenderContext {
    job: String,
    eut: String,
    base: Map<String, Value>,
    module: Map<String, Value>,
    project: RegressionConfigProject,
    scripts: Vec<HashMap<String, Vec<String>>>,
}

#[derive(Serialize, Debug)]
struct FeatureRenderContext {
    job: String,
    eut: String,
    base: Map<String, Value>,
    module: Map<String, Value>,
    project: RegressionConfigProject,
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
    artifacts: Value,
}

#[derive(Serialize, Debug)]
struct RteVerificationRenderContext {
    ci: Map<String, Value>,
    test: String,
    rte: String,
    job: String,
    name: String,
    module: String,
    data: String,
    scripts: Vec<HashMap<String, Vec<String>>>,
}

#[derive(Serialize, Debug)]
struct RteTestRenderContext {
    ci: Map<String, Value>,
    rte: String,
    job: String,
    name: String,
    data: String,
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
    name: String,
    tests: Vec<RteTestRenderContext>,
    shares: Vec<RteProviderShareRenderContext>,
    components: Vec<RteComponentRenderContext>,
}

#[derive(Default, Serialize, Debug)]
struct ScriptEutRenderContext {
    rte: String,
    name: String,
    site: String,
    rtes: Vec<String>,
    index: usize,
    counter: usize,
    release: String,
    project: RegressionConfigProject,
    provider: String,
}

#[derive(Serialize, Debug)]
struct ScriptCollectorRenderContext {
    eut: String,
    name: String,
    data: String,
    module: String,
    project: RegressionConfigProject,
}

#[derive(Serialize, Debug)]
struct ScriptFeatureRenderContext {
    eut: String,
    name: String,
    data: String,
    module: String,
    release: String,
    project: RegressionConfigProject,
}

#[derive(Serialize, Debug)]
struct ScriptApplicationRenderContext {
    eut: String,
    name: String,
    release: String,
    project: RegressionConfigProject,
}

#[derive(Serialize, Debug)]
struct ScriptVerificationRenderContext {
    rte: String,
    name: String,
    data: String,
    module: String,
    provider: String,
    test_name: String,
    test_module: String,
}

#[derive(Serialize, Debug)]
struct ScriptTestRenderContext {
    rte: String,
    eut: String,
    name: String,
    data: String,
    module: String,
    project: RegressionConfigProject,
    provider: String,
    features: Vec<String>,
    refs: ObjectRefs,
}

#[derive(Serialize, Debug)]
struct ScriptRteRenderContext {
    eut: String,
    rte: String,
    site: String,
    release: String,
    project: RegressionConfigProject,
    provider: String,
    destinations: String,
}

#[derive(Serialize, Clone, Debug)]
struct ScriptRteSiteShareDataRenderContext {
    rte: String,
    name: String,
    index: usize,
    has_client: bool,
    has_server: bool,
}

#[derive(Serialize, Debug)]
struct ScriptRteSitesShareDataRenderContext {
    sites: HashMap<String, ScriptRteSiteShareDataRenderContext>,
}

#[derive(Serialize, Debug)]
struct ScriptRteProviderShareRenderContext {
    eut: String,
    rte: String,
    map: String,
    sites: String,
    counter: usize,
    project: RegressionConfigProject,
    provider: String,
}

#[derive(Serialize, Debug)]
struct ScriptDashboardRenderContext {
    name: String,
    module: String,
    project: RegressionConfigProject,
}

#[derive(Serialize, Debug)]
struct ScriptProjectRenderContext {
    project: RegressionConfigProject,
    release: String,
}

#[derive(Serialize, Debug)]
struct ObjectRefs {
    refs: HashMap<String, Vec<String>>,
}

impl ObjectRefs {
    pub fn new(object: &Map<String, Value>) -> Self {
        let mut refs: HashMap<String, Vec<String>> = Default::default();

        for r#ref in object.get(KEY_REFS).unwrap().as_array().unwrap().iter() {
            let module = r#ref.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
            let r#type = r#ref.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap();

            return if refs.get(r#type).is_none() {
                refs.insert(r#type.to_string(), vec!(module));
                Self { refs: refs.clone() }
            } else {
                let items: &mut Vec<String> = refs.get_mut(r#type).unwrap();
                items.push(module);
                Self { refs: refs.clone() }
            };
        }

        Self { refs: refs.clone() }
    }
}

#[typetag::serialize(tag = "type")]
pub trait RenderContext {
    fn as_any(&self) -> &dyn Any;
}

pub trait Renderer<'a> {
    fn gen_render_ctx(&self, config: &RegressionConfig, ctx: Vec<HashMap<String, Vec<String>>>) -> Box<dyn RenderContext>;
    fn gen_script_render_ctx(&self, config: &RegressionConfig) -> Vec<HashMap<String, Vec<String>>>;
}

pub trait ScriptRenderContext {}

impl ScriptRenderContext for ScriptEutRenderContext {}

impl ScriptRenderContext for ScriptRteRenderContext {}

impl ScriptRenderContext for ScriptTestRenderContext {}

impl ScriptRenderContext for ScriptProjectRenderContext {}

impl ScriptRenderContext for ScriptFeatureRenderContext {}

impl ScriptRenderContext for ScriptCollectorRenderContext {}

impl ScriptRenderContext for ScriptDashboardRenderContext {}

impl ScriptRenderContext for ScriptApplicationRenderContext {}

impl ScriptRenderContext for ScriptVerificationRenderContext {}

impl ScriptRenderContext for ScriptRteProviderShareRenderContext {}

pub fn render_script(context: &(impl ScriptRenderContext + serde::Serialize), input: &str) -> String {
    info!("Render script context...");
    let ctx = Context::from_serialize(context);
    let rendered = Tera::one_off(input, &ctx.unwrap(), false).unwrap();
    info!("Render script context -> Done.");
    rendered
}

pub fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge_json(a.entry(k.clone()).or_insert(Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

struct RteCtxParameters<'a> {
    rte: &'a VertexProperties,
    config: &'a RegressionConfig,
    project: RegressionConfigProject,
    eut: &'a VertexProperties,
    rte_name: String,
    features: Vec<String>,
    provider: Vec<&'a VertexProperties>,
    rte_crcs: &'a mut RteRenderContext,
}

trait RteCharacteristics: {
    fn init(&self, r_o: &Vertex);
    fn build_ctx(&self, rte: &VertexProperties, site_count: usize, srsd: &mut HashMap<String, ScriptRteSitesShareDataRenderContext>);
    fn build_conn_ctx(&self, params: RteCtxParameters);
}

struct RteTypeA<'a> {
    db: &'a Db,
}

impl<'a> RteCharacteristics for RteTypeA<'a> {
    fn init(&self, r_o: &Vertex) {
        error!("RTE TYPE A init connection components --> {:?}", &r_o);
        // Connection -> Component
        let _c = self.db.get_object_neighbour_out(&r_o.id, EdgeTypes::HasConnections);
        let connections = self.db.get_object_neighbours_out(&_c.id, EdgeTypes::HasConnection);
        let _p = self.db.get_object_neighbour_out(&r_o.id, EdgeTypes::NeedsProvider);
        let rte_provider = self.db.get_object_neighbours_with_properties_out(&_p.id, EdgeTypes::ProvidesProvider);

        for c in connections.iter() {
            let c_s = self.db.get_object_neighbour_with_properties_out(&c.id, EdgeTypes::HasConnectionSrc).unwrap();
            let site = self.db.get_object_neighbour_out(&c_s.vertex.id, EdgeTypes::RefersSite);
            let site_provider = self.db.get_object_neighbour_with_properties_out(&site.id, EdgeTypes::UsesProvider).unwrap();
            let s_p_name = site_provider.props.get(PropertyType::Base.index()).unwrap().value.as_object().
                unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

            let _c_d_s: Vec<VertexProperties> = self.db.get_object_neighbours_with_properties_out(&c_s.vertex.id, EdgeTypes::HasConnectionDst);
            for p in rte_provider.iter() {
                let _components = self.db.get_object_neighbour_out(&p.vertex.id, EdgeTypes::HasComponents);
                let component_src = self.db.get_object_neighbour_out(&_components.id, EdgeTypes::HasComponentSrc);
                let r_p_name = p.props.get(PropertyType::Module.index()).unwrap().value.as_object().
                    unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                if s_p_name == r_p_name {
                    self.db.create_relationship(&c_s.vertex, &component_src);
                }
            }

            //CONNECTION DSTs
            for c_d in _c_d_s.iter() {
                for p in rte_provider.iter() {
                    let _components = self.db.get_object_neighbour_out(&p.vertex.id, EdgeTypes::HasComponents);
                    let component_dst = self.db.get_object_neighbour_out(&_components.id, EdgeTypes::HasComponentDst);
                    self.db.create_relationship(&c_d.vertex, &component_dst);
                }
            }
        }
        info!("Init rte type a connection components -> Done.");
    }

    fn build_ctx(&self, rte: &VertexProperties, mut site_count: usize, srsd: &mut HashMap<String, ScriptRteSitesShareDataRenderContext>) {
        error!("RTE TYPE A build ctx --> {:?}", rte);

        let rte_name = rte.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
        srsd.insert(rte_name.to_string(), ScriptRteSitesShareDataRenderContext { sites: Default::default() });

        let _c = self.db.get_object_neighbour_out(&rte.vertex.id, EdgeTypes::HasConnections);
        let connections = self.db.get_object_neighbours_with_properties_out(&_c.id, EdgeTypes::HasConnection);

        for conn in connections.iter() {
            let src = self.db.get_object_neighbour_with_properties_out(&conn.vertex.id, EdgeTypes::HasConnectionSrc).unwrap();
            let src_site = self.db.get_object_neighbour_with_properties_out(&src.vertex.id, EdgeTypes::RefersSite).unwrap();
            let src_site_name = src_site.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

            match srsd.get_mut(rte_name) {
                Some(rte) => {
                    match rte.sites.get_mut(src_site_name) {
                        Some(site) => {
                            if !site.has_client {
                                site.has_client = true
                            }
                        }
                        None => {
                            let srssd_rc = ScriptRteSiteShareDataRenderContext {
                                rte: rte_name.to_string(),
                                name: src_site_name.to_string(),
                                index: site_count,
                                has_client: true,
                                has_server: false,
                            };

                            rte.sites.entry(src_site_name.to_string()).or_insert(srssd_rc);
                            site_count += 1;

                            let dsts = self.db.get_object_neighbours_with_properties_out(&src.vertex.id, EdgeTypes::HasConnectionDst);
                            for dst in dsts.iter() {
                                let dst_site = self.db.get_object_neighbour_with_properties_out(&dst.vertex.id, EdgeTypes::RefersSite).unwrap();
                                let dst_site_name = dst_site.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                                match rte.sites.get_mut(dst_site_name) {
                                    Some(site) => {
                                        if !site.has_server {
                                            site.has_server = true
                                        }
                                    }
                                    None => {
                                        let srssd_rc = ScriptRteSiteShareDataRenderContext {
                                            rte: rte_name.to_string(),
                                            name: dst_site_name.to_string(),
                                            index: site_count,
                                            has_client: false,
                                            has_server: true,
                                        };
                                        rte.sites.entry(dst_site_name.to_string()).or_insert(srssd_rc);
                                        site_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                None => error!("RTE {} does not exist", rte_name),
            }
        }
    }
    fn build_conn_ctx(&self, params: RteCtxParameters) {
        error!("RTE TYPE A build conn ctx --> {}", params.rte_name);
        //Connection DST rt set
        let mut server_destinations: HashSet<String> = HashSet::new();

        let _c = self.db.get_object_neighbour_out(&params.rte.vertex.id, EdgeTypes::HasConnections);
        let connections = self.db.get_object_neighbours_with_properties_out(&_c.id, EdgeTypes::HasConnection);
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
            let rte_job_name = format!("{}_{}_{}_{}_{}_{}_{}", params.project.module, KEY_RTE, params.rte_name, &connection_name, &src_p_name, &src_name, &comp_src_name).replace('_', "-");

            //Process site_to_rte_map
            let mut _rtes: HashSet<String> = HashSet::new();
            _rtes.insert(params.rte_name.to_string());
            site_to_rte_map.entry(src_site_name.to_string()).or_insert(_rtes);

            //Process rte src component scripts
            let scripts_path = comp_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
            let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();

            //Process client destination list
            let mut client_destinations: HashSet<String> = HashSet::new();
            let dsts = self.db.get_object_neighbours_with_properties_out(&src.vertex.id, EdgeTypes::HasConnectionDst);
            for dst in dsts.iter() {
                client_destinations.insert(dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string());
            }

            for p in params.provider.iter() {
                let p_name = p.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                for script in comp_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                    if src_p_name == p_name {
                        let path = format!("{}/{}/{}/{}/{}/{}/{}", params.config.root_path, params.config.rte.path, params.rte_name, scripts_path, p_name, comp_src_name, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                        let contents = std::fs::read_to_string(&path).expect("panic while opening rte apply.script file");
                        let ctx = ScriptRteRenderContext {
                            rte: params.rte_name.to_string(),
                            eut: params.eut.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                            site: src_site_name.to_string(),
                            release: "".to_string(),
                            provider: p_name.to_string(),
                            project: params.config.project.clone(),
                            destinations: serde_json::to_string(&client_destinations).unwrap(),
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
            }

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
                let rte_job_name = format!("{}_{}_{}_{}_{}_{}_{}", params.project.module, KEY_RTE, &params.rte_name, &connection_name, &dst_p_name, &dst_name, &comp_dst_name).replace('_', "-");

                //Process server destination list
                let rt_dsts = self.db.get_object_neighbours_with_properties_in(&dst.vertex.id, EdgeTypes::HasConnectionDst);
                for dst in rt_dsts.iter() {
                    server_destinations.insert(dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string());
                }

                //Process rte dst component scripts
                let scripts_path = comp_dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();

                for p in params.provider.iter() {
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
                                provider: p_name.to_string(),
                                project: params.config.project.clone(),
                                destinations: serde_json::to_string(&server_destinations).unwrap(),
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
                }

                let rte_crc = RteComponentRenderContext {
                    job: rte_job_name.to_string(),
                    rte: params.rte_name.to_string(),
                    site: dst_site_name.to_string(),
                    name: comp_dst_name.to_string(),
                    provider: dst_p_name.to_string(),
                    scripts,
                };
                params.rte_crcs.components.push(rte_crc);
            }

            //Tests
            let tests_p = self.db.get_object_neighbours_with_properties_out(&src.vertex.id, EdgeTypes::Runs);
            for t in tests_p.iter() {
                let t_job_name = format!("{}_{}_{}_{}",
                                         params.project.module,
                                         KEY_TEST,
                                         src_name,
                                         t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap()
                ).replace('_', "-");

                //Process test scripts
                let t_p_base = t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();
                let t_p_module = t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();
                let t_name = t_p_base.get(KEY_NAME).unwrap().as_str().unwrap();
                let t_module = t_p_base.get(KEY_MODULE).unwrap().as_str().unwrap();
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
                        module: t_module.to_string(),
                        project: params.config.project.clone(),
                        provider: src_name.to_string(),
                        features: params.features.to_vec(),
                        refs: ObjectRefs::new(t_p_base),
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

                    //Process test scripts
                    let v_name = v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                    let v_module = v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                    let v_data = v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_DATA).unwrap().as_str().unwrap();
                    let scripts_path = v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                    let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
                    for script in v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                        let path = format!("{}/{}/{}/{}/{}", params.config.root_path, params.config.verifications.path, v_module, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                        let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
                        let ctx = ScriptVerificationRenderContext {
                            rte: params.rte_name.to_string(),
                            name: v_name.to_string(),
                            data: v_data.to_string(),
                            module: v_module.to_string(),
                            provider: src_name.to_string(),
                            test_name: t_name.to_string(),
                            test_module: t_module.to_string(),
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
    fn init(&self, r_o: &Vertex) {
        error!("RTE TYPE B init connection component --> {:?}", &r_o.t.as_str());
        // Connection -> Component
        let _c = self.db.get_object_neighbour_out(&r_o.id, EdgeTypes::HasConnections);
        let connections = self.db.get_object_neighbours_out(&_c.id, EdgeTypes::HasConnection);
        let _p = self.db.get_object_neighbour_out(&r_o.id, EdgeTypes::NeedsProvider);
        let rte_available_provider = self.db.get_object_neighbours_with_properties_out(&_p.id, EdgeTypes::ProvidesProvider);
        let rte_p = self.db.get_object_with_properties(&r_o.id);
        let rte_active_provider = rte_p.props.get(PropertyType::Base.index()).unwrap().value.get(KEY_PROVIDER).unwrap().as_array().unwrap();

        for p in rte_available_provider.iter() {
            let _p = self.db.get_object_with_properties(&p.vertex.id);
            let p_name = _p.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

            if rte_active_provider.contains(&to_value(p_name).unwrap()) {
                for c in connections.iter() {
                    let c_s = self.db.get_object_neighbour_with_properties_out(&c.id, EdgeTypes::HasConnectionSrc).unwrap();
                    let _c_d_s: Vec<VertexProperties> = self.db.get_object_neighbours_with_properties_out(&c_s.vertex.id, EdgeTypes::HasConnectionDst);
                    let _components = self.db.get_object_neighbour_out(&p.vertex.id, EdgeTypes::HasComponents);
                    let component_src = self.db.get_object_neighbour_out(&_components.id, EdgeTypes::HasComponentSrc);
                    self.db.create_relationship(&c_s.vertex, &component_src);
                }
            }
        }
        info!("Init rte type b connection components-> Done.");
    }

    fn build_ctx(&self, rte: &VertexProperties, mut site_count: usize, srsd: &mut HashMap<String, ScriptRteSitesShareDataRenderContext>) {
        error!("RTE TYPE B build ctx --> {:?}", rte.vertex.t.as_str());
        let rte_name = rte.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
        srsd.insert(rte_name.to_string(), ScriptRteSitesShareDataRenderContext { sites: Default::default() });

        let _c = self.db.get_object_neighbour_out(&rte.vertex.id, EdgeTypes::HasConnections);
        let connections = self.db.get_object_neighbours_with_properties_out(&_c.id, EdgeTypes::HasConnection);

        for _conn in connections.iter() {
            match srsd.get_mut(rte_name) {
                Some(rte) => {
                    let srssd_rc = ScriptRteSiteShareDataRenderContext {
                        rte: rte_name.to_string(),
                        name: "dummy".to_string(),
                        index: site_count,
                        has_client: true,
                        has_server: false,
                    };
                    rte.sites.entry("dummy".to_string()).or_insert(srssd_rc);
                    site_count += 1;
                }
                None => error!("RTE {} does not exist", rte_name),
            }
        }
    }

    fn build_conn_ctx(&self, params: RteCtxParameters) {
        error!("RTE TYPE B build conn ctx --> {}", params.rte_name);
        let _c = self.db.get_object_neighbour_out(&params.rte.vertex.id, EdgeTypes::HasConnections);
        let connections = self.db.get_object_neighbours_with_properties_out(&_c.id, EdgeTypes::HasConnection);

        for conn in connections.iter() {
            let conn_src = self.db.get_object_neighbour_with_properties_out(&conn.vertex.id, EdgeTypes::HasConnectionSrc).unwrap();
            let conn_name = conn.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            let conn_src_name = conn_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            let comp_src = self.db.get_object_neighbour_with_properties_out(&conn_src.vertex.id, EdgeTypes::HasComponentSrc).unwrap();
            let comp_src_name = &comp_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            let components = self.db.get_object_neighbours_in(&comp_src.vertex.id, EdgeTypes::HasComponentSrc);
            let mut component_provider = String::new();

            for item in components.iter() {
                match item {
                    k if k.t.to_string() == KEY_COMPONENTS => {
                        let p = self.db.get_object_neighbours_with_properties_in(&k.id, EdgeTypes::HasComponents);
                        component_provider = p.get(0).unwrap().props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string();
                    }
                    &_ => {}
                }
            }

            //Process rte src component scripts
            let scripts_path = comp_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
            for p in params.provider.iter() {
                let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
                let p_name = p.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let rte_job_name = format!("{}_{}_{}_{}_{}", params.project.module, KEY_RTE, params.rte_name, &p_name, &conn_name).replace('_', "-");

                for script in comp_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                    let path = format!("{}/{}/{}/{}/{}/{}/{}", params.config.root_path, params.config.rte.path, params.rte_name, scripts_path, p_name, comp_src_name, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                    let contents = std::fs::read_to_string(&path).expect("panic while opening rte apply.script file");
                    let ctx = ScriptRteRenderContext {
                        rte: params.rte_name.to_string(),
                        eut: params.eut.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                        site: "".to_string(),
                        release: "".to_string(),
                        project: params.config.project.clone(),
                        provider: p_name.to_string(),
                        destinations: "".to_string(),
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

                let rte_crc = RteComponentRenderContext {
                    job: rte_job_name.clone(),
                    rte: params.rte_name.to_string(),
                    name: comp_src_name.to_string(),
                    site: "".to_string(),
                    provider: p_name.to_string(),
                    scripts,
                };
                params.rte_crcs.components.push(rte_crc);
            }

            //Tests
            let tests_p = self.db.get_object_neighbours_with_properties_out(&conn_src.vertex.id, EdgeTypes::Runs);
            for t in tests_p.iter() {
                //Process test scripts
                let t_p_base = t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap();
                let t_p_module = t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();
                let t_name = t_p_base.get(KEY_NAME).unwrap().as_str().unwrap();
                let t_module = t_p_base.get(KEY_MODULE).unwrap().as_str().unwrap();
                let t_job_name = format!("{}_{}_{}",
                                         params.project.module,
                                         KEY_TEST,
                                         t_name).replace('_', "-");
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
                        module: t_module.to_string(),
                        project: params.config.project.clone(),
                        provider: component_provider.to_string(),
                        features: params.features.to_vec(),
                        refs: ObjectRefs::new(t_p_base),
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
                    //Process test scripts
                    let v_p_base = v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap();
                    let v_p_module = v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();
                    let v_name = v_p_base.get(KEY_NAME).unwrap().as_str().unwrap();
                    let v_data = v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_DATA).unwrap().as_str().unwrap();
                    let v_module = v_p_base.get(KEY_MODULE).unwrap().as_str().unwrap();
                    let v_job_name = format!("{}_{}_{}", params.project.module, KEY_VERIFICATION, v_name).replace('_', "-");
                    let scripts_path = v_p_module.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                    let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
                    for script in v_p_module.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                        let path = format!("{}/{}/{}/{}/{}", params.config.root_path, params.config.verifications.path, v_module, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                        let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
                        let ctx = ScriptVerificationRenderContext {
                            rte: params.rte_name.to_string(),
                            name: v_name.to_string(),
                            data: v_data.to_string(),
                            module: v_module.to_string(),
                            provider: component_provider.to_string(),
                            test_name: t_name.to_string(),
                            test_module: t_module.to_string(),
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
                        test: t_name.to_string(),
                        rte: params.rte_name.to_string(),
                        job: v_job_name,
                        name: v_p_base.get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                        module: v_p_base.get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                        data: v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_DATA).unwrap().as_str().unwrap().to_string(),
                        scripts,
                    };
                    verifications.push(rte_vrc);
                }

                let rterc = RteTestRenderContext {
                    ci: t_p_base.get(KEY_CI).unwrap().as_object().unwrap().clone(),
                    rte: params.rte_name.to_string(),
                    job: t_job_name,
                    name: t_p_base.get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                    data: t_p_base.get(KEY_DATA).unwrap().as_str().unwrap().to_string(),
                    module: t_p_base.get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                    provider: conn_src_name.to_string(),
                    scripts,
                    verifications,
                };
                params.rte_crcs.tests.push(rterc);
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

    fn init(&self, r_o: &Vertex) {
        self.rte.init(r_o);
    }

    fn build_ctx(&self, rte: &VertexProperties, site_count: usize, srsd: &mut HashMap<String, ScriptRteSitesShareDataRenderContext>) {
        self.rte.build_ctx(rte, site_count, srsd);
    }

    fn build_conn_ctx(&self, params: RteCtxParameters) {
        self.rte.build_conn_ctx(params);
    }
}

pub struct Regression<'a> {
    db: &'a Db,
    pub config: RegressionConfig,
    pub root_path: String,
}

impl<'a> Regression<'a> {
    pub fn new(db: &'a Db, path: &str, file: &str) -> Self {
        Regression {
            db,
            config: Regression::load_regression_config(path, file),
            root_path: path.to_string(),
        }
    }

    pub fn init(&self) -> Uuid {
        // Project
        let project = Project::init(self.db, &self.config, &mut vec![], &self.config.project.module, 0);

        // Dashboard
        let dashboard = Dashboard::init(self.db, &self.config, &mut project.get_id_path().get_vec(), "", 0);
        self.db.create_relationship(&project.get_object(), &dashboard.get_object());

        // Ci
        let ci = Ci::init(self.db, &self.config, &mut project.get_id_path().get_vec(), "", 0);
        self.db.create_relationship(&project.get_object(), &ci.get_object());

        // Eut
        let eut = Eut::init(self.db, &self.config, &mut project.get_id_path().get_vec(), &self.config.eut.module, 0);
        let eut_module_cfg = eut.get_module_cfg();
        self.db.create_relationship(&project.get_object(), &eut.get_object());
        let eut_providers = Providers::init(self.db, &self.config, &mut eut.get_id_path().get_vec(), "", 0);
        self.db.create_relationship(&eut.get_object(), &eut_providers.get_object());

        for k in EUT_KEY_ORDER.iter() {
            let obj = eut_module_cfg.get(*k).unwrap();
            match *k {
                k if k == KEY_PROVIDER => {
                    for p in obj.as_array().unwrap().iter() {
                        let eut_provider = EutProvider::init(self.db, &self.config, &mut eut.get_id_path().get_vec(), &p.as_str().unwrap(), 0);
                        self.db.create_relationship(&eut_providers.get_object(), &eut_provider.get_object());
                        //eut_provider.add_base_properties(json!({KEY_NAME: &p.as_str().unwrap()}));
                    }
                }
                k if k == KEY_SITES => {
                    let o = Sites::init(self.db, &self.config, &mut eut.get_id_path().get_vec(), "", 2);
                    self.db.create_relationship(&eut.get_object(), &o.get_object());
                    let _p = self.db.get_object_neighbour_out(&eut.get_id(), EdgeTypes::HasProviders);
                    let provider = self.db.get_object_neighbours_with_properties_out(&_p.id, EdgeTypes::ProvidesProvider);
                    let mut id_name_map: HashMap<&str, Uuid> = HashMap::new();

                    //Generate provider name to vertex id map
                    for p in provider.iter() {
                        id_name_map.insert(p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().
                            get(KEY_NAME).unwrap().as_str().unwrap(), p.vertex.id);
                    }

                    for (site_name, site_attr) in obj.as_object().unwrap().iter() {
                        let site_count = site_attr.as_object().unwrap().get(KEY_COUNT).unwrap().as_i64().unwrap();
                        match site_count.cmp(&1i64) {
                            Ordering::Equal => {
                                let s_o = Site::init(&self.db, site_attr, &mut o.get_id_path().get_vec(), site_name, 0);
                                self.db.create_relationship(&o.get_object(), &s_o.get_object());
                                let provider = &site_attr.as_object().unwrap().get(KEY_PROVIDER).unwrap().as_str().unwrap();
                                let p_o = self.db.get_object(id_name_map.get(provider).unwrap());
                                self.db.create_relationship(&s_o.get_object(), &p_o);
                                self.db.add_object_properties(&s_o.get_object(), &json!({KEY_NAME: site_name}), PropertyType::Base);
                            }
                            Ordering::Greater => {
                                for c in 1..=site_count {
                                    let s_o = Site::init(&self.db, site_attr, &mut o.get_id_path().get_vec(),
                                                         &*format!("{}_{}", site_name, c), 0);
                                    self.db.create_relationship(&o.get_object(), &s_o.get_object());
                                    let provider = &site_attr.as_object().unwrap().get(KEY_PROVIDER).unwrap().as_str().unwrap();
                                    let p_o = self.db.get_object(id_name_map.get(provider).unwrap());
                                    self.db.create_relationship(&s_o.get_object(), &p_o);
                                    self.db.add_object_properties(&s_o.get_object(),
                                                                  &json!({KEY_NAME: format!("{}_{}",
                                                                      site_name, c)}),
                                                                  PropertyType::Base);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                k if k == KEY_FEATURES => {
                    let o = Features::init(self.db, &self.config, &mut eut.get_id_path().get_vec(), "", 2);
                    self.db.create_relationship(&eut.get_object(), &o.get_object());

                    for f in obj.as_array().unwrap().iter() {
                        let f_module = f.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                        let f_sites = f.as_object().unwrap().get(KEY_SITES).unwrap().as_array().unwrap();
                        let _sites = self.db.get_object_neighbour_out(&&eut.get_id(), EdgeTypes::HasSites);
                        let sites = self.db.get_object_neighbours_with_properties_out(&_sites.id, EdgeTypes::HasSite);

                        let f_o = Feature::init(self.db, &self.config, f, &mut o.get_id_path().get_vec(), f_module, 0);
                        self.db.create_relationship(&o.get_object(), &f_o.get_object());

                        //Feature -> Site
                        for f_site in f_sites {
                            let re = Regex::new(f_site.as_str().unwrap()).unwrap();
                            for site in sites.iter() {
                                let site_name = site.props.get(PropertyType::Base.index()).unwrap().value.as_object().
                                    unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                                if let Some(_t) = re.captures(site_name) {
                                    self.db.create_relationship(&f_o.get_object(), &site.vertex);
                                }
                            }
                        }
                    }
                }
                k if k == KEY_APPLICATIONS => {
                    let o = Applications::init(self.db, &self.config, &mut eut.get_id_path().get_vec(), "", 2);
                    self.db.create_relationship(&eut.get_object(), &o.get_object());

                    for a in obj.as_array().unwrap().iter() {
                        let a_module = a.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                        let a_o = Application::init(self.db, &self.config, a, &mut o.get_id_path().get_vec(), a_module, 0);
                        self.db.create_relationship(&o.get_object(), &a_o.get_object());

                        //Build rel Application --> Feature
                        let props = a_o.get_base_properties();
                        let refs = props.get("refs").unwrap().as_array().unwrap();

                        for r in refs {
                            let v_type = VertexTypes::get_type_by_key(r.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap());
                            let ref_module = r.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();

                            match v_type {
                                VertexTypes::Feature => {
                                    let features = Features::load(&self.db, &eut.get_object(), &self.config);

                                    for f in features {
                                        if f.get_module_properties().get(KEY_NAME).unwrap().as_str().unwrap() == ref_module {
                                            self.db.create_relationship(&a_o.get_object(), &f.get_object());
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                k if k == KEY_RTES => {
                    let o_rtes = Rtes::init(&self.db, &self.config, &mut eut.get_id_path().get_vec(), "", 1);
                    self.db.create_relationship(&eut.get_object(), &o_rtes.get_object());

                    for rte in obj.as_array().unwrap().iter() {
                        /*let (r_o, _id_path) = self.db.create_object_and_init(VertexTypes::Rte, &mut o_rtes.get_id_path().get_vec(),
                                                                             &rte.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap(),
                                                                             0);*/
                        let r_o = Rte::init(&self.db, &self.config, rte, &mut o_rtes.get_id_path().get_vec(), &rte.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap(), 0);
                        self.db.create_relationship(&o_rtes.get_object(), &r_o.get_object());
                        //RTE -> Providers
                        let (rte_p_o, _id_path) = self.db.create_object_and_init(VertexTypes::Providers, &mut r_o.get_id_path().get_vec(), "", 0);
                        let rte_p_o_p = self.db.get_object_with_properties(&rte_p_o.id);
                        self.db.create_relationship(&r_o.get_object(), &rte_p_o);

                        //RTE -> Features
                        let eut_f_o = self.db.get_object_neighbour_out(&&eut.get_id(), EdgeTypes::HasFeatures);
                        self.db.create_relationship(&r_o.get_object(), &eut_f_o);

                        //Rte
                        for (k, v) in rte.as_object().unwrap().iter() {
                            match k {
                                k if k == KEY_MODULE => {
                                    let r_o_p = self.db.get_object_properties(&r_o.get_object()).unwrap().props;
                                    let mut p = r_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                    p.insert(k.clone(), v.clone());
                                    self.db.add_object_properties(&r_o.get_object(), &p, PropertyType::Base);
                                }
                                // Active Provider
                                k if k == KEY_PROVIDER => {
                                    let r_o_p = self.db.get_object_properties(&r_o.get_object()).unwrap().props;
                                    let mut p = r_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                    p.insert(k.clone(), v.clone());
                                    self.db.add_object_properties(&r_o.get_object(), &p, PropertyType::Base);
                                }
                                //Connections
                                k if k == KEY_CONNECTIONS => {
                                    let cs_o = Connections::init(&self.db, &self.config, &mut o_rtes.get_id_path().get_vec(), "", 1);
                                    self.db.create_relationship(&r_o.get_object(), &cs_o.get_object());

                                    for item in v.as_array().unwrap().iter() {
                                        //Connection
                                        let c_name = item.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                                        let c_o = Connection::init(&self.db, &self.config, &json!({KEY_NAME: c_name}), &mut cs_o.get_id_path().get_vec(), "", 0);
                                        self.db.create_relationship(&cs_o.get_object(), &c_o.get_object());

                                        //Connection Source
                                        let source = item.as_object().unwrap().get(KEY_SOURCE).unwrap().as_str().unwrap();
                                        let src_o = ConnectionSource::init(&self.db, &self.config,
                                                                           &json!({KEY_NAME: &source, KEY_RTE: &rte.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap()}), &mut c_o.get_id_path().get_vec(),
                                                                           "", 0);
                                        self.db.create_relationship(&c_o.get_object(), &src_o.get_object());
                                        let _sites = self.db.get_object_neighbour_out(&&eut.get_id(), EdgeTypes::HasSites);
                                        let sites = self.db.get_object_neighbours_with_properties_out(&_sites.id, EdgeTypes::HasSite);

                                        //Connection Source -> Site
                                        for s in sites.iter() {
                                            let site_name = s.props.get(PropertyType::Base.index()).unwrap().value.as_object().
                                                unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                                            if site_name == source {
                                                self.db.create_relationship(&src_o.get_object(), &s.vertex);
                                                //site --> rte
                                                self.db.create_relationship(&s.vertex, &r_o.get_object());
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
                                                    let dst_o = ConnectionDestination::init(&self.db,
                                                                                            &self.config,
                                                                                            &json!({KEY_NAME: &d,
                                                        KEY_RTE: &rte.as_object().unwrap().get(KEY_MODULE)
                                                        .unwrap().as_str().unwrap()}), &mut c_o.get_id_path().get_vec(),
                                                                                            "", 0);

                                                    self.db.create_relationship(&src_o.get_object(), &dst_o.get_object());
                                                    self.db.add_object_properties(&dst_o.get_object(), &json!({KEY_NAME: &d,
                                                        KEY_RTE: &rte.as_object().unwrap().get(KEY_MODULE)
                                                        .unwrap().as_str().unwrap()}),
                                                                                  PropertyType::Base);
                                                    //Connection Destination -> Site
                                                    self.db.create_relationship(&dst_o.get_object(), &site.vertex);
                                                    //site --> rte
                                                    self.db.create_relationship(&site.vertex, &r_o.get_object());
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

                                            let t_o = Test::init(&self.db, &self.config,
                                                                 &test, &mut c_o.get_id_path().get_vec(),
                                                                 test[KEY_MODULE].as_str().unwrap(),
                                                                 _index);
                                            self.db.create_relationship(&src_o.get_object(), &t_o.get_object());

                                            for (k, v) in test.as_object().unwrap().iter() {
                                                match k {
                                                    k if k == KEY_REFS => {
                                                        let applications = Applications::load_collection(&self.db, &eut.get_object(), &self.config);

                                                        // Ref Test -> Application
                                                        if v.as_array().unwrap().len() > 0 {
                                                            for r#ref in v.as_array().unwrap().iter() {
                                                                match VertexTypes::get_type_by_key(r#ref.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap()) {
                                                                    VertexTypes::Application => {
                                                                        let application = Applications::load_application(&self.db, &applications.get_object(), &r#ref.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap(), &self.config);
                                                                        match application {
                                                                            Some(a) => {
                                                                                self.db.create_relationship(&t_o.get_object(), &a.get_object());
                                                                            }
                                                                            None => error!("no application object found")
                                                                        }
                                                                    }
                                                                    _ => {}
                                                                }
                                                            }
                                                        }
                                                    }
                                                    k if k == KEY_VERIFICATIONS => {
                                                        for v in v.as_array().unwrap().iter() {
                                                            let v_module = v.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                                                            let v_o = Verification::init(&self.db, &self.config, &v, &mut t_o.get_id_path().get_vec(), v_module, 0);
                                                            self.db.create_relationship(&t_o.get_object(), &v_o.get_object());
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

                        //Rte module cfg
                        //let r_p = self.db.get_object_properties(&r_o.get_object()).unwrap().props;
                        //let module = r_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                        //let cfg = self.load_object_config(VertexTypes::get_name_by_object(&r_o.get_object()), module);
                        let rte_module_cfg = r_o.get_module_properties();
                        let _rte_providers_id_path = rte_p_o_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();

                        for (k, v) in rte_module_cfg.iter() {
                            match k {
                                k if k == KEY_PROVIDER => {
                                    for (p, v) in v.as_object().unwrap().iter() {
                                        let mut rte_providers_id_path: Vec<String> = _rte_providers_id_path.iter().map(|c| c.as_str().unwrap().to_string()).collect();
                                        let (o, _id_path) = self.db.create_object_and_init(VertexTypes::RteProvider,
                                                                                           &mut rte_providers_id_path,
                                                                                           p, 0);
                                        self.db.create_relationship(&rte_p_o, &o);
                                        self.db.add_object_properties(&o, &json!({KEY_NAME: p}), PropertyType::Module);

                                        for (k, v) in v.as_object().unwrap().iter() {
                                            match k {
                                                k if k == KEY_CI => {
                                                    let (p_ci_o, _id_path) = self.db.create_object_and_init(VertexTypes::Ci,
                                                                                                            &mut rte_providers_id_path,
                                                                                                            "", 0);
                                                    self.db.create_relationship(&o, &p_ci_o);
                                                    self.db.add_object_properties(&p_ci_o, &v.as_object().unwrap(), PropertyType::Base);
                                                }
                                                k if k == KEY_SHARE => {
                                                    let (s_o, _id_path) = self.db.create_object_and_init(VertexTypes::Share, &mut rte_providers_id_path, "", 0);
                                                    self.db.create_relationship(&o, &s_o);
                                                    self.db.add_object_properties(&s_o, &v.as_object().unwrap(), PropertyType::Base);
                                                }
                                                k if k == KEY_COMPONENTS => {
                                                    let (c_o, _id_path) = self.db.create_object_and_init(VertexTypes::Components, &mut rte_providers_id_path, "", 1);
                                                    self.db.create_relationship(&o, &c_o);

                                                    for (k, v) in v.as_object().unwrap().iter() {
                                                        match k {
                                                            k if k == KEY_SRC => {
                                                                let (c_src_o, _id_path) = self.db.create_object_and_init(VertexTypes::ComponentSrc, &mut rte_providers_id_path, "", 0);
                                                                self.db.create_relationship(&c_o, &c_src_o);

                                                                for (k, v) in v.as_object().unwrap().iter() {
                                                                    let c_src_o_p = self.db.get_object_properties(&c_src_o).unwrap().props;
                                                                    match k {
                                                                        k if k == KEY_NAME => {
                                                                            let mut p = c_src_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                            p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                            self.db.add_object_properties(&c_src_o, &p, PropertyType::Base);
                                                                        }
                                                                        k if k == KEY_SCRIPTS => {
                                                                            let mut p = c_src_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                            p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                            self.db.add_object_properties(&c_src_o, &p, PropertyType::Base);
                                                                        }
                                                                        k if k == KEY_SCRIPTS_PATH => {
                                                                            let mut p = c_src_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                            p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                            self.db.add_object_properties(&c_src_o, &p, PropertyType::Base);
                                                                        }
                                                                        _ => {}
                                                                    }
                                                                }
                                                            }
                                                            k if k == KEY_DST => {
                                                                let (c_dst_o, _id_path) = self.db.create_object_and_init(VertexTypes::ComponentDst, &mut rte_providers_id_path, "", 0);
                                                                self.db.create_relationship(&c_o, &c_dst_o);

                                                                for (k, v) in v.as_object().unwrap().iter() {
                                                                    let c_dst_o_p = self.db.get_object_properties(&c_dst_o).unwrap().props;
                                                                    match k {
                                                                        k if k == KEY_NAME => {
                                                                            let mut p = c_dst_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                            p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                            self.db.add_object_properties(&c_dst_o, &p, PropertyType::Base);
                                                                        }
                                                                        k if k == KEY_SCRIPTS => {
                                                                            let mut p = c_dst_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                            p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                            self.db.add_object_properties(&c_dst_o, &p, PropertyType::Base);
                                                                        }
                                                                        k if k == KEY_SCRIPTS_PATH => {
                                                                            let mut p = c_dst_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                                            p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                            self.db.add_object_properties(&c_dst_o, &p, PropertyType::Base);
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
                        let rte_type = rte_module_cfg.get(KEY_TYPE).unwrap().as_str().unwrap();
                        let rte = RteType::new(rte_type, self.db);
                        if let Some(r) = rte { r.init(&r_o.get_object()) }
                    }
                }
                _ => {}
            }
        }

        let ci_o_p = self.db.get_object_properties(&ci.get_object()).unwrap();
        let ci_o_p_base = ci_o_p.props.get(PropertyType::Base.index()).unwrap();
        let _ci_id_path = ci_o_p_base.value.as_object().unwrap().get(KEY_ID_PATH).unwrap().as_array().unwrap();
        let mut ci_id_path: Vec<String> = _ci_id_path.iter().map(|c| c.as_str().unwrap().to_string()).collect();

        //Project Stages Deploy
        let project_stage_deploy = self.add_ci_stages(&mut ci_id_path, &ci.get_object(), &self.config.project.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Dashboard Stages Deploy
        let dashboard_stage_deploy = self.add_ci_stages(&mut ci_id_path, &project_stage_deploy.unwrap(), &self.config.dashboard.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Rte Stages Deploy
        let rte_stage_deploy = self.add_ci_stages(&mut ci_id_path, &dashboard_stage_deploy.unwrap(), &self.config.rte.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Feature Stages Deploy
        let feature_stage_deploy = self.add_ci_stages(&mut ci_id_path, &rte_stage_deploy.unwrap(), &self.config.features.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Eut Stages Deploy
        let eut_stage_deploy = self.add_ci_stages(&mut ci_id_path, &feature_stage_deploy.unwrap(), &self.config.eut.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Application Stages Deploy
        let application_stage_deploy = self.add_ci_stages(&mut ci_id_path, &eut_stage_deploy.unwrap(), &self.config.applications.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Test Stages Deploy
        let test_stage_deploy = self.add_ci_stages(&mut ci_id_path, &application_stage_deploy.unwrap(), &self.config.tests.ci.stages.deploy, &VertexTypes::StageDeploy);

        //Test and Verification sequential job stages
        let _rtes = self.db.get_object_neighbour_out(&&eut.get_id(), EdgeTypes::UsesRtes);
        let rtes = self.db.get_object_neighbours_with_properties_out(&_rtes.id, EdgeTypes::ProvidesRte);
        let mut _test_stages_seq: Vec<String> = Vec::new();
        let mut _verification_stages_seq: Vec<String> = Vec::new();

        for rte in rtes.iter() {
            let _c = self.db.get_object_neighbour_out(&rte.vertex.id, EdgeTypes::HasConnections);
            let _conns = self.db.get_object_neighbours_out(&_c.id, EdgeTypes::HasConnection);
            for conn in _conns.iter() {
                let c_src = self.db.get_object_neighbour_with_properties_out(&conn.id, EdgeTypes::HasConnectionSrc).unwrap();
                let tests = self.db.get_object_neighbours_with_properties_out(&c_src.vertex.id, EdgeTypes::Runs);
                for t in tests.iter() {

                    let t_stage_name = format!("{}-{}-{}-{}-{}-{}",
                                               KEY_TEST,
                                               rte.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                               c_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                               &t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap(),
                                               &t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                               KEY_DEPLOY
                    ).replace('_', "-");
                    _test_stages_seq.push(t_stage_name);

                    //Verification stages
                    let verifications = self.db.get_object_neighbours_with_properties_out(&t.vertex.id, EdgeTypes::Needs);

                    for v in verifications.iter() {
                        let v_stage_name = format!("{}-{}-{}-{}-{}-{}-{}",
                                                   KEY_VERIFICATION,
                                                   rte.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                                   c_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                                   &t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                                   &v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap(),
                                                   &v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                                   KEY_DEPLOY
                        ).replace('_', "-");
                        _verification_stages_seq.push(v_stage_name);
                    }
                }
            }
        }
        let test_stage_deploy_seq = self.add_ci_stages(&mut ci_id_path, &test_stage_deploy.unwrap(), &_test_stages_seq, &VertexTypes::StageDeploy);

        //Verification Stages Deploy
        let verification_stage_deploy = self.add_ci_stages(&mut ci_id_path, &test_stage_deploy_seq.unwrap(), &self.config.verifications.ci.stages.deploy, &VertexTypes::StageDeploy);
        self.add_ci_stages(&mut ci_id_path, &verification_stage_deploy.unwrap(), &_verification_stages_seq, &VertexTypes::StageDeploy);

        //Feature Stages Destroy
        let mut stage_destroy: Option<Vertex> = None;
        let _features = self.db.get_object_neighbour_out(&eut.get_id(), EdgeTypes::HasFeatures);
        let features = self.db.get_object_neighbours_out(&_features.id, EdgeTypes::HasFeature);

        if !features.is_empty() {
            stage_destroy = self.add_ci_stages(&mut ci_id_path, &ci.get_object(), &self.config.features.ci.stages.destroy, &VertexTypes::StageDestroy);
        }

        ci_id_path.truncate(2);

        //Eut Stages Destroy
        match stage_destroy {
            Some(f) => stage_destroy = self.add_ci_stages(&mut ci_id_path, &f, &self.config.eut.ci.stages.destroy, &VertexTypes::StageDestroy),
            None => stage_destroy = self.add_ci_stages(&mut ci_id_path, &ci.get_object(), &self.config.eut.ci.stages.destroy, &VertexTypes::StageDestroy)
        }

        //Application Stages Destroy
        let _applications = self.db.get_object_neighbour_out(&&eut.get_id(), EdgeTypes::HasApplications);
        let applications = self.db.get_object_neighbours_out(&_applications.id, EdgeTypes::ProvidesApplication);

        if !applications.is_empty() {
            stage_destroy = self.add_ci_stages(&mut ci_id_path, &stage_destroy.unwrap(), &self.config.applications.ci.stages.destroy, &VertexTypes::StageDestroy);
        }

        //Rte Stages Destroy
        match stage_destroy {
            Some(a) => stage_destroy = self.add_ci_stages(&mut ci_id_path, &a, &self.config.rte.ci.stages.destroy, &VertexTypes::StageDestroy),
            None => stage_destroy = self.add_ci_stages(&mut ci_id_path, &eut.get_object(), &self.config.rte.ci.stages.destroy, &VertexTypes::StageDestroy)
        }

        //Dashboard Stages Destroy
        stage_destroy = self.add_ci_stages(&mut ci_id_path, &stage_destroy.unwrap(), &self.config.dashboard.ci.stages.destroy, &VertexTypes::StageDestroy);

        //Project Stages Destroy
        self.add_ci_stages(&mut ci_id_path, &stage_destroy.unwrap(), &self.config.project.ci.stages.destroy, &VertexTypes::StageDestroy);

        project.get_id().clone()
    }

    fn load_regression_config(path: &str, file: &str) -> RegressionConfig {
        info!("Loading regression configuration data...");
        let data: String = format!("{path}/{CONFIG_FILE_PATH}/{file}");
        error!("Sense8 config file: {}", &data);
        let raw = std::fs::read_to_string(data).unwrap();
        let _tmp: Value = serde_json::from_str(&raw).unwrap();
        let mut _cfg = _tmp.as_object().unwrap().clone();
        _cfg.insert("root_path".to_string(), Value::from(path.to_string()));
        let cfg = serde_json::from_value::<RegressionConfig>(to_value(&_cfg).unwrap()).unwrap();
        info!("Loading regression configuration data -> Done.");

        info!("Render regression configuration file...");
        let mut _tera = Tera::new(&*format!("{path}/{CONFIG_FILE_PATH}/*")).unwrap();
        let mut context = Context::new();
        context.insert(KEY_EUT, &cfg.eut);
        context.insert(KEY_RTE, &cfg.rte);
        context.insert(KEY_TESTS, &cfg.tests);
        context.insert(KEY_PROJECT, &cfg.project);
        context.insert(KEY_FEATURES, &cfg.features);
        context.insert(KEY_DASHBOARD, &cfg.dashboard);
        //context.insert("collector", &cfg.collector);
        context.insert(KEY_APPLICATIONS, &cfg.applications);
        context.insert(KEY_VERIFICATIONS, &cfg.verifications);

        let eutc = _tera.render(file, &context).unwrap();
        info!("Render regression configuration file -> Done.");

        info!("Loading regression configuration data...");
        let _tmp: Value = serde_json::from_str(&eutc).unwrap();
        let mut _cfg = _tmp.as_object().unwrap().clone();
        _cfg.insert("root_path".to_string(), Value::from(path.to_string()));
        let cfg = serde_json::from_value::<RegressionConfig>(to_value(&_cfg).unwrap()).unwrap();
        info!("Loading regression configuration data -> Done.");

        cfg
    }

    fn add_ci_stages(&self, id_path: &mut Vec<String>, ancestor: &Vertex, stages: &[String], object_type: &VertexTypes) -> Option<Vertex> {
        let mut curr = Vertex { id: Default::default(), t: Default::default() };

        for (i, stage) in stages.iter().enumerate() {
            let (new, _id_path) = self.db.create_object_and_init(object_type.clone(), id_path, stage, 0);
            self.db.add_object_properties(&new, &json!({KEY_NAME: stage}), PropertyType::Base);

            if i == 0 {
                self.db.create_relationship(ancestor, &new);
                curr = new.clone();
            } else {
                self.db.create_relationship(&curr, &new);
                curr = new.clone();
            }
        }
        Some(curr)
    }

    pub fn build_context(&self, id: Uuid) -> Context {
        info!("Build render context...");
        let mut actions: ActionsRenderContext = ActionsRenderContext {
            rtes: vec![],
            sites: vec![],
            tests: vec![],
            features: vec![],
            applications: vec![],
            verifications: vec![],
        };

        //Project
        let project = Project::load(&self.db, &id, &self.config);
        let project_p_base = project.get_base_properties();
        let project_module = project_p_base.get(KEY_MODULE).unwrap().as_str().unwrap();
        let scripts = project.gen_script_render_ctx(&self.config);
        let project_rc = project.gen_render_ctx(&self.config, scripts.clone());

        //Dashboard
        let dashboard = Dashboard::load(self.db, &project.get_object(), &self.config);
        let scripts = dashboard.gen_script_render_ctx(&self.config);
        let dashboard_rc = dashboard.gen_render_ctx(&self.config, scripts.clone());

        // Eut
        let eut = Eut::load(&self.db, &project, &self.config);
        let eut_p_base = eut.get_base_properties();
        let eut_p_module = eut.get_module_properties();
        let eut_name = eut_p_module.get(KEY_NAME).unwrap().as_str().unwrap().to_string();

        //Process eut provider
        let _eut_providers = self.db.get_object_neighbour_out(&eut.get_id(), EdgeTypes::HasProviders);
        let eut_provider = self.db.get_object_neighbours_with_properties_out(&_eut_providers.id, EdgeTypes::ProvidesProvider);

        let mut eut_provider_p_base = Vec::new();
        for p in eut_provider.iter() {
            let name = p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            eut_provider_p_base.push(String::from(name));
        }

        //Process features
        let features_rc: Vec<Box<dyn RenderContext>> = Features::gen_render_ctx(self.db, &eut.get_object(), &self.config);

        //Process applications
        let applications_rc: Vec<Box<dyn RenderContext>> = Applications::gen_render_ctx(self.db, &eut.get_object(), &self.config);

        //Get EUT sites
        let _sites = self.db.get_object_neighbour_out(&eut.get_id(), EdgeTypes::HasSites);
        let sites = self.db.get_object_neighbours_with_properties_out(&_sites.id, EdgeTypes::HasSite);

        //Get EUT rtes
        let _rtes = self.db.get_object_neighbour_out(&eut.get_id(), EdgeTypes::UsesRtes);
        let rtes = self.db.get_object_neighbours_with_properties_out(&_rtes.id, EdgeTypes::ProvidesRte);

        //Process rte share data script render context
        let site_count: usize = 0;
        let mut srsd: HashMap<String, ScriptRteSitesShareDataRenderContext> = HashMap::new();

        for rte in rtes.iter() {
            let rte_type = rte.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap();
            let _rte = RteType::new(rte_type, self.db);
            if let Some(r) = _rte { r.build_ctx(rte, site_count, &mut srsd) }
        }

        //Build rte_to_site_map structure and add ScriptRteSiteShareDataRenderContext context
        let mut srsd_rc: Vec<ScriptRteSiteShareDataRenderContext> = Vec::new();
        let mut rte_to_site_map: HashMap<String, HashSet<String>> = HashMap::new();
        for (rte, data) in srsd.iter() {
            let mut sites: HashSet<String> = HashSet::new();

            for (site, data) in data.sites.iter() {
                sites.insert(site.to_string());
                srsd_rc.push(data.clone());
            }

            rte_to_site_map.entry(rte.to_string()).or_insert(sites);
        }

        //Process eut rtes
        let mut rtes_rc: Vec<RteRenderContext> = Vec::new();
        let mut rte_names: Vec<String> = Vec::new();

        for rte in rtes.iter() {
            let rte_name = rte.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
            rte_names.push(rte_name.to_string());

            let mut rte_crcs = RteRenderContext {
                ci: HashMap::new(),
                name: rte_name.to_string(),
                tests: vec![],
                shares: vec![],
                components: Default::default(),
            };

            let _provider = self.db.get_object_neighbour_out(&rte.vertex.id, EdgeTypes::NeedsProvider);
            let provider = self.db.get_object_neighbours_with_properties_out(&_provider.id, EdgeTypes::ProvidesProvider);
            let _active_provider = rte.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_PROVIDER).unwrap().as_array().unwrap();
            let mut active_provider: Vec<&VertexProperties> = Vec::new();

            for p in provider.iter() {
                let p_name = p.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                if _active_provider.contains(&to_value(p_name).unwrap()) {
                    active_provider.push(p);
                    let ci_p = self.db.get_object_neighbour_with_properties_out(&p.vertex.id, EdgeTypes::HasCi).unwrap();
                    rte_crcs.ci.insert(p_name.to_string(),
                                       RteCiRenderContext {
                                           timeout: ci_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("timeout").unwrap().clone(),
                                           variables: ci_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("variables").unwrap().clone(),
                                           artifacts: ci_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get("artifacts").unwrap().clone(),
                                       },
                    );

                    //Process provider share scripts render context
                    if let Some(share_p) = self.db.get_object_neighbour_with_properties_out(&p.vertex.id, EdgeTypes::NeedsShare) {
                        let scripts_path = share_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                        let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();

                        for script in share_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                            let path = format!("{}/{}/{}/{}/{}/{}/{}", self.config.root_path, self.config.rte.path, rte_name, scripts_path, p_name, KEY_SHARE, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                            let contents = std::fs::read_to_string(path).expect("panic while opening feature script file");
                            let ctx = ScriptRteProviderShareRenderContext {
                                rte: rte_name.to_string(),
                                eut: eut_name.to_string(),
                                map: serde_json::to_string(&rte_to_site_map).unwrap(),
                                sites: serde_json::to_string(&srsd_rc).unwrap(),
                                counter: site_count,
                                provider: p_name.to_string(),
                                project: self.config.project.clone(),
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

                        rte_crcs.shares.push(RteProviderShareRenderContext {
                            job: format!("{}_{}_{}_{}_{}", project_module, KEY_RTE, &rte_name, p_name, KEY_SHARE).replace('_', "-"),
                            rte: rte_name.to_string(),
                            eut: eut_name.to_string(),
                            provider: p_name.to_string(),
                            scripts,
                        });
                    }
                }
            }

            //Process connections
            let rte_type = rte.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap();
            let _rte = RteType::new(rte_type, self.db);
            let mut feature_names: Vec<String> = Vec::new();

            for _feature in &features_rc {
                let feature: &FeatureRenderContext = match _feature.as_any().downcast_ref::<FeatureRenderContext>() {
                    Some(f) => f,
                    None => panic!("&f isn't a FeatureRenderContext!"),
                };
                feature_names.push(feature.module.get(KEY_NAME).unwrap().to_string());
            }

            if let Some(r) = _rte {
                r.build_conn_ctx(RteCtxParameters {
                    rte,
                    config: &self.config,
                    project: self.config.project.clone(),
                    eut: &eut.get_object_with_properties(),
                    rte_name: rte_name.to_string(),
                    features: feature_names,
                    provider: active_provider,
                    rte_crcs: &mut rte_crcs,
                })
            }

            for component in rte_crcs.components.iter() {
                actions.rtes.push(component.job.clone());
            }

            for test in rte_crcs.tests.iter() {
                actions.tests.push(test.job.clone());
                for verification in test.verifications.iter() {
                    actions.verifications.push(verification.job.clone());
                }
            }

            rtes_rc.push(rte_crcs);
        }

        //Process eut sites
        let mut eut_sites: Vec<EutSiteRenderContext> = vec![];
        for (i, s) in sites.iter().enumerate() {
            let site_name = s.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            let s_p = self.db.get_object_neighbour_with_properties_out(&s.vertex.id, EdgeTypes::UsesProvider).unwrap();
            let provider_name = s_p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            let r_p = self.db.get_object_neighbour_with_properties_out(&s.vertex.id, EdgeTypes::SiteRefersRte);
            let mut rte_name: String = Default::default();

            if let Some(v) = r_p {
                rte_name = v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
            }

            //Process eut site scripts
            let scripts_path = eut_p_module.get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
            let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
            for script in eut_p_module.get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                let path = format!("{}/{}/{}/{}/{}", self.config.root_path, self.config.eut.path, eut_name, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                let contents = std::fs::read_to_string(path).expect("panic while opening eut site script file");
                let ctx = ScriptEutRenderContext {
                    project: self.config.project.clone(),
                    rte: rte_name.to_string(),
                    rtes: rte_names.clone(),
                    name: eut_name.to_string(),
                    site: site_name.to_string(),
                    index: i,
                    counter: sites.len(),
                    release: eut_p_module.get(KEY_RELEASE).unwrap().as_str().unwrap().to_string(),
                    provider: provider_name.to_string(),
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
            let eut_s_rc = EutSiteRenderContext {
                job: format!("{}_{}_{}_{}", project_module, KEY_EUT, &eut_name, &site_name).replace('_', "-"),
                name: site_name.to_string(),
                index: i,
                scripts,
                provider: provider_name.to_string(),
            };
            actions.sites.push(eut_s_rc.job.clone());
            eut_sites.push(eut_s_rc);
        }

        let eut_rc = EutRenderContext {
            base: eut_p_base.clone(),
            module: eut_p_module.clone(),
            provider: eut_provider_p_base.clone(),
            project: self.config.project.clone(),
            sites: eut_sites,
        };

        let mut stages: Vec<String> = Vec::new();
        let mut deploy_stages: Vec<String> = Vec::new();
        let mut destroy_stages: Vec<String> = Vec::new();

        let project_ci = self.db.get_object_neighbour_out(&project.get_id(), EdgeTypes::HasCi);
        let s_deploy = self.db.get_object_neighbour_with_properties_out(&project_ci.id, EdgeTypes::HasDeployStages).unwrap();
        let s_destroy = self.db.get_object_neighbour_with_properties_out(&project_ci.id, EdgeTypes::HasDestroyStages).unwrap();
        deploy_stages.push(s_deploy.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string());
        self.get_next_stage(&s_deploy.vertex.id, &mut deploy_stages);
        deploy_stages.push(s_destroy.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string());
        self.get_next_stage(&s_destroy.vertex.id, &mut destroy_stages);

        stages.append(&mut deploy_stages);
        stages.append(&mut destroy_stages);

        let mut context = Context::new();
        context.insert(KEY_EUT, &eut_rc);
        context.insert(KEY_RTES, &rtes_rc);
        context.insert(KEY_CONFIG, &self.config);
        context.insert(KEY_STAGES, &stages);
        context.insert(KEY_ACTIONS, &actions);
        context.insert(KEY_PROJECT, &project_rc);
        context.insert(KEY_FEATURES, &features_rc);
        context.insert(KEY_DASHBOARD, &dashboard_rc);
        context.insert(KEY_APPLICATIONS, &applications_rc);

        //error!("{:#?}", context.get(KEY_STAGES));
        info!("Build render context -> Done.");
        context
    }

    fn get_next_stage(&self, id: &Uuid, data: &mut Vec<String>) {
        for stage in self.db.get_object_neighbours_with_properties_out(id, EdgeTypes::NextStage).iter() {
            data.push(stage.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string());
            self.get_next_stage(&stage.vertex.id, data);
        }
    }

    pub fn render(&self, context: &Context) -> String {
        info!("Render regression pipeline file first step...");
        let mut _tera = Tera::new(&self.config.project.templates).unwrap();
        let rendered = _tera.render(PIPELINE_TEMPLATE_FILE_NAME, context).unwrap();
        info!("Render regression pipeline file first step -> Done.");
        rendered
    }

    #[allow(dead_code)]
    pub fn to_json(&self) -> String {
        let j = json!({KEY_CONFIG: &self.config,});
        j.to_string()
    }

    pub fn to_file(&self, data: &str, path: &str, file: &str) {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!("{path}/{file}"))
            .expect("Couldn't open file");

        f.write_all(data.as_bytes()).expect("panic while writing to file");
    }

    pub fn to_gv(&self) -> String {
        let mut context = Context::new();
        let _nodes = self.db.get_all_objects();

        match &_nodes {
            Some(nodes) => {
                let mut items = Vec::new();

                for n in nodes.iter() {
                    let n_p = self.db.get_object_properties(&self.db.get_object(&n.id));
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
        let _edges = self.db.get_all_edges();

        match &_edges {
            Some(edge) => {
                let mut items = Vec::new();

                for e in edge.iter() {
                    let o_a = self.db.get_object(&e.outbound_id);
                    let o_b = self.db.get_object(&e.inbound_id);
                    let a_id = self.db.get_object(&e.outbound_id).t.to_string();
                    let b_id = self.db.get_object(&e.inbound_id).t.to_string();
                    let a_p = self.db.get_object_properties(&self.db.get_object(&o_a.id));
                    let b_p = self.db.get_object_properties(&self.db.get_object(&o_b.id));

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
        _tera.render("graph.tpl", &context).unwrap()
    }

    pub fn render_entry_page(&self, context: &Context) -> Result<String, Box<dyn Error>> {
        error!("Render entry page..");
        let mut _tera = Tera::new(&self.config.project.templates).unwrap();
        Ok(_tera.render("entry.tpl", context).unwrap())
    }

    pub fn render_actions_json_file(&self, context: &Context) -> Result<String, Box<dyn Error>> {
        error!("Render actions json file..");
        let mut _tera = Tera::new(&self.config.project.templates).unwrap();
        Ok(_tera.render("actions.tpl", context).unwrap())
    }
}