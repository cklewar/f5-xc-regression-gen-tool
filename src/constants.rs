pub const CONFIG_FILE_NAME: &str = "config.json";
pub const CONFIG_FILE_PATH: &str = "regression/config";

// KEYS
pub const KEY_CI: &str = "ci";
pub const KEY_EUT: &str = "eut";
pub const KEY_RTE: &str = "rte";
pub const KEY_SRC: &str = "src";
pub const KEY_DST: &str = "dst";
pub const KEY_FILE: &str = "file";
pub const KEY_RTES: &str = "rtes";
pub const KEY_TYPE: &str = "type";
pub const KEY_TEST: &str = "test";
pub const KEY_TESTS: &str = "tests";
pub const KEY_GVID: &str = "id";
pub const KEY_NAME: &str = "name";
pub const KEY_APPLY: &str = "apply";
pub const KEY_SITES: &str = "sites";
pub const KEY_DEPLOY: &str = "deploy";
pub const KEY_SHARE: &str = "share";
pub const KEY_COUNT: &str = "count";
pub const KEY_CONFIG: &str = "config";
pub const KEY_STAGES: &str = "stages";
pub const KEY_MODULE: &str = "module";
pub const KEY_SCRIPT: &str = "script";
pub const KEY_RELEASE: &str = "release";
pub const KEY_ID_PATH: &str = "id_path";
pub const KEY_SCRIPTS: &str = "scripts";
pub const KEY_ACTIONS: &str = "actions";
pub const KEY_SOURCE: &str = "source";
pub const KEY_PROJECT: &str = "project";
pub const KEY_FEATURE: &str = "feature";
pub const KEY_GV_LABEL: &str = "label";
pub const KEY_FEATURES: &str = "features";
pub const KEY_PROVIDER: &str = "provider";
pub const KEY_PROVIDERS: &str = "providers";
pub const KEY_DASHBOARD: &str = "dashboard";
pub const KEY_COMPONENT: &str = "component";
pub const KEY_COMPONENTS: &str = "components";
pub const KEY_CONNECTION: &str = "connection";
pub const KEY_CONNECTIONS: &str = "connections";
pub const KEY_VERIFICATION: &str = "verification";
pub const KEY_SCRIPTS_PATH: &str = "scripts_path";
pub const KEY_VERIFICATIONS: &str = "verifications";

// miscellaneous
pub const PIPELINE_FILE_NAME: &str = ".gitlab-ci.yml";
pub const PIPELINE_TEMPLATE_FILE_NAME: &str = ".gitlab-ci.yml.tpl";

pub const ENTRY_FILE_NAME: &str = "entry.md";
pub const ACTIONS_FILE_NAME: &str = "actions.json";

pub const PROPERTY_TYPE_GV: &str = "gv";
pub const PROPERTY_TYPE_BASE: &str = "base";
pub const PROPERTY_TYPE_MODULE: &str = "module";

// Key order
pub const EUT_KEY_ORDER: &[&str] = &["ci", "provider", "sites", "features", "name", "release", "rtes", "scripts", "scripts_path"];

