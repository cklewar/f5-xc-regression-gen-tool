/*!
F5 XC regression test CI pipeline file generator.
Provides command line tool to generate Gitlab CI pipeline file.

Consumes input from regression configuration file provided as command line argument.
Template file relays on tool provided data structure to render stage, job or variables sections.
Tool supports direct rendering of given template file or generates JSON output which could be
used as input for another program or workflow.
 */

use clap::Parser;
use log::{error, info};

use sense8_ci_generator::constants::{ACTIONS_FILE_NAME, ENTRY_FILE_NAME, PIPELINE_FILE_NAME};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Regression configuration file
    #[arg(long)]
    root_path: String,
    #[arg(long)]
    config_file: String,
    /// Regression CI template file
    #[arg(long)]
    template: String,
    /// Write CI pipeline file
    #[arg(long)]
    write_ci: bool,
    ci_file_path: String,
    /// Export data to json file
    #[arg(long)]
    write_json: bool,
    /// Render CI pipeline file
    #[arg(short, long)]
    render_ci: bool,
    /// Write to GraphViz file
    #[arg(long)]
    write_gv: bool,
    /// Generate action names
    #[arg(long)]
    gen_actions: bool,
    entry_file_path: Option<String>,
    #[arg(long)]
    gen_actions_json: bool,
    json_file_path: Option<String>,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let db = sense8_ci_generator::db::Db::new();
    let r = sense8_ci_generator::Regression::new(&db, &cli.root_path, &cli.config_file, &cli.template);
    let p = r.init();

    let ctx = r.build_context(p);

    if cli.write_ci {
        r.to_file(&r.render(&ctx), cli.ci_file_path.as_str(), PIPELINE_FILE_NAME);
    }
    if cli.write_json {
        r.to_json();
        info!("{}", r.to_json());
    }
    if cli.render_ci {
        info!("{}", r.render(&ctx));
    }
    if cli.write_gv {
        r.to_file(&r.to_gv(), "./out", &"graph.gv");
    }
    if cli.gen_actions_json {
        let a = r.render_actions_json_file(&ctx);
        match a {
            Ok(data) => {
                error!("Render actions json -> Done");
                r.to_file(&data, cli.json_file_path.unwrap().as_str(), ACTIONS_FILE_NAME);
            }
            Err(err) => {
                error!("ERR: {}", err)
            }
        }
    }
    if cli.gen_actions {
        let a = r.render_entry_page(&ctx);
        match a {
            Ok(data) => {
                error!("Render entry page -> Done");
                r.to_file(&data, cli.entry_file_path.unwrap().as_str(), ENTRY_FILE_NAME);
            }
            Err(err) => {
                error!("ERR: {}", err)
            }
        }
    }
}