use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let m = App::from(yaml).get_matches();
    eprintln!("m = {:#?}", m);
}
