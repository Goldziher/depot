use depot_core::config::Config;

pub fn verify(_config: Config) {
    eprintln!("error: lock verify is not implemented in this MVP");
    std::process::exit(2);
}

pub fn update(_config: Config) {
    eprintln!("error: lock update is not implemented in this MVP");
    std::process::exit(2);
}