//Objects types
pub const VERTEX_TYPE_CI: &str = "ci";
pub const VERTEX_TYPE_EUT: &str = "eut";
pub const VERTEX_TYPE_RTE: &str = "rte";
pub const VERTEX_TYPE_RTES: &str = "rtes";
pub const VERTEX_TYPE_TEST: &str = "test";
pub const VERTEX_TYPE_NONE: &str = "none";
pub const VERTEX_TYPE_SITE: &str = "site";
pub const VERTEX_TYPE_SITES: &str = "sites";
pub const VERTEX_TYPE_SHARE: &str = "share";
pub const VERTEX_TYPE_SCRIPT: &str = "script";
pub const VERTEX_TYPE_SCRIPTS: &str = "scripts";
pub const VERTEX_TYPE_PROJECT: &str = "project";
pub const VERTEX_TYPE_FEATURE: &str = "feature";
pub const VERTEX_TYPE_FEATURES: &str = "features";
pub const VERTEX_TYPE_COLLECTOR: &str = "collector";
pub const VERTEX_TYPE_PROVIDERS: &str = "providers";
pub const VERTEX_TYPE_DASHBOARD: &str = "dashboard";
pub const VERTEX_TYPE_COMPONENTS: &str = "components";
pub const VERTEX_TYPE_CONNECTION: &str = "connection";
pub const VERTEX_TYPE_CONNECTIONS: &str = "connections";
pub const VERTEX_TYPE_VERIFICATION: &str = "verification";
pub const VERTEX_TYPE_EUT_PROVIDER: &str = "eut_provider";
pub const VERTEX_TYPE_RTE_PROVIDER: &str = "rte_provider";
pub const VERTEX_TYPE_STAGE_DEPLOY: &str = "deploy";
pub const VERTEX_TYPE_STAGE_DESTROY: &str = "stage_destroy";
pub const VERTEX_TYPE_COMPONENT_SRC: &str = "component_src";
pub const VERTEX_TYPE_COMPONENT_DST: &str = "component_dst";
pub const VERTEX_TYPE_CONNECTION_SRC: &str = "connection_src";
pub const VERTEX_TYPE_CONNECTION_DST: &str = "connection_dst";
pub const VERTEX_TYPE_DASHBOARD_PROVIDER: &str = "dashboard_provider";
// Rel type
pub const EDGE_TYPE_HAS: &str = "has";
pub const EDGE_TYPE_RUNS: &str = "runs";
pub const EDGE_TYPE_NEEDS: &str = "needs";
pub const EDGE_TYPE_HAS_CI: &str = "has_ci";
pub const EDGE_TYPE_HAS_EUT: &str = "has_eut";
pub const EDGE_TYPE_HAS_SITE: &str = "has_site";
pub const EDGE_TYPE_HAS_SITES: &str = "has_sites";
pub const EDGE_TYPE_NEEDS_SHARE: &str = "needs_share";
pub const EDGE_TYPE_USES_RTES: &str = "uses_rtes";
pub const EDGE_TYPE_NEXT_STAGE: &str = "next_stage";
pub const EDGE_TYPE_REFERS_SITE: &str = "refers_site";
pub const EDGE_TYPE_HAS_FEATURE: &str = "has_feature";
pub const EDGE_TYPE_HAS_FEATURES: &str = "has_features";
pub const EDGE_TYPE_PROVIDES_RTE: &str = "provides_rte";
pub const EDGE_TYPE_HAS_PROVIDERS: &str = "has_providers";
pub const EDGE_TYPE_USES_PROVIDER: &str = "uses_provider";
pub const EDGE_TYPE_NEEDS_PROVIDER: &str = "needs_provider";
pub const EDGE_TYPE_HAS_COMPONENTS: &str = "has_components";
pub const EDGE_TYPE_HAS_CONNECTION: &str = "has_connection";
pub const EDGE_TYPE_HAS_CONNECTIONS: &str = "has_connections";
pub const EDGE_TYPE_SITE_REFERS_RTE: &str = "site_refers_rte";
pub const EDGE_TYPE_PROVIDES_PROVIDER: &str = "provides_provider";
pub const EDGE_TYPE_HAS_COMPONENT_SRC: &str = "has_component_src";
pub const EDGE_TYPE_HAS_COMPONENT_DST: &str = "has_component_dst";
pub const EDGE_TYPE_HAS_CONNECTION_SRC: &str = "has_connection_src";
pub const EDGE_TYPE_HAS_CONNECTION_DST: &str = "has_connection_dst";
pub const EDGE_TYPE_HAS_DEPLOY_STAGES: &str = "has_deploy_stages";
pub const EDGE_TYPE_HAS_DESTROY_STAGES: &str = "has_destroy_stages";
pub const EDGE_TYPE_FEATURE_REFERS_SITE: &str = "feature_refers_site";

//RTE TYPES

pub const RTE_TYPE_A: &str = "rte_type_a";
pub const RTE_TYPE_B: &str = "rte_type_b";