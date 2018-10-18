#![recursion_limit = "16384"]
#![allow(clippy::cyclomatic_complexity)]

extern crate gio;
extern crate glib;
extern crate gtk;
#[macro_use]
extern crate im;

#[macro_use]
extern crate vgtk;

use gio::ApplicationFlags;
use gtk::prelude::*;
use gtk::*;
use vgtk::{Application, Component, VItem, View};

use im::Vector;

#[derive(Clone, Debug, Default)]
struct Radio {
    labels: Vec<String>,
    active: usize,
}

enum RadioMsg {
    Selected(usize),
}

impl Component for Radio {
    type Message = RadioMsg;
    type Properties = Radio;

    fn create(props: Self::Properties) -> Self {
        props
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            RadioMsg::Selected(selected) => self.active = selected,
        }
        true
    }
}

impl View<Radio> for Radio {
    fn view(&self) -> VItem<Radio> {
        gtk!{
            <Box center=true, orientation=Orientation::Horizontal, spacing=10, expand=true,>
                { for self.labels.iter().enumerate().map(|(index, label)| {
                    gtk!{
                        <ToggleButton label=label, active=index == self.active,
                                      on toggled=|_| RadioMsg::Selected(index),/>
                    }
                }) }
            </Box>
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Filter {
    All,
    Active,
    Completed,
}

#[derive(Clone, Default, Debug)]
struct Item {
    label: String,
    done: bool,
}

impl Item {
    fn new(label: String) -> Self {
        Item { label, done: false }
    }
}

#[derive(Clone, Debug)]
struct Model {
    items: Vector<Item>,
    filter: Filter,
}

impl Default for Model {
    fn default() -> Self {
        Model {
            items: vector![Item::new("foo".to_string()), Item::new("bar".to_string())],
            filter: Filter::All,
        }
    }
}

impl Model {
    fn filter(&self, filter: Filter) -> impl Iterator<Item = &Item> {
        self.items.iter().filter(move |item| match filter {
            Filter::All => true,
            Filter::Active => !item.done,
            Filter::Completed => item.done,
        })
    }

    fn left_label(&self) -> String {
        let left = self.filter(Filter::Active).count();
        match left {
            1 => String::from("1 item left"),
            left => format!("{} items left", left),
        }
    }
}

enum Msg {
    Add { item: String },
    Remove { index: usize },
    Toggle { index: usize },
    Filter { filter: Filter },
    ToggleAll,
    ClearCompleted,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn update(&mut self, msg: Self::Message) -> bool {
        let left = self.filter(Filter::Active).count();
        match msg {
            Msg::Add { item } => {
                self.items.push_back(Item::new(item));
            }
            Msg::Remove { index } => {
                self.items.remove(index);
            }
            Msg::Toggle { index } => self.items[index].done = !self.items[index].done,
            Msg::Filter { filter } => self.filter = filter,
            Msg::ToggleAll if left > 0 => self.items.iter_mut().for_each(|item| item.done = true),
            Msg::ToggleAll => self.items.iter_mut().for_each(|item| item.done = false),
            Msg::ClearCompleted => self.items.retain(|item| !item.done),
        }
        true
    }
}

impl View<Model> for Model {
    fn view(&self) -> VItem<Model> {
        gtk!{
            <Window default_width=800, default_height=480, border_width=20u32,>
                <HeaderBar title="TodoMVC", subtitle="wtf do we do now",
                           show_close_button=true, />
                <Box spacing=10, orientation=Orientation::Vertical,>
                    <Box spacing=10, orientation=Orientation::Horizontal, expand=false,>
                        <Button image="edit-select-all",
                                always_show_image=true, on clicked=|_| Msg::ToggleAll,/>
                        <Entry placeholder_text="What needs to be done?",
                               expand=true, fill=true,
                               on activate=|e| {
                                   let entry: Entry = e.source.downcast().unwrap();
                                   let label = entry.get_text().unwrap_or_default();
                                   entry.select_region(0, label.len() as i32);
                                   Msg::Add {
                                       item: label
                                   }
                               }, />
                    </Box>
                    <ScrolledWindow expand=true, fill=true,>
                        <ListBox selection_mode=SelectionMode::None,>
                            { for self.filter(self.filter).enumerate().map(|(index, item)| render_item(index, item)) }
                        </ListBox>
                    </ScrolledWindow>
                    <Box spacing=10, orientation=Orientation::Horizontal, expand=false,>
                        <Label label=self.left_label(),/>
                        // <Radio: labels=["All", "Active", "Completed"],/>
                        <Box center=true, orientation=Orientation::Horizontal, spacing=10, expand=true,>
                            <ToggleButton label="All", active=self.filter == Filter::All,
                                          on toggled=|_| Msg::Filter { filter:Filter::All },/>
                            <ToggleButton label="Active", active=self.filter == Filter::Active,
                                          on toggled=|_| Msg::Filter { filter:Filter::Active },/>
                            <ToggleButton label="Completed", active=self.filter == Filter::Completed,
                                          on toggled=|_| Msg::Filter { filter:Filter::Completed },/>
                        </Box>
                        {
                            self.filter(Filter::Completed).count() > 0 => gtk!{
                                <Button label="Clear completed", pack_type=PackType::End,
                                        on clicked=|_| Msg::ClearCompleted,/>
                            }
                        }
                    </Box>
                </Box>
            </Window>
        }
    }
}

fn render_item(index: usize, item: &Item) -> VItem<Model> {
    let label = if item.done {
        format!(
            "<span strikethrough=\"true\" alpha=\"50%\">{}</span>",
            item.label
        )
    } else {
        item.label.clone()
    };
    gtk!{
        <ListBoxRow>
            <Box spacing=10, orientation=Orientation::Horizontal,>
                <CheckButton active=item.done, on toggled=|_| Msg::Toggle { index },/>
                <Label label=label, use_markup=true, fill=true,/>
                <Button pack_type=PackType::End, relief=ReliefStyle::None,
                        always_show_image=true, image="edit-delete",
                        on clicked=|_| Msg::Remove { index },/>
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
