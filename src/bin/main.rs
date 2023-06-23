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
    use std::io::Write;
    use serde_derive::{Deserialize, Serialize};
    use serde_json::json;
    use tera::Tera;

    const CONFIG_FILE_NAME: &str = "config.json";
    const PIPELINE_TEMPLATE_FILE_NAME: &str = ".gitlab-ci.yml.tpl";

    #[derive(Deserialize, Serialize, Debug)]
    struct RegressionCommonConfig {
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
    struct RegressionEutProviderConfig {
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
        tests: RegressionTests,
        common: RegressionCommonConfig,
        verifications: RegressionVerifications,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RteConfigScripts {
        name: String,
        value: Vec<String>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct RteConfig {
        name: String,
        stages: Vec<String>,
        scripts: Vec<RteConfigScripts>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct TestModuleVerificationConfig {
        name: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct TestModuleRteConfig {
        name: String,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct TestModuleConfig {
        name: String,
        rte: TestModuleRteConfig,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct TestModuleAndVerificationConfig {
        name: String,
        module: String,
        rte: TestModuleRteConfig,
        stage: String,
        verifications: Vec<TestModuleVerificationConfig>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct EnvironmentProvider {
        tmvc: Vec<TestModuleAndVerificationConfig>,
        rtes: Vec<RteConfig>,
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

    impl Environment {
        pub fn render(&self) -> String {
            println!("Render regression pipeline file...");
            let mut _tera = Tera::new(&self.rc.common.templates).unwrap();
            let mut context = tera::Context::new();
            context.insert("rc", &self.rc);
            context.insert("ci", &self.ci);
            context.insert("providers", &self.providers);
            let rendered = _tera.render(PIPELINE_TEMPLATE_FILE_NAME, &context).unwrap();
            println!("Render regression pipeline file -> Done.");
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
            let eutc = _tera.render("regression.json", &context).unwrap();
            println!("Render regression configuration file -> Done.");

            println!("Loading regression configuration data...");
            let cfg = serde_json::from_str::<RegressionConfig>(&eutc).unwrap();
            println!("Loading regression configuration data -> Done.");

            cfg
        }

        fn load_test_modules_config(&self, tests: &Vec<RegressionEutRunTestsConfig>) -> Vec<TestModuleAndVerificationConfig> {
            println!("Loading regression test modules configuration...");
            let mut modules: Vec<TestModuleAndVerificationConfig> = Vec::new();

            for test in tests.iter() {
                let mut vmc: Vec<TestModuleVerificationConfig> = Vec::new();

                for verification in test.verifications.iter() {
                    let raw = std::fs::read_to_string(format!("{}/{}/{}/{}", self.common.root_path, self.verifications.path, verification.module, CONFIG_FILE_NAME)).unwrap();
                    let cfg = serde_json::from_str::<TestModuleVerificationConfig>(&raw).unwrap();
                    vmc.push(cfg);
                }

                let raw = std::fs::read_to_string(format!("{}/{}/{}/{}", self.common.root_path, self.tests.path, test.module, CONFIG_FILE_NAME)).unwrap();
                let cfg = serde_json::from_str::<TestModuleConfig>(&raw).unwrap();

                let tmvc = TestModuleAndVerificationConfig {
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

        fn load_rte_config(&self, tm_name: &String, rte_name: &String) -> RteConfig {
            println!("Loading test module <{}> specific regression test environment configuration...", tm_name);
            let rte_cfg = format!("{}/{}/{}/{}", self.common.root_path, self.rte.path, rte_name, CONFIG_FILE_NAME);
            let raw = std::fs::read_to_string(rte_cfg).unwrap();
            let cfg = serde_json::from_str::<RteConfig>(&raw).unwrap();
            println!("Loading test module <{}> specific regression test environment configuration -> Done.", tm_name);
            cfg
        }
    }

    pub fn new(file: String) -> Environment {
        println!("Loading new regression environment...");
        let rc = RegressionConfig::load_regression_config(file);
        let mut eps: HashMap<String, EnvironmentProvider> = HashMap::new();
        let mut ci_stages: Vec<String> = Vec::new();
        let mut unique_ci_stages: HashSet<String> = HashSet::new();

        for (provider, config) in rc.eut.provider.iter() {
            let mut unique_rte: HashSet<String> = HashSet::new();
            let tmvc = rc.load_test_modules_config(&config.tests);
            let mut rtes: Vec<RteConfig> = Vec::new();

            for tm in tmvc.iter() {
                let name = String::from(&tm.rte.name);
                if !unique_rte.contains(&name) {
                    unique_rte.insert(name.clone());
                    let rte_cfg: RteConfig = rc.load_rte_config(&tm.module, &tm.rte.name);
                    for stage in rte_cfg.stages.iter() {
                        if !unique_ci_stages.contains(stage) {
                            ci_stages.push(stage.clone());
                            unique_ci_stages.insert(stage.clone());
                        }
                    }
                    rtes.push(rte_cfg);
                }
            }
            let _ep = EnvironmentProvider { tmvc, rtes };
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

        let eci = EnvironmentCi { stages: ci_stages };
        let renv = Environment { rc, ci: eci, providers: eps };
        renv
    }
}

fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();
    let e = regression::new(cli.config);

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