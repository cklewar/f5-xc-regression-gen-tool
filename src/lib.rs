use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::io::Write;

use indradb::{Vertex, VertexProperties};
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Map, to_value, Value};
use serde_json::Value::Null;
use tera::{Context, Tera};
use uuid::Uuid;

use crate::constants::*;
use crate::db::Db;

pub mod constants;
pub mod db;

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
    HasProviders,
    NeedsProvider,
    HasComponents,
    HasConnection,
    SiteRefersRte,
    HasConnections,
    HasDeployStages,
    HasComponentSrc,
    HasComponentDst,
    ProvidesProvider,
    HasConnectionSrc,
    HasConnectionDst,
    HasDestroyStages,
    FeatureRefersSite,
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
            EdgeTypes::NeedsShare => EDGE_TYPE_NEEDS_SHARE,
            EdgeTypes::HasFeatures => EDGE_TYPE_HAS_FEATURES,
            EdgeTypes::ProvidesRte => EDGE_TYPE_PROVIDES_RTE,
            EdgeTypes::HasProviders => EDGE_TYPE_HAS_PROVIDERS,
            EdgeTypes::UsesProvider => EDGE_TYPE_USES_PROVIDER,
            EdgeTypes::NeedsProvider => EDGE_TYPE_NEEDS_PROVIDER,
            EdgeTypes::HasComponents => EDGE_TYPE_HAS_COMPONENTS,
            EdgeTypes::SiteRefersRte => EDGE_TYPE_SITE_REFERS_RTE,
            EdgeTypes::HasConnection => EDGE_TYPE_HAS_CONNECTION,
            EdgeTypes::HasConnections => EDGE_TYPE_HAS_CONNECTIONS,
            EdgeTypes::HasComponentSrc => EDGE_TYPE_HAS_COMPONENT_SRC,
            EdgeTypes::HasComponentDst => EDGE_TYPE_HAS_COMPONENT_DST,
            EdgeTypes::HasDeployStages => EDGE_TYPE_HAS_DEPLOY_STAGES,
            EdgeTypes::HasDestroyStages => EDGE_TYPE_HAS_DESTROY_STAGES,
            EdgeTypes::ProvidesProvider => EDGE_TYPE_PROVIDES_PROVIDER,
            EdgeTypes::HasConnectionSrc => EDGE_TYPE_HAS_CONNECTION_SRC,
            EdgeTypes::HasConnectionDst => EDGE_TYPE_HAS_CONNECTION_DST,
            EdgeTypes::FeatureRefersSite => EDGE_TYPE_FEATURE_REFERS_SITE,
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
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::HasDeployStages.name());
        map.insert(VertexTuple(VertexTypes::Ci.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::HasDestroyStages.name());
        map.insert(VertexTuple(VertexTypes::StageDeploy.name().to_string(), VertexTypes::StageDeploy.name().to_string()), EdgeTypes::NextStage.name());
        map.insert(VertexTuple(VertexTypes::StageDestroy.name().to_string(), VertexTypes::StageDestroy.name().to_string()), EdgeTypes::NextStage.name());
        map.insert(VertexTuple(VertexTypes::Features.name().to_string(), VertexTypes::Feature.name().to_string()), EdgeTypes::HasFeature.name());
        map.insert(VertexTuple(VertexTypes::Feature.name().to_string(), VertexTypes::Site.name().to_string()), EdgeTypes::FeatureRefersSite.name());
        map.insert(VertexTuple(VertexTypes::Scripts.name().to_string(), VertexTypes::Script.name().to_string()), EdgeTypes::Has.name());
        map
    };
    static ref EDGES_COUNT: usize = EDGE_TYPES.len();
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
    data_vars_path: String,
    data_scripts_path: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct RegressionConfigVerifications {
    path: String,
    ci: RegressionConfigGenericCi,
}

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
struct RegressionConfigProjectVars {
    file: String,
    path: String,
}

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
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
    job: String,
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
    name: String,
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
struct ScriptFeatureRenderContext {
    eut: String,
    name: String,
    sites: String,
    release: String,
    project: RegressionConfigProject,
    provider: Vec<String>,

}

#[derive(Serialize, Debug)]
struct ScriptVerificationRenderContext {
    rte: String,
    name: String,
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
    module: String,
    provider: String,
    features: Vec<String>,
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
    vars: RegressionConfigProjectVars,
    sites: String,
    counter: usize,
    project: String,
    provider: String,
}

pub trait RenderContext {}

impl RenderContext for ScriptEutRenderContext {}

impl RenderContext for ScriptRteRenderContext {}

impl RenderContext for ScriptFeatureRenderContext {}

impl RenderContext for ScriptTestRenderContext {}

impl RenderContext for ScriptVerificationRenderContext {}

impl RenderContext for ScriptRteProviderShareRenderContext {}

pub fn render_script(context: &(impl RenderContext + serde::Serialize), input: &str) -> String {
    info!("Render script context...");
    let ctx = Context::from_serialize(context);
    let rendered = Tera::one_off(input, &ctx.unwrap(), false).unwrap();
    info!("Render script context -> Done.");
    rendered
}

