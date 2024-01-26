/*!
F5 XC regression test CI pipeline file generator.
Provides command line tool to generate Gitlab CI pipeline file.

Consumes input from regression configuration file provided as command line argument.
Template file relays on tool provided data structure to render stage, job or variables sections.
Tool supports direct rendering of given template file or generates JSON output which could be
used as input for another program or workflow.
 */

use clap::Parser;
use log::info;

use sense8_ci_generator::constants::PIPELINE_FILE_NAME;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Regression configuration file
    #[arg(long)]
    root_path: String,
    #[arg(long)]
    config_file: String,
    /// Write CI pipeline file
    #[arg(long)]
    write_ci: bool,
    ci_file_path: String,
    /// Export data to json file
    #[arg(long)]
    write_json: bool,
    /// Render CI pipline file
    #[arg(short, long)]
    render_ci: bool,
    /// Write to GraphViz file
    #[arg(long)]
    write_gv: bool,
    /// Generate action names
    #[arg(long)]
    gen_actions: bool,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let db = sense8_ci_generator::db::Db::new();
    let r = sense8_ci_generator::Regression::new(&db, &cli.root_path, &cli.config_file);
    let p = r.init();

    if cli.write_ci {
        r.to_file(&r.render(&r.build_context(p)), cli.ci_file_path.as_str(), PIPELINE_FILE_NAME);
    }
    if cli.write_json {
        r.to_json();
        info!("{}", r.to_json());
    }
    if cli.render_ci {
        info!("{}", r.render(&r.build_context(p)));
    }
    if cli.write_gv {
        r.to_file(&r.to_gv(), "./out", &"graph.gv");
    }
    if cli.gen_actions {
        let a = r.get_action_names("./out/softbank/.gitlab-ci.yml");
        match a {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}