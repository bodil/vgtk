#![recursion_limit = "4096"]

mod about;
mod app;
mod items;
mod radio;

use gio::ApplicationFlags;
use vgtk::go;

use app::Model;

fn main() {
    pretty_env_logger::init();
    std::process::exit(go::<Model>("camp.lol.todomvc", ApplicationFlags::empty()));
}
