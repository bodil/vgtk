use std::fs::File;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::path::PathBuf;

use gtk::*;

use vgtk::{gtk, VNode};

use serde_derive::{Deserialize, Serialize};

use crate::app::{Model, Msg};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Item {
    pub task: String,
    pub done: bool,
}

impl Item {
    pub fn new<S: Into<String>>(label: S) -> Self {
        Item {
            task: label.into(),
            done: false,
        }
    }

    pub fn render(&self, index: usize) -> VNode<Model> {
        let label = if self.done {
            format!(
                "<span strikethrough=\"true\" alpha=\"50%\">{}</span>",
                self.task
            )
        } else {
            self.task.clone()
        };
        gtk! {
            <ListBoxRow>
                <Box spacing=10 orientation=Orientation::Horizontal>
                    <CheckButton active=self.done on toggled=|_| Msg::Toggle { index } />
                    <Label label=label use_markup=true Box::fill=true />
                    <Button Box::pack_type=PackType::End relief=ReliefStyle::None
                            always_show_image=true image="edit-delete"
                            on clicked=|_| Msg::Remove { index } />
                </Box>
            </ListBoxRow>
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Items {
    items: Vec<Item>,
    path: Option<PathBuf>,
}

impl Items {
    pub fn read_from<P: AsRef<Path>>(path: P) -> std::io::Result<Items> {
        let path = path.as_ref();
        let file = File::open(path)?;
        serde_json::from_reader(file)
            .map_err(From::from)
            .map(|items| Items {
                items,
                path: Some(path.to_owned()),
            })
    }

    pub fn write_to<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let path = path.as_ref();
        let file = File::create(path)?;
        let result = serde_json::to_writer_pretty(file, &self.items).map_err(From::from);
        if result.is_ok() {
            self.path = Some(path.to_owned());
        }
        result
    }

    pub fn has_path(&self) -> bool {
        self.path.is_some()
    }

    pub fn get_path(&self) -> Option<&Path> {
        if let Some(ref path) = self.path {
            Some(path.as_path())
        } else {
            None
        }
    }
}

impl Deref for Items {
    type Target = Vec<Item>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl DerefMut for Items {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl FromIterator<Item> for Items {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Item>,
    {
        Items {
            items: iter.into_iter().collect(),
            path: None,
        }
    }
}
