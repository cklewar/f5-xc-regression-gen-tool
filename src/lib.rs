use std::any::Any;
use std::cmp::Ordering;
use std::collections::{HashMap};
use std::error::Error;
use std::fmt::{Debug};
use std::format;
use std::io::{Write};

use indradb::{Vertex, VertexProperties};
use lazy_static::lazy_static;
use log::{error, info};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Map, to_value, Value};
use serde_json::Value::Null;
use tera::{Context, Tera};
use uuid::Uuid;

use objects::{Ci, Eut, Features, Feature, Project, Providers, EutProvider, Rtes, Rte, Sites, Site,
              Dashboard, Application, Applications, Collectors, Collector, Reports, Report,
              Connections};

use crate::constants::*;
use crate::db::Db;
use crate::objects::{ConnectionSource, Test};

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
    Report,
    Reports,
    Feature,
    Features,
    Providers,
    Collector,
    Dashboard,
    Collectors,
    Components,
    Connection,
    Connections,
    Application,
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
    ApplicationProvider,
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
    RefersRte,
    RefersEut,
    RefersSite,
    RefersTest,
    NeedsShare,
    HasReports,
    HasFeature,
    HasFeatures,
    ProvidesRte,
    UsesProvider,
    HasProviders,
    NeedsProvider,
    HasCollectors,
    HasComponents,
    HasConnection,
    SiteRefersRte,
    RefersFeature,
    HasConnections,
    ProvidesReport,
    HasDeployStages,
    HasApplications,
    HasComponentSrc,
    HasComponentDst,
    ProvidesProvider,
    HasConnectionSrc,
    HasConnectionDst,
    HasDestroyStages,
    FeatureRefersSite,
    ProvidesCollector,
    ProvidesApplication,
    TestRefersCollector,
    TestRefersApplication,
    ReportRefersCollector,
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
            VertexTypes::Report => VERTEX_TYPE_REPORT,
            VertexTypes::Reports => VERTEX_TYPE_REPORTS,
            VertexTypes::Project => VERTEX_TYPE_PROJECT,
            VertexTypes::Feature => VERTEX_TYPE_FEATURE,
            VertexTypes::Features => VERTEX_TYPE_FEATURES,
            VertexTypes::Dashboard => VERTEX_TYPE_DASHBOARD,
            VertexTypes::Collector => VERTEX_TYPE_COLLECTOR,
            VertexTypes::Providers => VERTEX_TYPE_PROVIDERS,
            VertexTypes::Connection => VERTEX_TYPE_CONNECTION,
            VertexTypes::Components => VERTEX_TYPE_COMPONENTS,
            VertexTypes::Collectors => VERTEX_TYPE_COLLECTORS,
            VertexTypes::Connections => VERTEX_TYPE_CONNECTIONS,
            VertexTypes::Application => VERTEX_TYPE_APPLICATION,
            VertexTypes::Applications => VERTEX_TYPE_APPLICATIONS,
            VertexTypes::EutProvider => VERTEX_TYPE_EUT_PROVIDER,
            VertexTypes::StageDeploy => VERTEX_TYPE_STAGE_DEPLOY,
            VertexTypes::StageDestroy => VERTEX_TYPE_STAGE_DESTROY,
            VertexTypes::Verification => VERTEX_TYPE_VERIFICATION,
            VertexTypes::ComponentSrc => VERTEX_TYPE_COMPONENT_SRC,
            VertexTypes::ComponentDst => VERTEX_TYPE_COMPONENT_DST,
            VertexTypes::ConnectionSrc => VERTEX_TYPE_CONNECTION_SRC,
            VertexTypes::ConnectionDst => VERTEX_TYPE_CONNECTION_DST,
            VertexTypes::DashboardProvider => VERTEX_TYPE_DASHBOARD_PROVIDER,
            VertexTypes::ApplicationProvider => VERTEX_TYPE_APPLICATION_PROVIDER,
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
            VERTEX_TYPE_REPORT => VertexTypes::Report.name(),
            VERTEX_TYPE_REPORTS => VertexTypes::Reports.name(),
            VERTEX_TYPE_PROJECT => VertexTypes::Project.name(),
            VERTEX_TYPE_FEATURE => VertexTypes::Feature.name(),
            VERTEX_TYPE_PROVIDERS => VertexTypes::Providers.name(),
            VERTEX_TYPE_DASHBOARD => VertexTypes::Dashboard.name(),
            VERTEX_TYPE_COLLECTOR => VertexTypes::Collector.name(),
            VERTEX_TYPE_CONNECTION => VertexTypes::Connection.name(),
            VERTEX_TYPE_COMPONENTS => VertexTypes::Components.name(),
            VERTEX_TYPE_COLLECTORS => VertexTypes::Collectors.name(),
            VERTEX_TYPE_CONNECTIONS => VertexTypes::Connections.name(),
            VERTEX_TYPE_APPLICATION => VertexTypes::Application.name(),
            VERTEX_TYPE_APPLICATIONS => VertexTypes::Applications.name(),
            VERTEX_TYPE_VERIFICATION => VertexTypes::Verification.name(),
            VERTEX_TYPE_STAGE_DEPLOY => VertexTypes::StageDeploy.name(),
            VERTEX_TYPE_EUT_PROVIDER => VertexTypes::EutProvider.name(),
            VERTEX_TYPE_STAGE_DESTROY => VertexTypes::StageDestroy.name(),
            VERTEX_TYPE_COMPONENT_SRC => VertexTypes::ComponentSrc.name(),
            VERTEX_TYPE_COMPONENT_DST => VertexTypes::ComponentDst.name(),
            VERTEX_TYPE_CONNECTION_SRC => VertexTypes::ConnectionSrc.name(),
            VERTEX_TYPE_CONNECTION_DST => VertexTypes::ConnectionDst.name(),
            VERTEX_TYPE_DASHBOARD_PROVIDER => VertexTypes::DashboardProvider.name(),
            VERTEX_TYPE_APPLICATION_PROVIDER => VertexTypes::ApplicationProvider.name(),
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
            VERTEX_TYPE_REPORT => VertexTypes::Report,
            VERTEX_TYPE_REPORTS => VertexTypes::Reports,
            VERTEX_TYPE_PROJECT => VertexTypes::Project,
            VERTEX_TYPE_FEATURE => VertexTypes::Feature,
            VERTEX_TYPE_FEATURES => VertexTypes::Features,
            VERTEX_TYPE_PROVIDERS => VertexTypes::Providers,
            VERTEX_TYPE_DASHBOARD => VertexTypes::Dashboard,
            VERTEX_TYPE_COLLECTOR => VertexTypes::Collector,
            VERTEX_TYPE_COLLECTORS => VertexTypes::Collectors,
            VERTEX_TYPE_CONNECTION => VertexTypes::Connection,
            VERTEX_TYPE_COMPONENTS => VertexTypes::Components,
            VERTEX_TYPE_CONNECTIONS => VertexTypes::Connections,
            VERTEX_TYPE_APPLICATION => VertexTypes::Application,
            VERTEX_TYPE_APPLICATIONS => VertexTypes::Applications,
            VERTEX_TYPE_VERIFICATION => VertexTypes::Verification,
            VERTEX_TYPE_EUT_PROVIDER => VertexTypes::EutProvider,
            VERTEX_TYPE_STAGE_DEPLOY => VertexTypes::StageDeploy,
            VERTEX_TYPE_STAGE_DESTROY => VertexTypes::StageDestroy,
            VERTEX_TYPE_COMPONENT_SRC => VertexTypes::ComponentSrc,
            VERTEX_TYPE_COMPONENT_DST => VertexTypes::ComponentDst,
            VERTEX_TYPE_CONNECTION_SRC => VertexTypes::ConnectionSrc,
            VERTEX_TYPE_CONNECTION_DST => VertexTypes::ConnectionDst,
            VERTEX_TYPE_DASHBOARD_PROVIDER => VertexTypes::DashboardProvider,
            VERTEX_TYPE_APPLICATION_PROVIDER => VertexTypes::ApplicationProvider,
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
            EdgeTypes::RefersRte => EDGE_TYPE_REFERS_RTE,
            EdgeTypes::RefersEut => EDGE_TYPE_REFERS_EUT,
            EdgeTypes::NextStage => EDGE_TYPE_NEXT_STAGE,
            EdgeTypes::RefersTest => EDGE_TYPE_REFERS_TEST,
            EdgeTypes::RefersSite => EDGE_TYPE_REFERS_SITE,
            EdgeTypes::HasReports => EDGE_TYPE_HAS_REPORTS,
            EdgeTypes::HasFeature => EDGE_TYPE_HAS_FEATURE,
            EdgeTypes::NeedsShare => EDGE_TYPE_NEEDS_SHARE,
            EdgeTypes::HasFeatures => EDGE_TYPE_HAS_FEATURES,
            EdgeTypes::ProvidesRte => EDGE_TYPE_PROVIDES_RTE,
            EdgeTypes::HasProviders => EDGE_TYPE_HAS_PROVIDERS,
            EdgeTypes::UsesProvider => EDGE_TYPE_USES_PROVIDER,
            EdgeTypes::NeedsProvider => EDGE_TYPE_NEEDS_PROVIDER,
            EdgeTypes::HasComponents => EDGE_TYPE_HAS_COMPONENTS,
            EdgeTypes::RefersFeature => EDGE_TYPE_REFERS_FEATURE,
            EdgeTypes::SiteRefersRte => EDGE_TYPE_APPLICATION_REFERS_FEATURE,
            EdgeTypes::HasConnection => EDGE_TYPE_HAS_CONNECTION,
            EdgeTypes::HasCollectors => EDGE_TYPE_HAS_COLLECTORS,
            EdgeTypes::HasConnections => EDGE_TYPE_HAS_CONNECTIONS,
            EdgeTypes::ProvidesReport => EDGE_TYPE_PROVIDES_REPORTS,
            EdgeTypes::HasComponentSrc => EDGE_TYPE_HAS_COMPONENT_SRC,
            EdgeTypes::HasComponentDst => EDGE_TYPE_HAS_COMPONENT_DST,
            EdgeTypes::HasApplications => EDGE_TYPE_HAS_APPLICATIONS,
            EdgeTypes::HasDeployStages => EDGE_TYPE_HAS_DEPLOY_STAGES,
            EdgeTypes::HasDestroyStages => EDGE_TYPE_HAS_DESTROY_STAGES,
            EdgeTypes::ProvidesProvider => EDGE_TYPE_PROVIDES_PROVIDER,
            EdgeTypes::HasConnectionSrc => EDGE_TYPE_HAS_CONNECTION_SRC,
            EdgeTypes::HasConnectionDst => EDGE_TYPE_HAS_CONNECTION_DST,
            EdgeTypes::FeatureRefersSite => EDGE_TYPE_FEATURE_REFERS_SITE,
            EdgeTypes::ProvidesCollector => EDGE_TYPE_PROVIDES_COLLECTOR,
            EdgeTypes::ProvidesApplication => EDGE_TYPE_PROVIDES_APPLICATION,
            EdgeTypes::TestRefersCollector => EDGE_TYPE_TEST_REFERS_COLLECTION,
            EdgeTypes::TestRefersApplication => EDGE_TYPE_TEST_REFERS_APPLICATION,
            EdgeTypes::ReportRefersCollector => EDGE_TYPE_REPORT_REFERS_COLLECTION,
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
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::HasCi.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Rtes.name().to_string()), EdgeTypes::UsesRtes.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Sites.name().to_string()), EdgeTypes::HasSites.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Scripts.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Features.name().to_string()), EdgeTypes::HasFeatures.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Providers.name().to_string()), EdgeTypes::HasProviders.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Collectors.name().to_string()), EdgeTypes::HasCollectors.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Applications.name().to_string()), EdgeTypes::HasApplications.name());
        map.insert(VertexTuple(VertexTypes::Eut.name().to_string(), VertexTypes::Reports.name().to_string()), EdgeTypes::HasReports.name());
        map.insert(VertexTuple(VertexTypes::Rtes.name().to_string(), VertexTypes::Rte.name().to_string()), EdgeTypes::ProvidesRte.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::HasCi.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Scripts.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Features.name().to_string()), EdgeTypes::Needs.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Collector.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Connections.name().to_string()), EdgeTypes::HasConnections.name());
        map.insert(VertexTuple(VertexTypes::Rte.name().to_string(), VertexTypes::Components.name().to_string()), EdgeTypes::HasComponents.name());
        map.insert(VertexTuple(VertexTypes::Sites.name().to_string(), VertexTypes::Site.name().to_string()), EdgeTypes::HasSite.name());
        map.insert(VertexTuple(VertexTypes::Site.name().to_string(), VertexTypes::Rte.name().to_string()), EdgeTypes::SiteRefersRte.name());
        map.insert(VertexTuple(VertexTypes::Site.name().to_string(), VertexTypes::EutProvider.name().to_string()), EdgeTypes::UsesProvider.name());
        map.insert(VertexTuple(VertexTypes::Providers.name().to_string(), VertexTypes::EutProvider.name().to_string()), EdgeTypes::ProvidesProvider.name());
        map.insert(VertexTuple(VertexTypes::Providers.name().to_string(), VertexTypes::ApplicationProvider.name().to_string()), EdgeTypes::ProvidesProvider.name());
        map.insert(VertexTuple(VertexTypes::Connections.name().to_string(), VertexTypes::Connection.name().to_string()), EdgeTypes::HasConnection.name());
        map.insert(VertexTuple(VertexTypes::Components.name().to_string(), VertexTypes::ComponentSrc.name().to_string()), EdgeTypes::HasComponentSrc.name());
        map.insert(VertexTuple(VertexTypes::Components.name().to_string(), VertexTypes::ComponentDst.name().to_string()), EdgeTypes::HasComponentDst.name());
        map.insert(VertexTuple(VertexTypes::Connection.name().to_string(), VertexTypes::ConnectionSrc.name().to_string()), EdgeTypes::HasConnectionSrc.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::Test.name().to_string()), EdgeTypes::Runs.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::ConnectionDst.name().to_string()), EdgeTypes::HasConnectionDst.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::ComponentSrc.name().to_string()), EdgeTypes::HasComponentSrc.name());
        map.insert(VertexTuple(VertexTypes::ConnectionSrc.name().to_string(), VertexTypes::Site.name().to_string()), EdgeTypes::RefersSite.name());
        map.insert(VertexTuple(VertexTypes::ConnectionDst.name().to_string(), VertexTypes::Site.name().to_string()), EdgeTypes::RefersSite.name());
        map.insert(VertexTuple(VertexTypes::ConnectionDst.name().to_string(), VertexTypes::ComponentDst.name().to_string()), EdgeTypes::HasComponentDst.name());
        map.insert(VertexTuple(VertexTypes::Test.name().to_string(), VertexTypes::Ci.name().to_string()), EdgeTypes::HasCi.name());
        map.insert(VertexTuple(VertexTypes::Test.name().to_string(), VertexTypes::Collector.name().to_string()), EdgeTypes::TestRefersCollector.name());
        map.insert(VertexTuple(VertexTypes::Test.name().to_string(), VertexTypes::Application.name().to_string()), EdgeTypes::TestRefersApplication.name());
        map.insert(VertexTuple(VertexTypes::Test.name().to_string(), VertexTypes::Verification.name().to_string()), EdgeTypes::Needs.name());
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::HasDeployStages.name());
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::HasDestroyStages.name());
        map.insert(VertexTuple(VertexTypes::StageDeploy.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::NextStage.name());
        map.insert(VertexTuple(VertexTypes::StageDestroy.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::NextStage.name());
        map.insert(VertexTuple(VertexTypes::Applications.name().to_string(), VertexTypes::Application.name().to_string()), EdgeTypes::ProvidesApplication.name());
        map.insert(VertexTuple(VertexTypes::Application.name().to_string(), VertexTypes::Feature.name().to_string()), EdgeTypes::RefersFeature.name());
        map.insert(VertexTuple(VertexTypes::Application.name().to_string(), VertexTypes::Rte.name().to_string()), EdgeTypes::RefersRte.name());
        map.insert(VertexTuple(VertexTypes::Application.name().to_string(), VertexTypes::Site.name().to_string()), EdgeTypes::RefersSite.name());
        map.insert(VertexTuple(VertexTypes::Application.name().to_string(), VertexTypes::Providers.name().to_string()), EdgeTypes::NeedsProvider.name());
        map.insert(VertexTuple(VertexTypes::Features.name().to_string(), VertexTypes::Feature.name().to_string()), EdgeTypes::HasFeature.name());
        map.insert(VertexTuple(VertexTypes::Feature.name().to_string(), VertexTypes::Site.name().to_string()), EdgeTypes::FeatureRefersSite.name());
        map.insert(VertexTuple(VertexTypes::Scripts.name().to_string(), VertexTypes::Script.name().to_string()), EdgeTypes::Has.name());
        map.insert(VertexTuple(VertexTypes::Reports.name().to_string(), VertexTypes::Report.name().to_string()), EdgeTypes::ProvidesReport.name());
        map.insert(VertexTuple(VertexTypes::Report.name().to_string(), VertexTypes::Collector.name().to_string()), EdgeTypes::ReportRefersCollector.name());
        map.insert(VertexTuple(VertexTypes::Collectors.name().to_string(), VertexTypes::Collector.name().to_string()), EdgeTypes::ProvidesCollector.name());
        map.insert(VertexTuple(VertexTypes::Collector.name().to_string(), VertexTypes::Test.name().to_string()), EdgeTypes::RefersTest.name());
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
    artifacts_dir: String,
    artifacts_file: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigEut {
    ci: RegressionConfigGenericCi,
    path: String,
    module: String,
    config: Option<String>,
    artifacts_dir: String,
    artifacts_file: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigCollectors {
    path: String,
    artifacts_dir: String,
    artifacts_file: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigReports {
    ci: RegressionConfigGenericCi,
    path: String,
    data_vars_path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigFeatures {
    ci: RegressionConfigGenericCi,
    path: String,
    artifacts_dir: String,
    artifacts_file: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigRte {
    ci: RegressionConfigGenericCi,
    path: String,
    artifacts_dir: String,
    artifacts_file: String,
    data_vars_path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigTests {
    ci: RegressionConfigGenericCi,
    path: String,
    artifacts_dir: String,
    artifacts_file: String,
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
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RegressionConfig {
    ci: RegressionConfigCi,
    eut: RegressionConfigEut,
    rte: RegressionConfigRte,
    tests: RegressionConfigTests,
    project: RegressionConfigProject,
    reports: RegressionConfigReports,
    features: RegressionConfigFeatures,
    root_path: String,
    dashboard: RegressionConfigDashboard,
    collectors: RegressionConfigCollectors,
    applications: RegressionConfigApplications,
    verifications: RegressionConfigVerifications,
}

#[derive(Serialize, Debug)]
struct ActionsRenderContext {
    rtes: Vec<String>,
    sites: Vec<String>,
    tests: Vec<String>,
    reports: Vec<String>,
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
struct ReportRenderContext {
    job: String,
    base: Map<String, Value>,
    module: Map<String, Value>,
    project: RegressionConfigProject,
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
    refs: Map<String, Value>,
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

#[derive(Serialize, Clone, Debug)]
struct RteCiRenderContext {
    timeout: Value,
    variables: Value,
    artifacts: Value,
}

#[derive(Serialize, Clone, Debug)]
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

#[derive(Serialize, Clone, Debug)]
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

#[derive(Serialize, Clone, Debug)]
struct RteComponentRenderContext {
    job: String,
    rte: String,
    name: String,
    site: String,
    scripts: Vec<HashMap<String, Vec<String>>>,
    provider: String,
}

#[derive(Serialize, Debug, Clone)]
struct RteRenderContext {
    ci: RteCiRenderContext,
    name: String,
    base: Map<String, Value>,
    module: Map<String, Value>,
    tests: Vec<RteTestRenderContext>,
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
    artifacts_path: String,
}

#[derive(Serialize, Debug)]
struct ScriptCollectorRenderContext {
    eut: String,
    name: String,
    data: String,
    refs: Map<String, Value>,
    module: String,
    project: RegressionConfigProject,
    artifacts_path: String,
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
    data: String,
    data_dir: String,
    refs: Map<String, Value>,
    module: String,
    release: String,
    provider: String,
    project: RegressionConfigProject,
    artifacts_path: String,
}

#[derive(Serialize, Debug)]
struct ScriptVerificationRenderContext {
    name: String,
    data: String,
    module: String,
    provider: String,
    collector: String,
    test_name: String,
    test_module: String,
    test_artifacts_path: String,
    rte_name: String,
    rte_module: String,
    rte_artifacts_path: String,
}

#[derive(Serialize, Debug)]
struct ScriptTestRenderContext {
    rte: String,
    eut: String,
    name: String,
    data: String,
    refs: Map<String, Value>,
    module: String,
    project: RegressionConfigProject,
    provider: String,
    artifacts_path: String,
    rte_artifacts_path: String,
}

#[derive(Serialize, Debug)]
struct ScriptRteRenderContext {
    eut: String,
    site: String,
    base: Map<String, Value>,
    module: Map<String, Value>,
    release: String,
    project: RegressionConfigProject,
    provider: String,
    destinations: String,
    artifacts_path: String,
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
struct ScriptReportRenderContext {
    eut: String,
    name: String,
    data: String,
    refs: Map<String, Value>,
    module: String,
    project: RegressionConfigProject,
    collector_name: String,
    collector_module: String,
}

/*fn get_type_of<T>(_: T) {
    println!("TYPE: {}", std::any::type_name::<T>())
}*/

//Struct used to return Vec of Values and obj id. To be used later to create object references
#[derive(Debug)]
pub struct ObjRefs {
    refs: Vec<Value>, // vec of refs for specific type
    id: Uuid, // obj to build rel with
}

fn build_refs_map(refs: &mut HashMap<String, Vec<String>>, r#type: &str, path: &str) {
    if refs.get(&r#type.to_string()).is_none() {
        refs.insert(r#type.to_string(), vec!(path.to_string()));
    } else {
        let items: &mut Vec<String> = refs.get_mut(r#type).unwrap();
        if !items.contains(&path.to_string()) {
            items.push(path.to_string());
        }
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

impl ScriptRenderContext for ScriptReportRenderContext {}

impl ScriptRenderContext for ScriptProjectRenderContext {}

impl ScriptRenderContext for ScriptFeatureRenderContext {}

impl ScriptRenderContext for ScriptCollectorRenderContext {}

impl ScriptRenderContext for ScriptDashboardRenderContext {}

impl ScriptRenderContext for ScriptApplicationRenderContext {}

impl ScriptRenderContext for ScriptVerificationRenderContext {}

//impl ScriptRenderContext for ScriptRteProviderShareRenderContext {}

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
    eut: &'a VertexProperties,
    config: &'a RegressionConfig,
    project_config: RegressionConfigProject,
    project: &'a Vertex,
    rte_name: String,
    rte_crcs: &'a mut RteRenderContext,
    rte_scripts: Vec<HashMap<String, Vec<String>>>,
}

pub struct Regression<'a> {
    db: &'a Db,
    pub config: RegressionConfig,
    pub template: String,
    pub root_path: String,
}

impl<'a> Regression<'a> {
    pub fn new(db: &'a Db, path: &str, file: &str, template: &str, eut_file: &Option<String>) -> Self {
        Regression {
            db,
            config: Regression::load_regression_config(path, file, eut_file.clone()),
            template: String::from(template),
            root_path: path.to_string(),
        }
    }

    pub fn init(&self) -> (Uuid, Vec<ObjRefs>) {
        //Stores object refs statements for later refs creation
        let mut object_refs: Vec<ObjRefs> = Vec::new();

        // Project
        let project = Project::init(self.db, &self.config, &mut vec![], &self.config.project.module, 0);

        // Dashboard
        let dashboard = Dashboard::init(self.db, &self.config,
                                        &mut project.get_id_path().get_vec(), "", 0);
        self.db.create_relationship(&project.get_object(), &dashboard.get_object());

        // Ci
        let ci = Ci::init(self.db, &self.config, &json!(&self.config.ci),
                          &mut project.get_id_path().get_vec(), "", 0);
        self.db.create_relationship(&project.get_object(), &ci.get_object());

        // Eut
        let eut = Eut::init(self.db, &self.config,
                            &mut project.get_id_path().get_vec(), &self.config.eut.module, 0);
        let eut_module_cfg = eut.get_module_cfg();
        self.db.create_relationship(&project.get_object(), &eut.get_object());
        let eut_providers = Providers::init(self.db, &self.config,
                                            &mut eut.get_id_path().get_vec(), "", 0);
        self.db.create_relationship(&eut.get_object(), &eut_providers.get_object());

        for k in EUT_KEY_ORDER.iter() {
            let obj = eut_module_cfg.get(*k).unwrap();
            match *k {
                k if k == KEY_PROVIDER => {
                    for p in obj.as_array().unwrap().iter() {
                        let eut_provider = EutProvider::init(self.db, &self.config,
                                                             &mut eut.get_id_path().get_vec(),
                                                             &p.as_str().unwrap(), 0);
                        self.db.create_relationship(&eut_providers.get_object(), &eut_provider.get_object());
                    }
                }
                k if k == KEY_SITES => {
                    let o = Sites::init(self.db, &self.config,
                                        &mut eut.get_id_path().get_vec(), "", 2);
                    self.db.create_relationship(&eut.get_object(), &o.get_object());
                    let _p = self.db.get_object_neighbour_out(&eut.get_id(), EdgeTypes::HasProviders);
                    let provider = self.
                        db.get_object_neighbours_with_properties_out(&_p.unwrap().id,
                                                                     EdgeTypes::ProvidesProvider);
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
                                let s_o = Site::init(&self.db,
                                                     &self.config,
                                                     site_attr,
                                                     &mut o.get_id_path().get_vec(),
                                                     site_name,
                                                     0);
                                self.db.create_relationship(&o.get_object(), &s_o.get_object());
                                let provider = &site_attr.as_object().unwrap().get(KEY_PROVIDER).unwrap().as_str().unwrap();
                                let p_o = self.db.get_object(id_name_map.get(provider).unwrap());
                                self.db.create_relationship(&s_o.get_object(), &p_o);
                                self.db.add_object_property(&s_o.get_object(), &json!({KEY_NAME: site_name}), PropertyType::Base);
                            }
                            Ordering::Greater => {
                                for c in 1..=site_count {
                                    let s_o = Site::init(&self.db,
                                                         &self.config,
                                                         site_attr,
                                                         &mut o.get_id_path().get_vec(),
                                                         &*format!("{}_{}", site_name, c),
                                                         0);
                                    self.db.create_relationship(&o.get_object(), &s_o.get_object());
                                    let provider = &site_attr.as_object().unwrap().get(KEY_PROVIDER).unwrap().as_str().unwrap();
                                    let p_o = self.db.get_object(id_name_map.get(provider).unwrap());
                                    self.db.create_relationship(&s_o.get_object(), &p_o);
                                    self.db.add_object_property(&s_o.get_object(),
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
                        //let f_sites = f.as_object().unwrap().get(KEY_SITES).unwrap().as_array().unwrap();
                        //let _sites = self.db.get_object_neighbour_out(&&eut.get_id(), EdgeTypes::HasSites);
                        //let sites = self.db.get_object_neighbours_with_properties_out(&_sites.unwrap().id, EdgeTypes::HasSite);
                        let f_o = Feature::init(self.db, &self.config, f,
                                                &mut o.get_id_path().get_vec(), &o.get_object(),
                                                f_module, 0);

                        //Feature -> Site
                        /*for f_site in f_sites {
                            let re = Regex::new(f_site.as_str().unwrap()).unwrap();
                            for site in sites.iter() {
                                let site_name = site.props.get(PropertyType::Base.index()).unwrap().value.as_object().
                                    unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                                if let Some(_t) = re.captures(site_name) {
                                    self.db.create_relationship(&f_o.get_object(), &site.vertex);
                                }
                            }
                        }*/
                    }
                }
                k if k == KEY_COLLECTORS => {
                    let o = Collectors::init(&self.db, &self.config,
                                             &mut eut.get_id_path().get_vec(), "", 2);
                    self.db.create_relationship(&eut.get_object(), &o.get_object());
                    for c in obj.as_array().unwrap().iter() {
                        let c_o = Collector::init(&self.db, &self.config, c,
                                                  &mut o.get_id_path().get_vec(), "", 0);
                        self.db.create_relationship(&o.get_object(), &c_o.get_object());
                        let props = c_o.get_base_properties();

                        object_refs.push(ObjRefs {
                            refs: props.get(KEY_REFS).unwrap().as_array().unwrap().clone(),
                            id: c_o.get_id(),
                        })
                    }
                }
                k if k == KEY_REPORTS => {
                    let o = Reports::init(&self.db, &self.config,
                                          &mut eut.get_id_path().get_vec(), "", 2);
                    self.db.create_relationship(&eut.get_object(), &o.get_object());

                    for r in obj.as_array().unwrap().iter() {
                        let label = r.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                        let report = Report::init(&self.db, &self.config, r,
                                                  &mut o.get_id_path().get_vec(), label, 0);
                        self.db.create_relationship(&o.get_object(), &report.get_object());
                        let props = report.get_base_properties();

                        object_refs.push(ObjRefs {
                            refs: props.get(KEY_REFS).unwrap().as_array().unwrap().clone(),
                            id: report.get_id(),
                        })
                    }
                }
                k if k == KEY_APPLICATIONS => {
                    let o = Applications::init(self.db, &self.config,
                                               &mut eut.get_id_path().get_vec(), "", 2);
                    self.db.create_relationship(&eut.get_object(), &o.get_object());

                    for a in obj.as_array().unwrap().iter() {
                        let a_module = a.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                        let a_o = Application::init(self.db, &self.config, a,
                                                    &mut o.get_id_path().get_vec(), &o.get_object(),
                                                    a_module, 0);
                        let props = a_o.get_base_properties();

                        object_refs.push(ObjRefs {
                            refs: props.get(KEY_REFS).unwrap().as_array().unwrap().clone(),
                            id: a_o.get_id(),
                        })
                    }
                }
                k if k == KEY_RTES => {
                    let o_rtes = Rtes::init(&self.db, &self.config,
                                            &mut eut.get_id_path().get_vec(), "", 1);
                    self.db.create_relationship(&eut.get_object(), &o_rtes.get_object());
                    for rte in obj.as_array().unwrap().iter() {
                        Rte::init(&self.db, &self.config, rte, &mut o_rtes.get_id_path().get_vec(),
                                  &o_rtes, &mut object_refs,
                                  &rte.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap(),
                                  0);
                    }
                }
                _ => {}
            }
        }

        (project.get_id(), object_refs)
    }

    //Create refs from refs config stmts
    pub fn init_refs(&self, id: Uuid, obj_refs: &Vec<ObjRefs>) {
        let project = Project::load(&self.db, &id, &self.config);
        let eut = Eut::load(&self.db, &project, &self.config);

        for obj in obj_refs {
            for r in &obj.refs {
                let v_type = VertexTypes::get_type_by_key(r.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap());
                let ref_name = r.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                match v_type {
                    //Build rel obj --> Application
                    VertexTypes::Application => {
                        error!("Building rel between obj and application");
                        let applications = Applications::load_collection(&self.db, &eut.get_object(), &self.config);
                        let application = Applications::load_application(&self.db, &applications.get_object(), ref_name, &self.config);

                        match application {
                            Some(a) => {
                                self.db.create_relationship(&self.db.get_object(&obj.id), &a.get_object());
                            }
                            None => error!("application object not found")
                        }
                    }
                    //Build rel obj --> Collector
                    VertexTypes::Collector => {
                        error!("Building rel between obj and collector");
                        let collectors = Collectors::load_collection(&self.db, &eut.get_object(), &self.config);
                        let collector = Collectors::load_collector(&self.db, &collectors.get_object(), ref_name, &self.config);

                        match collector {
                            Some(c) => {
                                self.db.create_relationship(&self.db.get_object(&obj.id), &c.get_object());
                            }
                            None => error!("collector object not found")
                        }
                    }
                    //Build rel obj --> Eut site
                    VertexTypes::Site => {
                        error!("Building rel between obj and eut site");
                        let sites = Sites::load_collection(&self.db, &eut.get_object(), &self.config);
                        let site = Sites::load_site(&self.db, &sites.get_object(), ref_name, &self.config);

                        match site {
                            Some(a) => {
                                self.db.create_relationship(&self.db.get_object(&obj.id), &a.get_object());
                            }
                            None => error!("site object not found")
                        }
                    }
                    //Build rel obj --> Feature
                    VertexTypes::Feature => {
                        error!("Building rel between obj and feature");
                        let features = Features::load_collection(&self.db, &eut.get_object(), &self.config);
                        let feature = Features::load_feature(&self.db, &features.get_object(), ref_name, &self.config);

                        match feature {
                            Some(a) => {
                                self.db.create_relationship(&self.db.get_object(&obj.id), &a.get_object());
                            }
                            None => error!("feature object not found")
                        }
                    }
                    //Build rel obj --> Rte
                    VertexTypes::Rte => {
                        error!("Building rel between obj and rte");
                        let rtes = Rtes::load_collection(&self.db, &eut.get_object(), &self.config);
                        let rte = Rtes::load_rte(&self.db, &rtes.get_object(), ref_name, &self.config);

                        match rte {
                            Some(r) => {
                                self.db.create_relationship(&self.db.get_object(&obj.id), &r.get_object());
                            }
                            None => error!("rte object not found")
                        }
                    }
                    //Build rel obj --> Test
                    VertexTypes::Test => {
                        match VertexTypes::get_name_by_object(&self.db.get_object(&obj.id)) {
                            VERTEX_TYPE_COLLECTOR => {
                                error!("Building rel between collector and test");
                                let test_name = r.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                                let rte_name = r.as_object().unwrap().get(KEY_RTE).unwrap().as_str().unwrap();
                                let connection = r.as_object().unwrap().get(KEY_CONNECTION).unwrap().as_str().unwrap();
                                let rtes = Rtes::load_collection(&self.db, &eut.get_object(), &self.config);
                                let rte = Rtes::load_rte(&self.db, &rtes.get_object(), rte_name, &self.config);
                                let connections = Connections::load_collection(&self.db, &rte.unwrap().get_object(), &self.config);
                                let connection = Connections::load_connection(&self.db, &connections.get_object(), connection, &self.config);
                                let c_src = ConnectionSource::load(&self.db, &connection.unwrap().get_object(), &self.config);
                                let test = ConnectionSource::load_test(&self.db, &c_src.get_object(), &test_name, &self.config);

                                match test {
                                    Some(t) => {
                                        self.db.create_relationship(&self.db.get_object(&obj.id), &t.get_object());
                                    }
                                    None => error!("test object not found")
                                }
                            }
                            _ => {
                                error!("No identifier found for: {:?}", VertexTypes::get_name_by_object(&self.db.get_object(&obj.id)));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn init_artifacts(&self, id: Uuid, obj_refs: &Vec<ObjRefs>) {
        let project = Project::load(&self.db, &id, &self.config);
        let eut = Eut::load(&self.db, &project, &self.config);

        for obj in obj_refs {
            let mut refs: HashMap<String, Vec<String>> = Default::default();

            for r in &obj.refs {
                let v_type = VertexTypes::get_type_by_key(r.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap());
                let ref_name = r.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                match v_type {
                    VertexTypes::Application => {
                        error!("Init additional properties for application object");
                        let applications = Applications::load_collection(&self.db, &eut.get_object(), &self.config);
                        let application = Applications::load_application(&self.db, &applications.get_object(), ref_name, &self.config);

                        match application {
                            Some(a) => {
                                build_refs_map(&mut refs,
                                               r.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap(),
                                               &a.get_base_properties().
                                                   get(KEY_ARTIFACTS_PATH).
                                                   unwrap().as_str().
                                                   unwrap().to_string());
                            }
                            None => error!("application object not found")
                        }
                    }
                    VertexTypes::Collector => {
                        error!("Init additional properties for collector object");
                        let collectors = Collectors::load_collection(&self.db, &eut.get_object(), &self.config);
                        let collector = Collectors::load_collector(&self.db, &collectors.get_object(), ref_name, &self.config);

                        match collector {
                            Some(a) => {
                                build_refs_map(&mut refs,
                                               r.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap(),
                                               &a.get_base_properties().get(KEY_ARTIFACTS_PATH).unwrap().as_str().unwrap().to_string());
                            }
                            None => error!("collector object not found")
                        }
                    }
                    VertexTypes::Feature => {
                        error!("Init additional properties for feature object");
                        let features = Features::load_collection(&self.db, &eut.get_object(), &self.config);
                        let feature = Features::load_feature(&self.db, &features.get_object(), ref_name, &self.config);

                        match feature {
                            Some(a) => {
                                build_refs_map(&mut refs,
                                               r.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap(),
                                               &a.get_base_properties().get(KEY_ARTIFACTS_PATH).unwrap().as_str().unwrap().to_string());
                            }
                            None => error!("feature object not found")
                        }
                    }
                    VertexTypes::Rte => {
                        error!("Init additional properties for rte object");
                        let rtes = Rtes::load_collection(&self.db, &eut.get_object(), &self.config);
                        let rte = Rtes::load_rte(&self.db, &rtes.get_object(), ref_name, &self.config);

                        match rte {
                            Some(a) => {
                                build_refs_map(&mut refs,
                                               r.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap(),
                                               &a.get_base_properties().get(KEY_ARTIFACTS_PATH).unwrap().as_str().unwrap().to_string());
                            }
                            None => error!("rte object not found")
                        }
                    }
                    VertexTypes::Site => {
                        error!("Init additional properties for eut site object");
                        let sites = Sites::load_collection(&self.db, &eut.get_object(), &self.config);
                        let site = Sites::load_site(&self.db, &sites.get_object(), ref_name, &self.config);

                        match site {
                            Some(a) => {
                                build_refs_map(&mut refs,
                                               r.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap(),
                                               &a.get_base_properties().get(KEY_ARTIFACTS_PATH).unwrap().as_str().unwrap().to_string());
                            }
                            None => error!("site object not found")
                        }
                    }
                    VertexTypes::Test => {
                        error!("Init additional properties for test object");
                        let rte_name = r.as_object().unwrap().get(KEY_RTE).unwrap().as_str().unwrap();
                        let connection = r.as_object().unwrap().get(KEY_CONNECTION).unwrap().as_str().unwrap();
                        let rtes = Rtes::load_collection(&self.db, &eut.get_object(), &self.config);
                        let rte = Rtes::load_rte(&self.db, &rtes.get_object(), rte_name, &self.config);
                        let connections = Connections::load_collection(&self.db, &rte.unwrap().get_object(), &self.config);
                        let connection = Connections::load_connection(&self.db, &connections.get_object(), connection, &self.config);
                        let c_src = ConnectionSource::load(&self.db, &connection.unwrap().get_object(), &self.config);
                        let test = ConnectionSource::load_test(&self.db, &c_src.get_object(), &ref_name, &self.config);

                        match test {
                            Some(t) => {
                                build_refs_map(&mut refs,
                                               r.as_object().
                                                   unwrap().
                                                   get(KEY_TYPE).
                                                   unwrap().
                                                   as_str().
                                                   unwrap(),
                                               &t.get_base_properties().
                                                   get(KEY_ARTIFACTS_PATH).
                                                   unwrap().as_str().
                                                   unwrap().
                                                   to_string());
                            }
                            None => error!("test object not found")
                        }
                    }
                    _ => {
                        error!("Failed to init additional properties for unknown <{:?}> object", v_type);
                    }
                }
            }

            let o = self.db.get_object(&obj.id);
            let o_p = self.db.get_object_properties(&o).unwrap();

            match VertexTypes::get_name_by_object(&o) {
                KEY_APPLICATION => {
                    error!("Adding artifact refs to application object...");
                    let a = Application::load(&self.db, &o_p, &self.config);
                    a.insert_base_property(KEY_REF_ARTIFACTS_PATH.to_string(), json!(refs));
                }
                KEY_COLLECTOR => {
                    error!("Adding artifact refs to collector object...");
                    let c = Collector::load(&self.db, &o_p, &self.config);
                    c.insert_base_property(KEY_REF_ARTIFACTS_PATH.to_string(), json!(refs));
                }
                KEY_REPORT => {
                    error!("Adding artifact refs to report object...");
                    let r = Report::load(&self.db, &o_p, &self.config);
                    r.insert_base_property(KEY_REF_ARTIFACTS_PATH.to_string(), json!(refs));
                }
                KEY_TEST => {
                    error!("Adding artifact refs to test object...");
                    let t = Test::load(&self.db, &o_p.vertex.id, &self.config);
                    t.insert_base_property(KEY_REF_ARTIFACTS_PATH.to_string(), json!(refs));
                }
                &_ => {}
            }
        }
    }

    pub fn init_stages(&self, id: Uuid) {
        let project = Project::load(&self.db, &id, &self.config);
        let eut = Eut::load(&self.db, &project, &self.config);
        let ci = Ci::load(&self.db, &project.get_object(), &self.config);
        let ci_o_p_base = ci.get_base_properties();
        let _ci_id_path = ci_o_p_base.get(KEY_ID_PATH).unwrap().as_array().unwrap();
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
        let rtes = self.db.get_object_neighbours_with_properties_out(&_rtes.unwrap().id, EdgeTypes::ProvidesRte);
        let mut _test_stages_seq: Vec<String> = Vec::new();
        let mut _verification_stages_seq: Vec<String> = Vec::new();

        for rte in rtes.iter() {
            let _c = self.db.get_object_neighbour_out(&rte.vertex.id, EdgeTypes::HasConnections);
            let _conns = self.db.get_object_neighbours_out(&_c.unwrap().id, EdgeTypes::HasConnection);
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

        //Test Collector Stages Deploy
        let mut _test_collector_stages: Vec<String> = Vec::new();
        let collectors = Collectors::load(&self.db, &eut.get_object(), &self.config);
        for c in collectors {
            for item in c.get_module_properties().get(KEY_STAGES).unwrap().as_object().unwrap().get(KEY_DEPLOY).unwrap().as_array().unwrap() {
                _test_collector_stages.push(item.as_str().unwrap().to_string());
            }
            /*let edges = self.db.get_object_edges_in(&c.get_id());

            for e in edges {
                match e.t.as_str() {
                    EDGE_TYPE_TEST_REFERS_COLLECTION => {
                        for item in c.get_module_properties().get(KEY_STAGES).unwrap().as_object().unwrap().get(KEY_DEPLOY).unwrap().as_array().unwrap() {
                            _test_collector_stages.push(item.as_str().unwrap().to_string());
                        }
                    }
                    _ => {}
                }
            }*/
        }

        if _test_collector_stages.len() > 0 {
            let test_collector_stage_deploy = self.add_ci_stages(&mut ci_id_path, &test_stage_deploy_seq.unwrap(), _test_collector_stages.as_slice(), &VertexTypes::StageDeploy);

            //Verification Stages Deploy
            let verification_stage_deploy = self.add_ci_stages(&mut ci_id_path, &test_collector_stage_deploy.unwrap(), &self.config.verifications.ci.stages.deploy, &VertexTypes::StageDeploy);
            let verification_stages_seq = self.add_ci_stages(&mut ci_id_path, &verification_stage_deploy.unwrap(), &_verification_stages_seq, &VertexTypes::StageDeploy);
            //Reports Stages Deploy
            self.add_ci_stages(&mut ci_id_path, &verification_stages_seq.unwrap(), &self.config.reports.ci.stages.deploy, &VertexTypes::StageDeploy);
        } else {
            //Verification Stages Deploy
            let verification_stage_deploy = self.add_ci_stages(&mut ci_id_path, &test_stage_deploy_seq.unwrap(), &self.config.verifications.ci.stages.deploy, &VertexTypes::StageDeploy);
            let verification_stages_seq = self.add_ci_stages(&mut ci_id_path, &verification_stage_deploy.unwrap(), &_verification_stages_seq, &VertexTypes::StageDeploy);
            //Reports Stages Deploy
            self.add_ci_stages(&mut ci_id_path, &verification_stages_seq.unwrap(), &self.config.reports.ci.stages.deploy, &VertexTypes::StageDeploy);
        }

        //Feature Stages Destroy
        let mut stage_destroy: Option<Vertex> = None;
        let _features = self.db.get_object_neighbour_out(&eut.get_id(), EdgeTypes::HasFeatures);
        let features = self.db.get_object_neighbours_out(&_features.unwrap().id, EdgeTypes::HasFeature);

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
        let applications = self.db.get_object_neighbours_out(&_applications.unwrap().id, EdgeTypes::ProvidesApplication);

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
    }

    fn load_regression_config(path: &str, file: &str, eut_config: Option<String>) -> RegressionConfig {
        info!("Loading regression configuration data...");
        error!("EUT_CONFIG: {:?}", eut_config);
        let data: String = format!("{path}/{CONFIG_FILE_PATH}/{file}");
        error!("Sense8 config file: {}", &data);
        let raw = std::fs::read_to_string(data).unwrap();
        let _tmp: Value = serde_json::from_str(&raw).unwrap();
        let mut _cfg = _tmp.as_object().unwrap().clone();
        _cfg.insert("root_path".to_string(), Value::from(path.to_string()));
        let mut cfg = serde_json::from_value::<RegressionConfig>(to_value(&_cfg).unwrap()).unwrap();
        let _ = cfg.eut.config.insert(eut_config.clone().unwrap_or_default());

        info!("Loading regression configuration data -> Done.");
        info!("Render regression configuration file...");
        let mut _tera = Tera::new(&*format!("{path}/{CONFIG_FILE_PATH}/*")).unwrap();
        let mut context = Context::new();
        context.insert(KEY_EUT, &cfg.eut);
        context.insert(KEY_RTE, &cfg.rte);
        context.insert(KEY_TESTS, &cfg.tests);
        context.insert(KEY_PROJECT, &cfg.project);
        context.insert(KEY_FEATURES, &cfg.features);
        context.insert(KEY_REPORTS, &cfg.reports);
        context.insert(KEY_DASHBOARD, &cfg.dashboard);
        context.insert(KEY_COLLECTORS, &cfg.collectors);
        context.insert(KEY_APPLICATIONS, &cfg.applications);
        context.insert(KEY_VERIFICATIONS, &cfg.verifications);
        let eutc = _tera.render(file, &context).unwrap();
        info!("Render regression configuration file -> Done.");

        info!("Loading regression configuration data...");
        let _tmp: Value = serde_json::from_str(&eutc).unwrap();
        let mut _cfg = _tmp.as_object().unwrap().clone();
        _cfg.insert("root_path".to_string(), Value::from(path.to_string()));
        let mut cfg = serde_json::from_value::<RegressionConfig>(to_value(&_cfg).unwrap()).unwrap();
        let _ = cfg.eut.config.insert(eut_config.unwrap_or_default());
        info!("Loading regression configuration data -> Done.");

        cfg
    }

    fn add_ci_stages(&self, id_path: &mut Vec<String>, ancestor: &Vertex, stages: &[String], object_type: &VertexTypes) -> Option<Vertex> {
        let mut curr = Vertex { id: Default::default(), t: Default::default() };

        for (i, stage) in stages.iter().enumerate() {
            let (new, _id_path) = self.db.create_object_and_init(object_type.clone(), id_path, stage, 0);
            self.db.add_object_property(&new, &json!({KEY_NAME: stage}), PropertyType::Base);

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
            reports: vec![],
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
        let eut_provider = self.db.get_object_neighbours_with_properties_out(&_eut_providers.unwrap().id, EdgeTypes::ProvidesProvider);

        let mut eut_provider_p_base = Vec::new();
        for p in eut_provider.iter() {
            let name = p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            eut_provider_p_base.push(String::from(name));
        }

        //Process features
        let features_rc: Vec<Box<dyn RenderContext>> = Features::gen_render_ctx(self.db, &eut.get_object(), &self.config);

        //Process applications
        let applications_rc: Vec<Box<dyn RenderContext>> = Applications::gen_render_ctx(self.db, &eut.get_object(), &self.config);

        //Process collectors
        let collectors_rc: Vec<Box<dyn RenderContext>> = Collectors::gen_render_ctx(self.db, &eut.get_object(), &self.config);

        //Process reports
        let reports_rc: Vec<Box<dyn RenderContext>> = Reports::gen_render_ctx(self.db, &eut.get_object(), &self.config);

        //Get EUT sites
        let _sites = self.db.get_object_neighbour_out(&eut.get_id(), EdgeTypes::HasSites);
        let sites = self.db.get_object_neighbours_with_properties_out(&_sites.unwrap().id, EdgeTypes::HasSite);

        //Get EUT rtes
        let _rtes = Rtes::load_collection(&self.db, &eut.get_object(), &self.config);
        let rtes = Rtes::load(&self.db, &_rtes.get_object(), &self.config);

        //Process eut rtes
        let mut rtes_rc: Vec<RteRenderContext> = Vec::new();
        let mut rte_names: Vec<String> = Vec::new();

        for rte in rtes {
            let rte_base_p = rte.get_base_properties();
            let rte_name = rte_base_p.get(KEY_NAME).unwrap().as_str().unwrap();
            rte_names.push(rte_name.to_string());

            let scripts = rte.gen_script_render_ctx(&self.config);
            let _rte_crcs = rte.gen_render_ctx(&self.config, scripts.clone());

            let rte_crcs: &RteRenderContext = match _rte_crcs.as_any().downcast_ref::<RteRenderContext>() {
                Some(r) => r,
                None => panic!("not a RteRenderContext!"),
            };

            for component in rte_crcs.components.iter() {
                actions.rtes.push(component.job.clone());
            }

            for test in rte_crcs.tests.iter() {
                actions.tests.push(test.job.clone());
                for verification in test.verifications.iter() {
                    actions.verifications.push(verification.job.clone());
                }
            }

            rtes_rc.push(rte_crcs.clone());
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
                let path = format!("{}/{}/{}/{}/{}",
                                   self.config.root_path,
                                   self.config.eut.path,
                                   eut_name, scripts_path,
                                   script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
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
                    artifacts_path: eut_p_base.get(KEY_ARTIFACTS_PATH).unwrap().as_str().unwrap().to_string(),
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
            sites: eut_sites,
            module: eut_p_module.clone(),
            provider: eut_provider_p_base.clone(),
            project: self.config.project.clone(),
        };

        let mut stages: Vec<String> = Vec::new();
        let mut deploy_stages: Vec<String> = Vec::new();
        let mut destroy_stages: Vec<String> = Vec::new();

        let project_ci = self.db.get_object_neighbour_out(&project.get_id(), EdgeTypes::HasCi);
        let s_deploy = self.db.get_object_neighbour_with_properties_out(&project_ci.clone().unwrap().id, EdgeTypes::HasDeployStages).unwrap();
        let s_destroy = self.db.get_object_neighbour_with_properties_out(&project_ci.unwrap().id, EdgeTypes::HasDestroyStages).unwrap();
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
        context.insert(KEY_REPORTS, &reports_rc);
        context.insert(KEY_ACTIONS, &actions);
        context.insert(KEY_PROJECT, &project_rc);
        context.insert(KEY_FEATURES, &features_rc);
        context.insert(KEY_DASHBOARD, &dashboard_rc);
        context.insert(KEY_COLLECTORS, &collectors_rc);
        context.insert(KEY_APPLICATIONS, &applications_rc);

        //error!("{:#?}", context.get(KEY_APPLICATIONS));
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
        info!("Render first step template file {}", self.template);
        let mut _tera = Tera::new(&self.template).unwrap();
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

        let mut _tera = Tera::new(&self.template).unwrap();
        _tera.render("graph.tpl", &context).unwrap()
    }

    pub fn render_entry_page(&self, context: &Context) -> Result<String, Box<dyn Error>> {
        error!("Render entry page..");
        let mut _tera = Tera::new(&self.template).unwrap();
        Ok(_tera.render("entry.tpl", context).unwrap())
    }

    pub fn render_actions_json_file(&self, context: &Context) -> Result<String, Box<dyn Error>> {
        error!("Render actions json file..");
        let mut _tera = Tera::new(&self.template).unwrap();
        Ok(_tera.render("actions.tpl", context).unwrap())
    }
}