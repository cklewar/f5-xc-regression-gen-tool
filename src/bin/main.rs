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

use clap::Parser;

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

pub mod regression {
    use std::collections::{HashSet, HashMap};
    use std::io::{Write};
    use serde_derive::{Deserialize, Serialize};
    use serde_json::{json};
    use tera::Tera;

    const CONFIG_FILE_NAME: &str = "config.json";
    const PIPELINE_TEMPLATE_FILE_NAME: &str = ".gitlab-ci.yml.tpl";
    const SCRIPT_TYPE_APPLY: &str = "apply";
    const SCRIPT_TYPE_DESTROY: &str = "destroy";
    const SCRIPT_TYPE_ARTIFACTS: &str = "artifacts";
    const SCRIPT_TYPE_COLLECTOR_PATH: &str = "scripts";

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionCommonConfig {
        project: String,
        templates: String,
        root_path: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionCiVariablesConfig {
        name: String,
        value: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionCiArtifactsConfig {
        path: String,
        expire_in: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionCiConfig {
        tags: Vec<String>,
        image: String,
        artifacts: RegressionCiArtifactsConfig,
        variables: Vec<RegressionCiVariablesConfig>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionEutRunTestVerificationsConfig {
        name: String,
        module: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionEutRunTestsConfig {
        name: String,
        module: String,
        verifications: Vec<RegressionEutRunTestVerificationsConfig>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionEutProviderCollector {
        module: String,
        path: String,
        enable: bool,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionEutProviderConfig {
        collector: RegressionEutProviderCollector,
        tests: Vec<RegressionEutRunTestsConfig>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionEutConfig {
        name: String,
        path: String,
        stages: Vec<String>,
        provider: HashMap<String, RegressionEutProviderConfig>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionCollectorConfig {
        path: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionRteConfig {
        path: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionTests {
        path: String,
        stages: Vec<String>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionVerifications {
        path: String,
        stages: Vec<String>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct RegressionConfig {
        ci: RegressionCiConfig,
        eut: RegressionEutConfig,
        rte: RegressionRteConfig,
        collector: RegressionCollectorConfig,
        tests: RegressionTests,
        common: RegressionCommonConfig,
        verifications: RegressionVerifications,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    struct EnvironmentRteConfigScript {
        name: String,
        value: String,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    struct EnvironmentRteConfigVariables {
        name: String,
        value: String,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    struct EnvironmentRteConfigProvider {
        variables: Vec<EnvironmentRteConfigVariables>,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    struct EnvironmentRteConfig {
        name: String,
        stages: Vec<String>,
        timeout: String,
        provider: HashMap<String, EnvironmentRteConfigProvider>,
        scripts: Vec<EnvironmentRteConfigScript>,
        scripts_path: String,
    }
    
    #[derive(Deserialize, Serialize, Clone, Debug)]
    struct EnvironmentCollectorConfigRaw {
        name: String,
        timeout: String,
        script: String,
    }
    
    #[derive(Deserialize, Serialize, Clone, Debug)]
    struct EnvironmentCollectorConfig {
        enable: bool,
        name: String,
        timeout: String,
        script: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct EnvironmentTestModuleVerificationConfig {
        name: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct EnvironmentTestModuleRteConfig {
        name: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct TestModuleConfig {
        name: String,
        rte: EnvironmentTestModuleRteConfig,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct EnvironmentTestModuleAndVerificationConfig {
        name: String,
        module: String,
        rte: EnvironmentTestModuleRteConfig,
        stage: String,
        verifications: Vec<EnvironmentTestModuleVerificationConfig>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct EnvironmentProvider {
        collector: EnvironmentCollectorConfig,
        tmvc: Vec<EnvironmentTestModuleAndVerificationConfig>,
        rtes: Vec<EnvironmentRteConfig>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct EnvironmentCi {
        stages: Vec<String>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Environment {
        rc: RegressionConfig,
        ci: EnvironmentCi,
        providers: HashMap<String, EnvironmentProvider>,
    }

    #[derive(Debug)]
    struct ScriptRenderContext {
        provider: String,
        rte_name: Option<String>,
        rte_names: Option<Vec<String>>,
    }

    impl ScriptRenderContext {
        pub fn new(provider: String) -> Self {
            Self {
                provider,
                rte_name: None,
                rte_names: None,
            }
        }
    }

    impl Environment {
        pub fn new(file: String) -> Self {
            println!("Loading new regression environment...");
            let rc = RegressionConfig::load_regression_config(file);
            let mut eps: HashMap<String, EnvironmentProvider> = HashMap::new();
            let mut ci_stages: Vec<String> = Vec::new();
            let mut unique_ci_stages: HashSet<String> = HashSet::new();

            for (provider, config) in rc.eut.provider.iter() {
                let mut unique_rte: HashSet<String> = HashSet::new();
                let tmvc = rc.load_test_modules_config(&config.tests);
                let mut rtes: Vec<EnvironmentRteConfig> = Vec::new();
                let mut rte_names: Vec<String> = Vec::new();
                let mut collector: EnvironmentCollectorConfig;

                for tm in tmvc.iter() {
                    let name = String::from(&tm.rte.name);
                    if !unique_rte.contains(&name) {
                        unique_rte.insert(name.clone());
                        let rte_cfg: EnvironmentRteConfig = rc.load_rte_config(&tm.module, &tm.rte.name, &provider);
                        for stage in rte_cfg.stages.iter() {
                            if !unique_ci_stages.contains(stage) {
                                ci_stages.push(stage.clone());
                                unique_ci_stages.insert(stage.clone());
                            }
                        }
                        rte_names.push(rte_cfg.name.clone());
                        rtes.push(rte_cfg);
                    }
                }

                for rte in &mut rtes {
                    for script in &mut rte.scripts {
                        let mut ctx: ScriptRenderContext = ScriptRenderContext::new(provider.clone());
                        match script.name.as_ref() {
                            SCRIPT_TYPE_APPLY => {
                                ctx.rte_name = Option::from(rte.name.clone());
                            }
                            SCRIPT_TYPE_DESTROY => {}
                            SCRIPT_TYPE_ARTIFACTS => {
                                ctx.rte_name = Option::from(rte.name.clone());
                            }
                            _ => {
                                println!("Given script type does not match any know types")
                            }
                        }
                        script.value = render_script(&ctx, &script.value);
                    }
                }

                if config.collector.enable {
                    collector = rc.load_collector_config(&provider, &config.collector);
                    let mut ctx: ScriptRenderContext = ScriptRenderContext::new(provider.clone());
                    ctx.rte_names = Option::from(rte_names.clone());
                    collector.script = render_script(&ctx, &collector.script);
                } else {
                    collector = EnvironmentCollectorConfig {
                        enable: false,
                        name: "".to_string(),
                        timeout: "".to_string(),
                        script: "".to_string(),
                    };
                }

                let _ep = EnvironmentProvider { collector, tmvc, rtes };
                eps.insert(provider.clone(), _ep);
            }

            for stage in rc.eut.stages.iter() {
                ci_stages.push(stage.clone());
            }
            for stage in rc.tests.stages.iter() {
                ci_stages.push(stage.clone())
            }
            for (_k, v) in &mut eps.iter() {
                for tm in v.tmvc.iter() {
                    ci_stages.push(format!("regression-test-{}", tm.name.clone()));
                }
            }
            for stage in rc.verifications.stages.iter() {
                ci_stages.push(stage.clone())
            }

            let eci = EnvironmentCi {
                stages: ci_stages
            };
            let renv = Environment { rc, ci: eci, providers: eps };
            renv
        }

        pub fn render(&self) -> String {
            println!("Render regression pipeline file first step...");
            let mut _tera = Tera::new(&self.rc.common.templates).unwrap();
            let mut context = tera::Context::new();
            context.insert("rc", &self.rc);
            context.insert("ci", &self.ci);
            context.insert("providers", &self.providers);
            let rendered = _tera.render(PIPELINE_TEMPLATE_FILE_NAME, &context).unwrap();
            println!("Render regression pipeline file first step -> Done.");
            rendered
        }

        pub fn to_json(&self) -> String {
            let j = json!({
                "rc": &self.rc,
                "ci": &self.ci,
                "providers": &self.providers,
            });
            j.to_string()
        }

        pub fn to_file(&self, file: String) {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(file)
                .expect("Couldn't open file");

            f.write_all(&self.render().as_bytes()).expect("panic while writing to file");
        }
    }

    impl RegressionConfig {
        fn load_regression_config(file: String) -> RegressionConfig {
            println!("Loading regression configuration data...");
            let data: String = String::from(file);
            let raw = std::fs::read_to_string(&data).unwrap();
            let cfg = serde_json::from_str::<RegressionConfig>(&raw).unwrap();
            println!("Loading regression configuration data -> Done.");

            println!("Render regression configuration file...");
            let mut _tera = Tera::new("../../regression/configurations/*").unwrap();
            let mut context = tera::Context::new();
            context.insert("eut", &cfg.eut);
            context.insert("rte", &cfg.rte);
            context.insert("common", &cfg.common);
            context.insert("collector", &cfg.collector);
            context.insert("tests", &cfg.tests);
            context.insert("verifications", &cfg.verifications);
            let eutc = _tera.render("regression.json", &context).unwrap();
            println!("Render regression configuration file -> Done.");

            println!("Loading regression configuration data...");
            let cfg = serde_json::from_str::<RegressionConfig>(&eutc).unwrap();
            println!("Loading regression configuration data -> Done.");

            cfg
        }

        fn load_test_modules_config(&self, tests: &Vec<RegressionEutRunTestsConfig>) -> Vec<EnvironmentTestModuleAndVerificationConfig> {
            println!("Loading regression test modules configuration...");
            let mut modules: Vec<EnvironmentTestModuleAndVerificationConfig> = Vec::new();

            for test in tests.iter() {
                let mut vmc: Vec<EnvironmentTestModuleVerificationConfig> = Vec::new();

                for verification in test.verifications.iter() {
                    let raw = std::fs::read_to_string(format!("{}/{}/{}/{}", self.common.root_path, self.verifications.path, verification.module, CONFIG_FILE_NAME)).unwrap();
                    let cfg = serde_json::from_str::<EnvironmentTestModuleVerificationConfig>(&raw).unwrap();
                    vmc.push(cfg);
                }

                let raw = std::fs::read_to_string(format!("{}/{}/{}/{}", self.common.root_path, self.tests.path, test.module, CONFIG_FILE_NAME)).unwrap();
                let cfg = serde_json::from_str::<TestModuleConfig>(&raw).unwrap();

                let tmvc = EnvironmentTestModuleAndVerificationConfig {
                    rte: cfg.rte,
                    name: test.name.clone(),
                    stage: format!("regression-test-{}", test.name),
                    module: cfg.name,
                    verifications: vmc,
                };

                modules.push(tmvc);
            }
            println!("Loading regression test modules configuration -> Done.");
            modules
        }

        fn load_rte_config(&self, tm_name: &String, rte_name: &String, provider: &String) -> EnvironmentRteConfig {
            println!("Loading test module <{}> specific regression test environment configuration...", tm_name);
            let file = format!("{}/{}/{}/{}", self.common.root_path, self.rte.path, rte_name, CONFIG_FILE_NAME);
            let raw = std::fs::read_to_string(file).expect("panic while opening rte config file");
            let mut cfg = serde_json::from_str::<EnvironmentRteConfig>(&raw).unwrap();
            let mut scripts: Vec<EnvironmentRteConfigScript> = Vec::new();

            for script in cfg.scripts.iter() {
                let file = format!("{}/{}/{}/{}/{}/{}", self.common.root_path, self.rte.path, rte_name, cfg.scripts_path, provider, script.value);
                let contents = std::fs::read_to_string(file).expect("panic while opening rte apply.script file");
                let rcs = EnvironmentRteConfigScript {
                    name: script.name.clone(),
                    value: contents,
                };
                scripts.push(rcs);
            }
            cfg.scripts = scripts;
            println!("Loading test module <{}> specific regression test environment configuration -> Done.", tm_name);
            cfg
        }

        fn load_collector_config(&self, provider: &String, collector: &RegressionEutProviderCollector) -> EnvironmentCollectorConfig {
            println!("Loading provider <{}> collector module <{}> configuration...", provider, collector.module);
            let file = format!("{}/{}/{}/{}/{}", self.common.root_path, self.collector.path, collector.path, collector.module, CONFIG_FILE_NAME);
            let raw = std::fs::read_to_string(file).expect("panic while opening collector config file");
            let cfg_raw = serde_json::from_str::<EnvironmentCollectorConfigRaw>(&raw).unwrap();
            let script = format!("{}/{}/{}/{}/{}/{}", self.common.root_path, self.collector.path, collector.path, collector.module, SCRIPT_TYPE_COLLECTOR_PATH, cfg_raw.script);
            let contents = std::fs::read_to_string(&script).expect("panic while opening collector collector.script file");
            let cfg=  EnvironmentCollectorConfig {
                enable: collector.enable,
                name: cfg_raw.name,
                timeout: cfg_raw.timeout,
                script: contents
            };
            println!("Loading provider <{}> collector module <{}> configuration -> Done.", provider, collector.module);
            cfg
        }
    }

    fn render_script(context: &ScriptRenderContext, input: &String) -> String {
        println!("Render regression pipeline file script section...");
        let mut ctx = tera::Context::new();
        ctx.insert("provider", &context.provider);
        ctx.insert("rte_name", &context.rte_name);
        ctx.insert("rte_names", &context.rte_names);
        let rendered = Tera::one_off(input, &ctx, true).unwrap();
        println!("Render regression pipeline file script section -> Done.");
        rendered
    }
}

fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();
    let e = regression::Environment::new(cli.config);

    if cli.write {
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
    }

    Ok(())
}