#![recursion_limit = "4096"]

mod about;
mod app;
mod items;
mod radio;

use vgtk::run;

use app::Model;

fn main() {
    pretty_env_logger::init();
    std::process::exit(run::<Model>());
}
