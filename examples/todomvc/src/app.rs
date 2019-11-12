use std::fmt::Debug;

use gio::{ActionExt, ApplicationFlags, SimpleAction};
use gtk::prelude::*;
use gtk::*;
use vgtk::{ext::*, gtk, Component, UpdateAction, VNode};

use log::{debug, error};
use strum_macros::{Display, EnumIter};

use crate::about::AboutDialog;
use crate::items::{Item, Items};
use crate::radio::Radio;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Display, EnumIter)]
#[repr(u32)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl Default for Filter {
    fn default() -> Self {
        Filter::All
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    items: Items,
    filter: Filter,
    clean: bool,
}

impl Default for Model {
    fn default() -> Self {
        Model {
            items: Items::default(),
            filter: Filter::All,
            clean: true,
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

#[derive(Clone, Debug)]
pub enum Msg {
    Add { item: String },
    Remove { index: usize },
    Toggle { index: usize },
    Filter { filter: Filter },
    ToggleAll,
    ClearCompleted,
    Exit,
    MenuOpen,
    MenuSave,
    MenuSaveAs,
    MenuAbout,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn update(&mut self, msg: Self::Message) -> UpdateAction {
        let left = self.filter(Filter::Active).count();
        match msg {
            Msg::Add { item } => {
                self.items.push(Item::new(item));
                self.clean = false;
            }
            Msg::Remove { index } => {
                self.items.remove(index);
                self.clean = false;
            }
            Msg::Toggle { index } => {
                self.items[index].done = !self.items[index].done;
                self.clean = false;
            }
            Msg::Filter { filter } => {
                self.filter = filter;
            }
            Msg::ToggleAll if left > 0 => {
                self.items.iter_mut().for_each(|item| item.done = true);
                self.clean = false;
            }
            Msg::ToggleAll => {
                self.items.iter_mut().for_each(|item| item.done = false);
                self.clean = false;
            }
            Msg::ClearCompleted => {
                self.items.retain(|item| !item.done);
                self.clean = false;
            }
            Msg::Exit => {
                vgtk::quit();
                return UpdateAction::None;
            }
            Msg::MenuOpen => {
                if open(self) {
                    self.clean = true;
                }
            }
            Msg::MenuSave => {
                let path = self
                    .items
                    .get_path()
                    .expect("document has no file path but save menu item was active!")
                    .to_owned();
                if let Err(err) = self.items.write_to(path) {
                    error!("I/O error when saving file: {:?}", err);
                } else {
                    self.clean = true;
                }
            }
            Msg::MenuSaveAs => {
                if save_as(self) {
                    self.clean = true;
                }
            }
            Msg::MenuAbout => {
                AboutDialog::run();
                return UpdateAction::None;
            }
        }
        UpdateAction::Render
    }

    fn view(&self) -> VNode<Model> {
        let title = if let Some(name) = self
            .items
            .get_path()
            .and_then(|p| p.file_name())
            .and_then(|p| p.to_str())
        {
            name
        } else {
            "Untitled todo list"
        };
        let clean = if self.clean { "" } else { " *" };

        use vgtk::menu;
        let main_menu = menu()
            .section(menu().item("Open...", "win.open"))
            .section(
                menu()
                    .item("Save", "win.save")
                    .item("Save as...", "win.save-as"),
            )
            .section(menu().item("About...", "app.about"))
            .section(menu().item("Quit", "app.quit"))
            .build();

        gtk! {
            <Application::new_unwrap(Some("camp.lol.todomvc"), ApplicationFlags::empty())>

                <SimpleAction::new("quit", None) Application::accels=["<Ctrl>q"].as_ref() enabled=true
                        on activate=|a, _| Msg::Exit/>
                <SimpleAction::new("about", None) enabled=true on activate=|_, _| Msg::MenuAbout/>

                <ApplicationWindow default_width=800 default_height=480 border_width=20 on destroy=|_| Msg::Exit>

                    <SimpleAction::new("open", None) ApplicationWindow::accels=["<Ctrl>o"].as_ref()
                                                     enabled=true on activate=|a, _| Msg::MenuOpen/>
                    <SimpleAction::new("save", None) ApplicationWindow::accels=["<Ctrl>s"].as_ref()
                                                     enabled=self.items.has_path() && !self.clean on activate=|_, _| Msg::MenuSave/>
                    <SimpleAction::new("save-as", None) ApplicationWindow::accels=["<Ctrl><Shift>s"].as_ref()
                                                        enabled=true on activate=|_, _| Msg::MenuSaveAs/>

                    <HeaderBar title=format!("TodoMVC - {}{}", title, clean) subtitle="wtf do we do now" show_close_button=true>
                        <MenuButton HeaderBar::pack_type=PackType::End @MenuButtonExt::direction=ArrowType::Down
                                    image="open-menu-symbolic">
                            <Menu::new_from_model(&main_menu)/>
                        </MenuButton>
                    </HeaderBar>
                    <Box spacing=10 orientation=Orientation::Vertical>
                        <Box spacing=10 orientation=Orientation::Horizontal Box::expand=false>
                            <Button image="edit-select-all" relief=ReliefStyle::Half
                                    always_show_image=true on clicked=|_| Msg::ToggleAll/>
                            <Entry placeholder_text="What needs to be done?"
                                   Box::expand=true Box::fill=true
                                   on activate=|entry| {
                                       let label = entry.get_text().map(|s| s.to_string()).unwrap_or_default();
                                       entry.select_region(0, label.len() as i32);
                                       Msg::Add {
                                           item: label
                                       }
                                   } />
                        </Box>
                        <ScrolledWindow Box::expand=true Box::fill=true>
                            <ListBox selection_mode=SelectionMode::None>
                                {
                                    self.filter(self.filter).enumerate()
                                        .map(|(index, item)| item.render(index))
                                }
                            </ListBox>
                        </ScrolledWindow>
                        <Box spacing=10 orientation=Orientation::Horizontal Box::expand=false>
                            <Label label=self.left_label()/>
                            <@Radio<Filter> active=self.filter Box::center_widget=true on_changed=|filter| Msg::Filter { filter } />
                            {
                                if self.filter(Filter::Completed).count() > 0 {
                                    (gtk!{
                                         <Button label="Clear completed" Box::pack_type=PackType::End
                                                 on clicked=|_| Msg::ClearCompleted/>
                                    }).into_iter()
                                } else {
                                    VNode::empty()
                                }
                            }
                        </Box>
                    </Box>
                </ApplicationWindow>
            </Application>
        }
    }
}

fn open(model: &mut Model) -> bool {
    let dialog = FileChooserNative::new(
        Some("Open a todo list"),
        vgtk::current_object()
            .and_then(|w| w.downcast::<Window>().ok())
            .as_ref(),
        FileChooserAction::Open,
        None,
        None,
    );
    dialog.set_modal(true);
    let filter = FileFilter::new();
    filter.set_name(Some("Todo list files"));
    filter.add_pattern("*.todo");
    dialog.add_filter(&filter);
    let result: ResponseType = dialog.run().into();
    if result == ResponseType::Accept {
        debug!("Selected file path: {:?}", dialog.get_filename());
        match Items::read_from(dialog.get_filename().unwrap()) {
            Ok(items) => {
                model.items = items;
                return true;
            }
            Err(err) => {
                error!("I/O error when opening file: {:?}", err);
            }
        }
    }
    false
}

fn save_as(model: &mut Model) -> bool {
    let dialog = FileChooserNative::new(
        Some("Save your todo list"),
        vgtk::current_object()
            .and_then(|w| w.downcast::<Window>().ok())
            .as_ref(),
        FileChooserAction::Save,
        None,
        None,
    );
    dialog.set_modal(true);
    let filter = FileFilter::new();
    filter.set_name(Some("Todo list files"));
    filter.add_pattern("*.todo");
    dialog.add_filter(&filter);
    let result: ResponseType = dialog.run().into();
    if result == ResponseType::Accept {
        debug!("Selected file path: {:?}", dialog.get_filename());
        if let Err(err) = model.items.write_to(dialog.get_filename().unwrap()) {
            error!("I/O error when saving file: {:?}", err);
        } else {
            return true;
        }
    }
    false
}
