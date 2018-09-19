extern crate gio;
extern crate glib;
extern crate gtk;
extern crate im;

#[macro_use]
extern crate vgtk;

use gio::ApplicationFlags;
use gtk::prelude::*;
use gtk::{Box, Entry, Grid, Label, ListBox, ListBoxRow, Window};
use vgtk::{Application, Component, VObject, View};

use im::Vector;

#[derive(Clone, Default)]
struct Item {
    label: String,
    done: bool,
}

impl Item {
    fn new(label: String) -> Self {
        Item { label, done: false }
    }
}

#[derive(Default)]
struct Model {
    items: Vector<Item>,
}

enum Msg {
    NoOp,
    Add { item: String },
    Remove { index: usize },
    Toggle { index: usize },
}

impl Component for Model {
    type Message = Msg;

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::NoOp => return false,
            Msg::Add { item } => {
                self.items.push_front(Item::new(item));
            }
            Msg::Remove { index } => {
                self.items.remove(index);
            }
            Msg::Toggle { index } => self.items[index].done = !self.items[index].done,
        }
        true
    }
}

impl View<Model> for Model {
    fn view(&self) -> VObject<Model> {
        gtk!{
            <Window title="Updog",>
                <Grid column_spacing=10, row_spacing=10,>
                    <Entry placeholder_text="What needs to be done?",
                           left_attach=0, top_attach=0,
                           on activate=|e| {
                               let label = e.source.downcast::<Entry>().unwrap().get_text().unwrap_or_default();
                               println!("activated entry: {:?}", label);
                               Msg::Add {
                                   item: label
                               }
                           }, />
                    <ListBox left_attach=0, top_attach=1,>
                        { for self.items.iter().enumerate().map(|(index, item)| render_item(index,item)) }
                    </ListBox>
                </Grid>
            </Window>
        }
    }
}

fn render_item(_index: usize, item: &Item) -> VObject<Model> {
    gtk!{
        <ListBoxRow>
            <Box>
                <Label label=item.label, />
            </Box>
        </ListBoxRow>
    }
}

fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    ::std::process::exit(Application::<Model>::run(
        "camp.lol.updog",
        ApplicationFlags::empty(),
        &args,
    ));
}