struct RteCtxParameters<'a> {
    rte: &'a VertexProperties,
    config: &'a RegressionConfig,
    eut_name: String,
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
            let rte_job_name = format!("{}_{}_{}_{}_{}_{}", KEY_RTE, params.rte_name, &connection_name, &src_p_name, &src_name, &comp_src_name).replace('_', "-");

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
                        let path = format!("{}/{}/{}/{}/{}/{}/{}", params.config.project.root_path, params.config.rte.path, params.rte_name, scripts_path, p_name, comp_src_name, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                        let contents = std::fs::read_to_string(&path).expect("panic while opening rte apply.script file");
                        let ctx = ScriptRteRenderContext {
                            rte: params.rte_name.to_string(),
                            eut: params.eut_name.to_string(),
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
                let dst_name = dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let dst_site = self.db.get_object_neighbour_with_properties_out(&dst.vertex.id, EdgeTypes::RefersSite).unwrap();
                let dst_site_name = dst_site.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let dst_provider = self.db.get_object_neighbour_with_properties_out(&dst_site.vertex.id, EdgeTypes::UsesProvider).unwrap();
                let dst_p_name = dst_provider.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let comp_dst = self.db.get_object_neighbour_with_properties_out(&dst.vertex.id, EdgeTypes::HasComponentDst).unwrap();
                let comp_dst_name = &comp_dst.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let rte_job_name = format!("{}_{}_{}_{}_{}_{}", KEY_RTE, &params.rte_name, &connection_name, &dst_p_name, &dst_name, &comp_dst_name).replace('_', "-");

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
                            let path = format!("{}/{}/{}/{}/{}/{}/{}", params.config.project.root_path, params.config.rte.path, params.rte_name, scripts_path, p_name, comp_dst_name, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                            let contents = std::fs::read_to_string(path).expect("panic while opening rte apply.script file");
                            let ctx = ScriptRteRenderContext {
                                rte: params.rte_name.to_string(),
                                eut: params.eut_name.to_string(),
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
                                         KEY_TEST,
                                         params.rte_name,
                                         src_name,
                                         t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap()
                ).replace('_', "-");

                //Process test scripts
                let t_name = t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let t_module = t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                let scripts_path = t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
                for script in t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                    let path = format!("{}/{}/{}/{}/{}", params.config.project.root_path, params.config.tests.path, t_module, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                    let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
                    let ctx = ScriptTestRenderContext {
                        rte: params.rte_name.to_string(),
                        eut: params.eut_name.to_string(),
                        name: t_name.to_string(),
                        module: t_module.to_string(),
                        provider: src_name.to_string(),
                        features: params.features.to_vec(),
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
                    let scripts_path = v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                    let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
                    for script in v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                        let path = format!("{}/{}/{}/{}/{}", params.config.project.root_path, params.config.verifications.path, v_module, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                        let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
                        let ctx = ScriptVerificationRenderContext {
                            rte: params.rte_name.to_string(),
                            name: v_name.to_string(),
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
                        name: v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                        module: v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                        scripts,
                    };
                    verifications.push(rte_vrc);
                }

                let rterc = RteTestRenderContext {
                    ci: t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_CI).unwrap().as_object().unwrap().clone(),
                    rte: params.rte_name.to_string(),
                    job: t_job_name,
                    name: t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                    module: t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
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
        error!("RTE TYPE B init connection component --> {:?}", &r_o);
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
        error!("RTE TYPE B build ctx --> {:?}", rte);
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
                let rte_job_name = format!("{}_{}_{}_{}_{}", KEY_RTE, params.rte_name, &conn_name, &p_name, &conn_src_name).replace('_', "-");

                for script in comp_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                    let path = format!("{}/{}/{}/{}/{}/{}/{}", params.config.project.root_path, params.config.rte.path, params.rte_name, scripts_path, p_name, comp_src_name, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                    let contents = std::fs::read_to_string(&path).expect("panic while opening rte apply.script file");
                    let ctx = ScriptRteRenderContext {
                        rte: params.rte_name.to_string(),
                        eut: params.eut_name.to_string(),
                        site: "".to_string(),
                        release: "".to_string(),
                        provider: p_name.to_string(),
                        project: params.config.project.clone(),
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
                let t_job_name = format!("{}_{}_{}_{}",
                                         KEY_TEST,
                                         params.rte_name,
                                         conn_src_name,
                                         t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap()
                ).replace('_', "-");

                //Process test scripts
                let t_name = t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                let t_module = t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                let scripts_path = t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
                for script in t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                    let path = format!("{}/{}/{}/{}/{}", params.config.project.root_path, params.config.tests.path, t_module, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                    let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
                    let ctx = ScriptTestRenderContext {
                        rte: params.rte_name.to_string(),
                        eut: params.eut_name.to_string(),
                        name: t_name.to_string(),
                        module: t_module.to_string(),
                        provider: component_provider.to_string(),
                        features: params.features.to_vec(),
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
                                             conn_src_name,
                                             &t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                             v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                    ).replace('_', "-");

                    //Process test scripts
                    let v_name = v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                    let v_module = v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                    let scripts_path = v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
                    let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
                    for script in v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                        let path = format!("{}/{}/{}/{}/{}", params.config.project.root_path, params.config.verifications.path, v_module, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                        let contents = std::fs::read_to_string(path).expect("panic while opening test script file");
                        let ctx = ScriptVerificationRenderContext {
                            rte: params.rte_name.to_string(),
                            name: v_name.to_string(),
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
                        ci: v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_CI).unwrap().as_object().unwrap().clone(),
                        test: t_name.to_string(),
                        rte: params.rte_name.to_string(),
                        job: v_job_name,
                        name: v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                        module: v.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                        scripts,
                    };
                    verifications.push(rte_vrc);
                }

                let rterc = RteTestRenderContext {
                    ci: t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_CI).unwrap().as_object().unwrap().clone(),
                    rte: params.rte_name.to_string(),
                    job: t_job_name,
                    name: t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap().to_string(),
                    module: t.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap().to_string(),
                    provider: conn_src_name.to_string(),
                    scripts,
                    verifications,
                };
                params.rte_crcs.tests.push(rterc);
            }
        }
    }
}

struct Rte<T> {
    rte: T,
    _type: String,
}

impl<'a> Rte<Box<dyn RteCharacteristics + 'a>> {
    fn new(rte_type: &str, db: &'a Db) -> Option<Rte<Box<dyn RteCharacteristics + 'a>>> {
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
    pub config: RegressionConfig,
    db: &'a Db,
}

impl<'a> Regression<'a> {
    pub fn new(db: &'a Db, path: &str, file: &str) -> Self {
        Regression {
            config: Regression::load_regression_config(path, file),
            db,
        }
    }

    pub fn init(&self) -> Uuid {
        // Project
        let project = self.db.create_object(VertexTypes::Project);
        self.db.add_object_properties(&project, &self.config.project, PropertyType::Base);
        self.db.add_object_properties(&project, &json!({
            KEY_GVID: self.config.project.name.replace('-', "_"),
            KEY_GV_LABEL: self.config.project.name.replace('-', "_"),
        }), PropertyType::Gv);

        // Ci
        let ci = self.db.create_object(VertexTypes::Ci);
        self.db.add_object_properties(&ci, &self.config.ci, PropertyType::Base);
        self.db.add_object_properties(&ci, &json!({
            KEY_GVID: format!("{}_{}", self.config.project.name.replace('-', "_"), KEY_CI),
            KEY_GV_LABEL: KEY_CI,
        }), PropertyType::Gv);
        self.db.create_relationship(&project, &ci);

        // Eut
        let eut = self.db.create_object(VertexTypes::Eut);
        self.db.add_object_properties(&eut, &self.config.eut, PropertyType::Base);
        self.db.add_object_properties(&eut, &json!({
            KEY_GVID: self.config.eut.module.replace('-', "_"),
            KEY_GV_LABEL: self.config.eut.module,
        }), PropertyType::Gv);

        let module = self.load_object_config(VertexTypes::get_name_by_object(&eut), &self.config.eut.module);
        let v = to_value(module).unwrap();
        self.db.create_relationship(&project, &eut);

        let eut_providers = self.db.create_object(VertexTypes::Providers);
        self.db.create_relationship(&eut, &eut_providers);

        for k in EUT_KEY_ORDER.iter() {
            let obj = v.as_object().unwrap().get(*k).unwrap();
            match *k {
                k if k == KEY_NAME => {
                    let eut_o_p = self.db.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.to_string(), obj.clone());
                    self.db.add_object_properties(&eut, &p, PropertyType::Module);
                    self.db.add_object_properties(&eut_providers, &json!({
                        KEY_GVID: format!("{}_{}_{}", eut.t.as_str(), KEY_PROVIDERS, &obj.as_str().unwrap()),
                        KEY_GV_LABEL: KEY_PROVIDERS
                    }), PropertyType::Gv);
                }
                k if k == KEY_RELEASE => {
                    let eut_o_p = self.db.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.to_string(), obj.clone());
                    self.db.add_object_properties(&eut, &p, PropertyType::Module);
                }
                k if k == KEY_PROVIDER => {
                    for p in obj.as_array().unwrap().iter() {
                        let p_o = self.db.create_object(VertexTypes::EutProvider);
                        self.db.create_relationship(&eut_providers, &p_o);
                        self.db.add_object_properties(&p_o, &json!({KEY_NAME: &p.as_str().unwrap()}), PropertyType::Base);
                        self.db.add_object_properties(&p_o, &json!({
                            KEY_GVID: format!("{}_{}_{}", eut.t.as_str(), KEY_PROVIDER, &p.as_str().unwrap()),
                            KEY_GV_LABEL: &p.as_str().unwrap()
                        }), PropertyType::Gv);
                    }
                }
                k if k == KEY_CI => {
                    let eut_o_p = self.db.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.to_string(), obj.clone());
                    self.db.add_object_properties(&eut, &p, PropertyType::Module);
                }
                k if k == KEY_SITES => {
                    let o = self.db.create_object(VertexTypes::get_type_by_key(k));
                    self.db.create_relationship(&eut, &o);
                    self.db.add_object_properties(&o, &json!({
                        KEY_GVID: format!("{}_{}", self.config.eut.module, k),
                        KEY_GV_LABEL: k
                    }), PropertyType::Gv);
                    let _p = self.db.get_object_neighbour_out(&eut.id, EdgeTypes::HasProviders);
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
                                let s_o = self.db.create_object(VertexTypes::Site);
                                self.db.create_relationship(&o, &s_o);
                                let provider = &site_attr.as_object().unwrap().get(KEY_PROVIDER).unwrap().as_str().unwrap();
                                let p_o = self.db.get_object(id_name_map.get(provider).unwrap());
                                self.db.create_relationship(&s_o, &p_o);
                                self.db.add_object_properties(&s_o, &json!({KEY_NAME: site_name}), PropertyType::Base);
                                self.db.add_object_properties(&s_o, &json!({
                                KEY_GVID: format!("{}_{}_{}", self.config.eut.module, k, site_name),
                                KEY_GV_LABEL: site_name
                            }), PropertyType::Gv);
                            }
                            Ordering::Greater => {
                                for c in 1..=site_count {
                                    let s_o = self.db.create_object(VertexTypes::Site);
                                    self.db.create_relationship(&o, &s_o);
                                    let provider = &site_attr.as_object().unwrap().get(KEY_PROVIDER).unwrap().as_str().unwrap();
                                    let p_o = self.db.get_object(id_name_map.get(provider).unwrap());
                                    self.db.create_relationship(&s_o, &p_o);
                                    self.db.add_object_properties(&s_o, &json!({KEY_NAME: format!("{}_{}", site_name, c)}),
                                                                  PropertyType::Base);
                                    self.db.add_object_properties(&s_o, &json!({
                                    KEY_GVID: format!("{}_{}_{}_{}", self.config.eut.module, k, site_name, c),
                                    KEY_GV_LABEL: format!("{}_{}", site_name, c)
                                }), PropertyType::Gv);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                k if k == KEY_FEATURES => {
                    let o = self.db.create_object(VertexTypes::get_type_by_key(k));
                    self.db.create_relationship(&eut, &o);
                    self.db.add_object_properties(&o, &json!({
                        KEY_GVID: format!("{}_{}", self.config.eut.module, k),
                        KEY_GV_LABEL: k
                    }), PropertyType::Gv);

                    for f in obj.as_array().unwrap().iter() {
                        let f_module = f.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                        let f_sites = f.as_object().unwrap().get(KEY_SITES).unwrap().as_array().unwrap();
                        let _sites = self.db.get_object_neighbour_out(&eut.id, EdgeTypes::HasSites);
                        let sites = self.db.get_object_neighbours_with_properties_out(&_sites.id, EdgeTypes::HasSite);
                        let f_o = self.db.create_object(VertexTypes::Feature);
                        self.db.create_relationship(&o, &f_o);
                        self.db.add_object_properties(&f_o, f.as_object().unwrap(), PropertyType::Base);
                        self.db.add_object_properties(&f_o, &json!({
                                KEY_GVID: format!("{}_{}_{}", KEY_FEATURE, self.config.eut.module, f_module),
                                KEY_GV_LABEL: &f_module
                            }), PropertyType::Gv);

                        //Feature -> Site
                        for f_site in f_sites {
                            let re = Regex::new(f_site.as_str().unwrap()).unwrap();
                            for site in sites.iter() {
                                let site_name = site.props.get(PropertyType::Base.index()).unwrap().value.as_object().
                                    unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                                if let Some(_t) = re.captures(site_name) {
                                    self.db.create_relationship(&f_o, &site.vertex);
                                }
                            }
                        }

                        // FEATURE MODULE CFG
                        let cfg = self.load_object_config(VertexTypes::get_name_by_object(&f_o), f_module);
                        for (k, v) in cfg.as_object().unwrap().iter() {
                            match k {
                                k if k == KEY_SCRIPTS_PATH => {
                                    let f_o_p = self.db.get_object_properties(&f_o).unwrap().props;
                                    let mut p = f_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                    p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                    self.db.add_object_properties(&f_o, &p, PropertyType::Module);
                                }
                                k if k == KEY_RELEASE => {
                                    let f_o_p = self.db.get_object_properties(&f_o).unwrap().props;
                                    let mut p = f_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                    p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                    self.db.add_object_properties(&f_o, &p, PropertyType::Module);
                                }
                                k if k == KEY_NAME => {
                                    let f_o_p = self.db.get_object_properties(&f_o).unwrap().props;
                                    let mut p = f_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                    p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                    self.db.add_object_properties(&f_o, &p, PropertyType::Module);
                                }
                                k if k == KEY_CI => {
                                    let f_o_p = self.db.get_object_properties(&f_o).unwrap().props;
                                    let mut p = f_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                    p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                    self.db.add_object_properties(&f_o, &p, PropertyType::Module);
                                }
                                k if k == KEY_SCRIPTS => {
                                    let f_o_p = self.db.get_object_properties(&f_o).unwrap().props;
                                    let mut p = f_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                    p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                    self.db.add_object_properties(&f_o, &p, PropertyType::Module);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                k if k == KEY_SCRIPTS_PATH => {
                    let eut_o_p = self.db.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.to_string(), obj.clone());
                    self.db.add_object_properties(&eut, &p, PropertyType::Module);
                }
                k if k == KEY_SCRIPTS => {
                    let eut_o_p = self.db.get_object_properties(&eut).unwrap().props;
                    let mut p = eut_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                    p.insert(k.to_string(), obj.clone());
                    self.db.add_object_properties(&eut, &p, PropertyType::Module);
                }
                k if k == KEY_RTES => {
                    let o = self.db.create_object(VertexTypes::get_type_by_key(k));
                    self.db.create_relationship(&eut, &o);

                    for rte in obj.as_array().unwrap().iter() {
                        let r_o = self.db.create_object(VertexTypes::Rte);
                        self.db.create_relationship(&o, &r_o);
                        self.db.add_object_properties(&r_o, &json!({
                            KEY_GVID: format!("{}_{}", KEY_RTE, &rte.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap()),
                            KEY_GV_LABEL: &rte.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap()
                        }), PropertyType::Gv);

                        let rte_p_o = self.db.create_object(VertexTypes::Providers);
                        self.db.create_relationship(&r_o, &rte_p_o);
                        self.db.add_object_properties(&rte_p_o, &json!({
                            KEY_GVID: format!("{}_{}_{}", KEY_RTE, KEY_PROVIDERS, &rte.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                            KEY_GV_LABEL: KEY_PROVIDERS
                        }), PropertyType::Gv);

                        //RTE -> Features
                        let eut_f_o = self.db.get_object_neighbour_out(&eut.id, EdgeTypes::HasFeatures);
                        self.db.create_relationship(&r_o, &eut_f_o);

                        //Rte
                        for (k, v) in rte.as_object().unwrap().iter() {
                            match k {
                                k if k == KEY_MODULE => {
                                    let r_o_p = self.db.get_object_properties(&r_o).unwrap().props;
                                    let mut p = r_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                    p.insert(k.clone(), v.clone());
                                    self.db.add_object_properties(&r_o, &p, PropertyType::Base);
                                }
                                // Active Provider
                                k if k == KEY_PROVIDER => {
                                    let r_o_p = self.db.get_object_properties(&r_o).unwrap().props;
                                    let mut p = r_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                    p.insert(k.clone(), v.clone());
                                    self.db.add_object_properties(&r_o, &p, PropertyType::Base);
                                }
                                //Collector
                                k if k == "collector" => {
                                    let c_o = self.db.create_object(VertexTypes::get_type_by_key(k));
                                    self.db.create_relationship(&r_o, &c_o);
                                    self.db.add_object_properties(&c_o, &json!({
                                                KEY_GVID: format!("{}_{}", &k, &rte.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                KEY_GV_LABEL: &c_o.t.as_str()
                                            }), PropertyType::Gv);
                                }
                                //Connections
                                k if k == KEY_CONNECTIONS => {
                                    let cs_o = self.db.create_object(VertexTypes::get_type_by_key(k));
                                    self.db.add_object_properties(&cs_o, &json!({
                                                KEY_GVID: format!("{}_{}", &k, &rte.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                KEY_GV_LABEL: &cs_o.t.as_str()
                                            }), PropertyType::Gv);
                                    self.db.create_relationship(&r_o, &cs_o);

                                    for item in v.as_array().unwrap().iter() {
                                        //Connection
                                        let c_o = self.db.create_object(VertexTypes::Connection);
                                        self.db.create_relationship(&cs_o, &c_o);
                                        let c_name = item.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                                        self.db.add_object_properties(&c_o, &json!({KEY_NAME: c_name}), PropertyType::Base);
                                        self.db.add_object_properties(&c_o, &json!({
                                                KEY_GVID: format!("{}_{}_{}", KEY_CONNECTION, c_name.replace('-', "_"), &rte.as_object().
                                                    unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                KEY_GV_LABEL: c_name
                                            }), PropertyType::Gv);

                                        //Connection Source
                                        let source = item.as_object().unwrap().get(KEY_SOURCE).unwrap().as_str().unwrap();
                                        let src_o = self.db.create_object(VertexTypes::ConnectionSrc);
                                        self.db.create_relationship(&c_o, &src_o);
                                        self.db.add_object_properties(&src_o, &json!({KEY_NAME: &source, KEY_RTE: &rte.as_object().
                                            unwrap().get(KEY_MODULE).unwrap().as_str().unwrap()}), PropertyType::Base);
                                        self.db.add_object_properties(&src_o, &json!({
                                            KEY_GVID: format!("{}_{}_{}_{}", "connection_src", &c_name.replace('-',"_"), &source, &rte.as_object().unwrap().
                                                get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                            KEY_GV_LABEL: source
                                        }), PropertyType::Gv);

                                        let _sites = self.db.get_object_neighbour_out(&eut.id, EdgeTypes::HasSites);
                                        let sites = self.db.get_object_neighbours_with_properties_out(&_sites.id, EdgeTypes::HasSite);

                                        //Connection Source -> Site
                                        for s in sites.iter() {
                                            let site_name = s.props.get(PropertyType::Base.index()).unwrap().value.as_object().
                                                unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                                            if site_name == source {
                                                self.db.create_relationship(&src_o, &s.vertex);
                                                //site --> rte
                                                self.db.create_relationship(&s.vertex, &r_o);
                                            }
                                        }

                                        //Connection Destinations
                                        let destinations = item.as_object().unwrap().get("destinations").unwrap().as_array().unwrap();

                                        for d in destinations.iter() {
                                            let re = Regex::new(d.as_str().unwrap()).unwrap();

                                            for site in sites.iter() {
                                                let site_name = site.props.get(PropertyType::Base.index()).unwrap().value.as_object().
                                                    unwrap().get(KEY_NAME).unwrap().as_str().unwrap();

                                                if let Some(_t) = re.captures(site_name) {
                                                    let dst_o = self.db.create_object(VertexTypes::ConnectionDst);
                                                    self.db.create_relationship(&src_o, &dst_o);
                                                    self.db.add_object_properties(&dst_o, &json!({KEY_NAME: &d, KEY_RTE: &rte.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap()}), PropertyType::Base);
                                                    self.db.add_object_properties(&dst_o, &json!({
                                                     KEY_GVID: format!("{}_{}_{}_{}", "connection_dst", &c_name.replace('-',"_"), d.as_str().unwrap(), &rte.as_object().
                                                        unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                     KEY_GV_LABEL: d.as_str().unwrap()
                                                 }), PropertyType::Gv);
                                                    //Connection Destination -> Site
                                                    self.db.create_relationship(&dst_o, &site.vertex);
                                                    //site --> rte
                                                    self.db.create_relationship(&site.vertex, &r_o);
                                                }
                                            }
                                        }

                                        //Tests
                                        let tests = item.as_object().unwrap().get(KEY_TESTS).unwrap().as_array().unwrap();
                                        for test in tests.iter() {
                                            let t_o = self.db.create_object(VertexTypes::Test);
                                            self.db.create_relationship(&src_o, &t_o);

                                            for (k, v) in test.as_object().unwrap().iter() {
                                                match k {
                                                    k if k == KEY_NAME => {
                                                        let t_o_p = self.db.get_object_properties(&t_o).unwrap().props;
                                                        let mut p = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                        p.insert(k.clone(), v.clone());
                                                        self.db.add_object_properties(&t_o, &p, PropertyType::Base);
                                                    }
                                                    k if k == KEY_MODULE => {
                                                        let t_o_p = self.db.get_object_properties(&t_o).unwrap().props;
                                                        let mut p = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                        p.insert(k.clone(), v.clone());
                                                        self.db.add_object_properties(&t_o, &p, PropertyType::Base);
                                                    }
                                                    k if k == "parallel" => {
                                                        let mut t_o_p = self.db.get_object_properties(&t_o).unwrap().props;
                                                        let mut p = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                        p.insert(k.clone(), v.clone());
                                                        self.db.add_object_properties(&t_o, &p, PropertyType::Base);
                                                        t_o_p = self.db.get_object_properties(&t_o).unwrap().props;
                                                        let t_name = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                                                        let t_module = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                                                        self.db.add_object_properties(&t_o, &json!({
                                                                             KEY_GVID: format!("{}_{}_{}", t_o.t.as_str(), t_name.replace('-', "_"), &rte.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                             KEY_GV_LABEL: format!("t_{}", t_module)
                                                                         }), PropertyType::Gv);
                                                    }
                                                    k if k == KEY_CI => {
                                                        let t_o_p = self.db.get_object_properties(&t_o).unwrap().props;
                                                        let mut p = t_o_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().clone();
                                                        p.append(&mut json!({k: v.as_object().unwrap().clone()}).as_object().unwrap().clone());
                                                        self.db.add_object_properties(&t_o, &p, PropertyType::Base);
                                                    }
                                                    k if k == KEY_VERIFICATIONS => {
                                                        for v in v.as_array().unwrap().iter() {
                                                            let v_o = self.db.create_object(VertexTypes::Verification);
                                                            self.db.create_relationship(&t_o, &v_o);
                                                            let v_name = v.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
                                                            let v_module = v.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                                                            self.db.add_object_properties(&v_o, v, PropertyType::Base);
                                                            self.db.add_object_properties(&v_o, &json!({
                                                                                 KEY_GVID: format!("{}_{}_{}", v_o.t.as_str(), v_name.replace('-', "_"), &rte.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                                 KEY_GV_LABEL: format!("v_{}", v_module)
                                                                             }), PropertyType::Gv);
                                                            // Verification module cfg
                                                            let cfg = self.load_object_config(VertexTypes::get_name_by_object(&v_o), v_module);
                                                            for (k, v) in cfg.as_object().unwrap().iter() {
                                                                match k {
                                                                    k if k == KEY_NAME => {
                                                                        let v_o_p = self.db.get_object_properties(&v_o).unwrap().props;
                                                                        let mut p = v_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                        self.db.add_object_properties(&v_o, &p, PropertyType::Module);
                                                                    }
                                                                    k if k == KEY_SCRIPTS => {
                                                                        let v_o_p = self.db.get_object_properties(&v_o).unwrap().props;
                                                                        let mut p = v_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                        self.db.add_object_properties(&v_o, &p, PropertyType::Module);
                                                                    }
                                                                    k if k == KEY_SCRIPTS_PATH => {
                                                                        let v_o_p = self.db.get_object_properties(&v_o).unwrap().props;
                                                                        let mut p = v_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                                        self.db.add_object_properties(&v_o, &p, PropertyType::Module);
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
                                            let t_p = self.db.get_object_properties(&t_o).unwrap().props;
                                            let module = t_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                                            let cfg = self.load_object_config(VertexTypes::get_name_by_object(&t_o), module);
                                            for (k, v) in cfg.as_object().unwrap().iter() {
                                                match k {
                                                    k if k == KEY_NAME => {
                                                        let t_o_p = self.db.get_object_properties(&t_o).unwrap().props;
                                                        let mut p = t_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                        self.db.add_object_properties(&t_o, &p, PropertyType::Module);
                                                    }
                                                    k if k == KEY_SCRIPTS => {
                                                        let t_o_p = self.db.get_object_properties(&t_o).unwrap().props;
                                                        let mut p = t_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                        self.db.add_object_properties(&t_o, &p, PropertyType::Module);
                                                    }
                                                    k if k == KEY_SCRIPTS_PATH => {
                                                        let t_o_p = self.db.get_object_properties(&t_o).unwrap().props;
                                                        let mut p = t_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                                        p.append(&mut json!({k: v.clone()}).as_object().unwrap().clone());
                                                        self.db.add_object_properties(&t_o, &p, PropertyType::Module);
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
                        let r_p = self.db.get_object_properties(&r_o).unwrap().props;
                        let module = r_p.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap();
                        let cfg = self.load_object_config(VertexTypes::get_name_by_object(&r_o), module);

                        for (k, v) in cfg.as_object().unwrap().iter() {
                            match k {
                                k if k == KEY_NAME => {
                                    let r_o_p = self.db.get_object_properties(&r_o).unwrap().props;
                                    let mut p = r_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                    p.append(&mut json!({k: v}).as_object().unwrap().clone());
                                    self.db.add_object_properties(&r_o, &p, PropertyType::Module);
                                }
                                k if k == KEY_TYPE => {
                                    let r_o_p = self.db.get_object_properties(&r_o).unwrap().props;
                                    let mut p = r_o_p.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().clone();
                                    p.append(&mut json!({k: v}).as_object().unwrap().clone());
                                    self.db.add_object_properties(&r_o, &p, PropertyType::Module);
                                }
                                k if k == KEY_PROVIDER => {
                                    for (p, v) in v.as_object().unwrap().iter() {
                                        let o = self.db.create_object(VertexTypes::RteProvider);
                                        self.db.create_relationship(&rte_p_o, &o);
                                        self.db.add_object_properties(&o, &json!({KEY_NAME: p}), PropertyType::Module);
                                        self.db.add_object_properties(&o, &json!({
                                                KEY_GVID: format!("{}_{}_{}", KEY_PROVIDER, p, &rte.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                KEY_GV_LABEL: p
                                            }), PropertyType::Gv);

                                        for (k, v) in v.as_object().unwrap().iter() {
                                            match k {
                                                k if k == KEY_CI => {
                                                    let p_ci_o = self.db.create_object(VertexTypes::Ci);
                                                    self.db.create_relationship(&o, &p_ci_o);
                                                    self.db.add_object_properties(&p_ci_o, &json!({
                                                            KEY_GVID: format!("{}_{}_{}_{}", KEY_PROVIDER, k, p, &rte.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                            KEY_GV_LABEL: k
                                                        }), PropertyType::Gv);
                                                    self.db.add_object_properties(&p_ci_o, &v.as_object().unwrap(), PropertyType::Base);
                                                }
                                                k if k == KEY_SHARE => {
                                                    let s_o = self.db.create_object(VertexTypes::Share);
                                                    self.db.create_relationship(&o, &s_o);
                                                    self.db.add_object_properties(&s_o, &json!({
                                                        KEY_GVID: format!("{}_{}_{}_{}", KEY_PROVIDER, k, p, &rte.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                        KEY_GV_LABEL: k
                                                    }), PropertyType::Gv);
                                                    self.db.add_object_properties(&s_o, &v.as_object().unwrap(), PropertyType::Base);
                                                }
                                                k if k == KEY_COMPONENTS => {
                                                    let c_o = self.db.create_object(VertexTypes::Components);
                                                    self.db.create_relationship(&o, &c_o);
                                                    self.db.add_object_properties(&c_o, &json!({
                                                            KEY_GVID: format!("{}_{}_{}_{}", KEY_PROVIDER, p, k, &rte.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                            KEY_GV_LABEL: k
                                                        }), PropertyType::Gv);

                                                    for (k, v) in v.as_object().unwrap().iter() {
                                                        match k {
                                                            k if k == KEY_SRC => {
                                                                let c_src_o = self.db.create_object(VertexTypes::ComponentSrc);
                                                                self.db.create_relationship(&c_o, &c_src_o);
                                                                self.db.add_object_properties(&c_src_o, &json!({
                                                                        KEY_GVID: format!("{}_{}_{}_{}_{}", KEY_RTE, p, KEY_COMPONENT, k, &rte.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                        KEY_GV_LABEL: k
                                                                    }), PropertyType::Gv);

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
                                                                let c_dst_o = self.db.create_object(VertexTypes::ComponentDst);
                                                                self.db.create_relationship(&c_o, &c_dst_o);
                                                                self.db.add_object_properties(&c_dst_o, &json!({
                                                                        KEY_GVID: format!("{}_{}_{}_{}_{}", KEY_RTE, p, KEY_COMPONENT, k, &rte.as_object().unwrap().get(PropertyType::Module.name()).unwrap().as_str().unwrap()),
                                                                        KEY_GV_LABEL: k
                                                                    }), PropertyType::Gv);

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
                        let rte_type = cfg.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap();
                        let rte = Rte::new(rte_type, self.db);
                        if let Some(r) = rte { r.init(&r_o) }
                    }
                }
                _ => {}
            }
        }

        //Rte Stages Deploy
        let rte_stage_deploy = self.add_ci_stages(&ci, &self.config.rte.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Feature Stages Deploy
        let feature_stage_deploy = self.add_ci_stages(&rte_stage_deploy.unwrap(), &self.config.features.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Eut Stages Deploy
        let eut_stage_deploy = self.add_ci_stages(&feature_stage_deploy.unwrap(), &self.config.eut.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Test Stages Deploy
        let mut test_stage_deploy = self.add_ci_stages(&eut_stage_deploy.unwrap(), &self.config.tests.ci.stages.deploy, &VertexTypes::StageDeploy);
        //Verification Stages Deploy
        let verification_stage_deploy = self.add_ci_stages(&test_stage_deploy.unwrap(), &self.config.verifications.ci.stages.deploy, &VertexTypes::StageDeploy);

        //Test and Verification single job stages
        let _rtes = self.db.get_object_neighbour_out(&eut.id, EdgeTypes::UsesRtes);
        let rtes = self.db.get_object_neighbours_with_properties_out(&_rtes.id, EdgeTypes::ProvidesRte);
        let mut _test_stages: Vec<String> = Vec::new();
        let mut _verification_stages: Vec<String> = Vec::new();

        for rte in rtes.iter() {
            let _c = self.db.get_object_neighbour_out(&rte.vertex.id, EdgeTypes::HasConnections);
            let _conns = self.db.get_object_neighbours_out(&_c.id, EdgeTypes::HasConnection);
            for conn in _conns.iter() {
                let c_src = self.db.get_object_neighbour_with_properties_out(&conn.id, EdgeTypes::HasConnectionSrc).unwrap();
                let tests = self.db.get_object_neighbours_with_properties_out(&c_src.vertex.id, EdgeTypes::Runs);
                for t in tests.iter() {
                    let t_stage_name = format!("{}-{}-{}-{}-{}",
                                               KEY_TEST,
                                               rte.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                               c_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                               &t.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                               KEY_APPLY
                    ).replace('_', "-");
                    _test_stages.push(t_stage_name);

                    //Verification stages
                    let verifications = self.db.get_object_neighbours_with_properties_out(&t.vertex.id, EdgeTypes::Needs);
                    for v in verifications.iter() {
                        let v_stage_name = format!("{}-{}-{}-{}-{}",
                                                   KEY_VERIFICATION,
                                                   rte.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                                   c_src.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                                   &v.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap(),
                                                   KEY_APPLY
                        ).replace('_', "-");
                        _verification_stages.push(v_stage_name);
                    }
                }
            }
        }

        test_stage_deploy = self.add_ci_stages(&verification_stage_deploy.unwrap(), &_test_stages, &VertexTypes::StageDeploy);
        self.add_ci_stages(&test_stage_deploy.unwrap(), &_verification_stages, &VertexTypes::StageDeploy);

        //Feature Stages Destroy
        let mut stage_destroy: Option<Vertex> = None;
        let _features = self.db.get_object_neighbour_out(&eut.id, EdgeTypes::HasFeatures);
        let features = self.db.get_object_neighbours_out(&_features.id, EdgeTypes::HasFeature);

        if !features.is_empty() {
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

    fn load_regression_config(path: &str, file: &str) -> RegressionConfig {
        info!("Loading regression configuration data...");
        let data: String = format!("{path}/{file}");
        let raw = std::fs::read_to_string(data).unwrap();
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

        let eutc = _tera.render(file, &context).unwrap();
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

        let raw = std::fs::read_to_string(String::from(&file)).unwrap();
        let cfg: Value = serde_json::from_str(&raw).unwrap();
        info!("Loading module <{module}> configuration data -> Done.");
        cfg
    }

    fn add_ci_stages(&self, ancestor: &Vertex, stages: &[String], object_type: &VertexTypes) -> Option<Vertex> {
        let mut curr = Vertex { id: Default::default(), t: Default::default() };

        for (i, stage) in stages.iter().enumerate() {
            let new = self.db.create_object(object_type.clone());
            self.db.add_object_properties(&new, &stage, PropertyType::Base);
            self.db.add_object_properties(&new, &json!({
                KEY_GVID: stage.replace('-', "_"),
                KEY_GV_LABEL: stage,
            }), PropertyType::Gv);

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
        let project = self.db.get_object_with_properties(&id);
        let project_p_base = project.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();

        let eut = self.db.get_object_neighbour_with_properties_out(&project.vertex.id, EdgeTypes::HasEut).unwrap();
        let eut_p_base = eut.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();
        let eut_p_module = eut.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap();
        let eut_name = eut_p_base.get(KEY_NAME).unwrap().as_str().unwrap().to_string();

        //Process eut provider
        let _eut_providers = self.db.get_object_neighbour_out(&eut.vertex.id, EdgeTypes::HasProviders);
        let eut_provider = self.db.get_object_neighbours_with_properties_out(&_eut_providers.id, EdgeTypes::ProvidesProvider);

        let mut eut_provider_p_base = Vec::new();
        for p in eut_provider.iter() {
            let name = p.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_NAME).unwrap().as_str().unwrap();
            eut_provider_p_base.push(String::from(name));
        }

        //Process features
        let _features = self.db.get_object_neighbour_out(&eut.vertex.id, EdgeTypes::HasFeatures);
        let features = self.db.get_object_neighbours_with_properties_out(&_features.id, EdgeTypes::HasFeature);
        let mut features_rc: Vec<FeatureRenderContext> = Vec::new();

        for feature in features.iter() {
            let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
            let f_name = feature.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_MODULE).unwrap().as_str().unwrap().to_string();
            let scripts_path = feature.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();

            //Process feature scripts
            for script in feature.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                let path = format!("{}/{}/{}/{}/{}", self.config.project.root_path, self.config.features.path, f_name, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                let contents = std::fs::read_to_string(path).expect("panic while opening feature script file");
                let ctx = ScriptFeatureRenderContext {
                    eut: eut_name.to_string(),
                    name: f_name.to_string(),
                    sites: serde_json::to_string(&feature.props.get(PropertyType::Base.index()).unwrap().value.as_object().unwrap().get(KEY_SITES).unwrap().as_array().unwrap()).unwrap(),
                    release: feature.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_RELEASE).unwrap().as_str().unwrap().to_string(),
                    provider: eut_provider_p_base.clone(),
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

            let frc = FeatureRenderContext {
                ci: feature.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_CI).unwrap().as_object().unwrap().clone(),
                job: format!("{}_{}_{}", KEY_FEATURE, &eut_name, &f_name).replace('_', "-"),
                eut: eut_name.to_string(),
                name: f_name,
                release: feature.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_RELEASE).unwrap().as_str().unwrap().to_string(),
                scripts,
            };
            features_rc.push(frc);
        }

        //Get EUT sites
        let _sites = self.db.get_object_neighbour_out(&eut.vertex.id, EdgeTypes::HasSites);
        let sites = self.db.get_object_neighbours_with_properties_out(&_sites.id, EdgeTypes::HasSite);

        //Get EUT rtes
        let _rtes = self.db.get_object_neighbour_out(&eut.vertex.id, EdgeTypes::UsesRtes);
        let rtes = self.db.get_object_neighbours_with_properties_out(&_rtes.id, EdgeTypes::ProvidesRte);

        //Process rte share data script render context
        let site_count: usize = 0;
        let mut srsd: HashMap<String, ScriptRteSitesShareDataRenderContext> = HashMap::new();

        for rte in rtes.iter() {
            let rte_type = rte.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_TYPE).unwrap().as_str().unwrap();
            let _rte = Rte::new(rte_type, self.db);
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
                            let path = format!("{}/{}/{}/{}/{}/{}/{}", self.config.project.root_path, self.config.rte.path, rte_name, scripts_path, p_name, KEY_SHARE, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
                            let contents = std::fs::read_to_string(path).expect("panic while opening feature script file");
                            let ctx = ScriptRteProviderShareRenderContext {
                                rte: rte_name.to_string(),
                                eut: eut_name.to_string(),
                                map: serde_json::to_string(&rte_to_site_map).unwrap(),
                                vars: self.config.project.vars.clone(),
                                sites: serde_json::to_string(&srsd_rc).unwrap(),
                                counter: site_count,
                                provider: p_name.to_string(),
                                project: self.config.project.name.to_string(),
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
                            job: format!("{}_{}_{}_{}", KEY_RTE, &rte_name, p_name, KEY_SHARE).replace('_', "-"),
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
            let _rte = Rte::new(rte_type, self.db);
            let mut feature_names: Vec<String> = Vec::new();

            for feature in features_rc.iter() {
                feature_names.push(feature.name.to_string());
            }

            if let Some(r) = _rte {
                r.build_conn_ctx(RteCtxParameters {
                    rte,
                    config: &self.config,
                    eut_name: eut_name.to_string(),
                    rte_name: rte_name.to_string(),
                    features: feature_names,
                    provider: active_provider,
                    rte_crcs: &mut rte_crcs,
                })
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
            let scripts_path = eut.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS_PATH).unwrap().as_str().unwrap();
            let mut scripts: Vec<HashMap<String, Vec<String>>> = Vec::new();
            for script in eut.props.get(PropertyType::Module.index()).unwrap().value.as_object().unwrap().get(KEY_SCRIPTS).unwrap().as_array().unwrap().iter() {
                let path = format!("{}/{}/{}/{}/{}", self.config.project.root_path, self.config.eut.path, eut_name, scripts_path, script.as_object().unwrap().get(KEY_FILE).unwrap().as_str().unwrap());
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
                job: format!("{}_{}_{}", KEY_EUT, &eut_name, &site_name).replace('_', "-"),
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

        let mut stages: Vec<String> = Vec::new();
        let mut deploy_stages: Vec<String> = Vec::new();
        let mut destroy_stages: Vec<String> = Vec::new();

        let project_ci = self.db.get_object_neighbour_out(&project.vertex.id, EdgeTypes::HasCi);
        let s_deploy = self.db.get_object_neighbour_with_properties_out(&project_ci.id, EdgeTypes::HasDeployStages).unwrap();
        let s_destroy = self.db.get_object_neighbour_with_properties_out(&project_ci.id, EdgeTypes::HasDestroyStages).unwrap();
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

        // error!("{:#?}", context);
        info!("Build render context -> Done.");
        context
    }

    fn get_next_stage(&self, id: &Uuid, data: &mut Vec<String>) {
        for stage in self.db.get_object_neighbours_with_properties_out(id, EdgeTypes::NextStage).iter() {
            data.push(stage.props.get(PropertyType::Base.index()).unwrap().value.as_str().unwrap().to_string());
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
}