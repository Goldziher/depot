use depot_core::config::Config;

pub fn run(_config: Config) {
    eprintln!("error: sync command is not implemented in this MVP");
    std::process::exit(2);
}
