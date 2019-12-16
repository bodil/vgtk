use std::fmt::Debug;
use std::sync::Arc;

#[cfg(feature = "grid-layout")]
use vgtk::grid::GridProps;
use vgtk::lib::gio::{ActionExt, ApplicationFlags, File, FileExt, SimpleAction};
use vgtk::lib::glib::Error;
use vgtk::lib::gtk::prelude::*;
use vgtk::lib::gtk::*;
use vgtk::{ext::*, gtk, on_signal, Component, UpdateAction, VNode};

use strum_macros::{Display, EnumIter};

use crate::about::AboutDialog;
use crate::items::{Item, Items};
#[cfg(feature = "box-layout")]
use crate::radio::Radio;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Display, EnumIter)]
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
    items: Arc<Items>,
    filter: Filter,
    file: Option<File>,
    clean: bool,
}

impl Default for Model {
    fn default() -> Self {
        Model {
            items: Default::default(),
            filter: Filter::All,
            file: None,
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

    #[cfg(feature = "box-layout")]
    fn main_panel(&self) -> VNode<Model> {
        gtk! {
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
        }
    }

    #[cfg(feature = "grid-layout")]
    fn main_panel(&self) -> VNode<Model> {
        gtk! {
            <Grid row_spacing=10 column_spacing=10>
                // Row 0
                <Button image="edit-select-all" relief=ReliefStyle::Half
                        always_show_image=true on clicked=|_| Msg::ToggleAll/>
                <Entry placeholder_text="What needs to be done?"
                       Grid::left=1
                       hexpand=true
                       on activate=|entry| {
                           let label = entry.get_text().map(|s| s.to_string()).unwrap_or_default();
                           entry.select_region(0, label.len() as i32);
                           Msg::Add {
                               item: label
                           }
                       } />

                // Row 1
                <ScrolledWindow Grid::top=1 Grid::width=2 hexpand=true vexpand=true>
                    <ListBox selection_mode=SelectionMode::None>
                        {
                            self.filter(self.filter).enumerate()
                                .map(|(index, item)| item.render(index))
                        }
                    </ListBox>
                </ScrolledWindow>

                // Row 2
                <Grid Grid::top=2 Grid::width=2 hexpand=true>
                    <Label label=self.left_label() halign=Align::Start/>
                    <Box orientation=Orientation::Horizontal spacing=10 halign=Align::Center hexpand=true Grid::left=1>
                        <ToggleButton label="All"
                                      active=self.filter == Filter::All
                                      on toggled=|_| Msg::Filter { filter: Filter::All }/>
                        <ToggleButton label="Active"
                                      active=self.filter == Filter::Active
                                      on toggled=|_| Msg::Filter { filter: Filter::Active }/>
                        <ToggleButton label="Completed"
                                      active=self.filter == Filter::Completed
                                      on toggled=|_| Msg::Filter { filter: Filter::Completed }/>
                    </Box>
                    {
                        if self.filter(Filter::Completed).count() > 0 {
                            (gtk!{
                                 <Button label="Clear completed"
                                         Grid::left=2
                                         halign=Align::End
                                         on clicked=|_| Msg::ClearCompleted/>
                            }).into_iter()
                        } else {
                            VNode::empty()
                        }
                    }
                </Grid>
            </Grid>
        }
    }
}

#[derive(Clone, Debug)]
pub enum Msg {
    NoOp,
    Add { item: String },
    Remove { index: usize },
    Toggle { index: usize },
    Filter { filter: Filter },
    ToggleAll,
    ClearCompleted,
    Exit,
    Loaded { file: File, items: Items },
    Saved { file: Option<File> },
    FileError { error: Error },
    MenuOpen,
    MenuSave,
    MenuSaveAs,
    MenuAbout,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn update(&mut self, msg: Self::Message) -> UpdateAction<Self> {
        let left = self.filter(Filter::Active).count();
        match msg {
            Msg::NoOp => return UpdateAction::None,
            Msg::FileError { error } => {
                return UpdateAction::defer(async move {
                    vgtk::message_dialog(
                        vgtk::current_window().as_ref(),
                        DialogFlags::empty(),
                        MessageType::Error,
                        ButtonsType::Ok,
                        true,
                        format!("<b>AN ERROR HAS OCCURRED!</b>\n\n{}", error),
                    )
                    .await;
                    Msg::NoOp
                })
            }
            Msg::Add { item } => {
                Arc::make_mut(&mut self.items).push(Item::new(item));
                self.clean = false;
            }
            Msg::Remove { index } => {
                Arc::make_mut(&mut self.items).remove(index);
                self.clean = false;
            }
            Msg::Toggle { index } => {
                Arc::make_mut(&mut self.items)[index].done = !self.items[index].done;
                self.clean = false;
            }
            Msg::Filter { filter } => {
                self.filter = filter;
            }
            Msg::ToggleAll if left > 0 => {
                Arc::make_mut(&mut self.items)
                    .iter_mut()
                    .for_each(|item| item.done = true);
                self.clean = false;
            }
            Msg::ToggleAll => {
                Arc::make_mut(&mut self.items)
                    .iter_mut()
                    .for_each(|item| item.done = false);
                self.clean = false;
            }
            Msg::ClearCompleted => {
                Arc::make_mut(&mut self.items).retain(|item| !item.done);
                self.clean = false;
            }
            Msg::Exit => {
                vgtk::quit();
                return UpdateAction::None;
            }
            Msg::Loaded { file, items } => {
                self.items = Arc::new(items);
                self.file = Some(file);
                self.clean = true;
            }
            Msg::Saved { file } => {
                self.clean = true;
                if let Some(file) = file {
                    self.file = Some(file);
                }
            }
            Msg::MenuOpen => {
                return UpdateAction::defer(async {
                    match open().await {
                        Ok(Some((file, items))) => Msg::Loaded { file, items },
                        Ok(None) => Msg::NoOp,
                        Err(error) => Msg::FileError { error },
                    }
                });
            }
            Msg::MenuSave => {
                let items = self.items.clone();
                let file = self.file.clone().unwrap();
                return UpdateAction::defer(async move {
                    match save(&*items, &file).await {
                        Ok(_) => Msg::Saved { file: None },
                        Err(error) => Msg::FileError { error },
                    }
                });
            }
            Msg::MenuSaveAs => {
                let items = self.items.clone();
                return UpdateAction::defer(async move {
                    match save_as(&*items).await {
                        Ok(Some(file)) => Msg::Saved { file: Some(file) },
                        Ok(None) => Msg::NoOp,
                        Err(error) => Msg::FileError { error },
                    }
                });
            }
            Msg::MenuAbout => {
                AboutDialog::run();
                return UpdateAction::None;
            }
        }
        UpdateAction::Render
    }

    fn view(&self) -> VNode<Model> {
        let title = if let Some(name) = self.file.as_ref().and_then(|p| p.get_basename()) {
            name.to_str().unwrap().to_string()
        } else {
            "Untitled todo list".to_string()
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
                                                     enabled=self.file.is_some() && !self.clean on activate=|_, _| Msg::MenuSave/>
                    <SimpleAction::new("save-as", None) ApplicationWindow::accels=["<Ctrl><Shift>s"].as_ref()
                                                        enabled=true on activate=|_, _| Msg::MenuSaveAs/>

                    <HeaderBar title=format!("TodoMVC - {}{}", title, clean) subtitle="wtf do we do now" show_close_button=true>
                        <MenuButton HeaderBar::pack_type=PackType::End @MenuButtonExt::direction=ArrowType::Down
                                    image="open-menu-symbolic">
                            <Menu::new_from_model(&main_menu)/>
                        </MenuButton>
                    </HeaderBar>
                    {
                        self.main_panel()
                    }
                </ApplicationWindow>
            </Application>
        }
    }
}

async fn open() -> Result<Option<(File, Items)>, Error> {
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
    dialog.show();
    if on_signal!(dialog, connect_response).await == Ok(ResponseType::Accept) {
        let file = dialog.get_file().unwrap();
        Items::read_from(&file)
            .await
            .map(|items| Some((file, items)))
    } else {
        Ok(None)
    }
}

async fn save(items: &Items, file: &File) -> Result<(), Error> {
    items.write_to(file).await
}

async fn save_as(items: &Items) -> Result<Option<File>, Error> {
    let dialog = FileChooserNative::new(
        Some("Save your todo list"),
        vgtk::current_window().as_ref(),
        FileChooserAction::Save,
        None,
        None,
    );
    dialog.set_modal(true);
    let filter = FileFilter::new();
    filter.set_name(Some("Todo list files"));
    filter.add_pattern("*.todo");
    dialog.add_filter(&filter);
    dialog.show();
    if on_signal!(dialog, connect_response).await == Ok(ResponseType::Accept) {
        let file = dialog.get_file().unwrap();
        save(items, &file).await.map(|_| Some(file))
    } else {
        Ok(None)
    }
}
