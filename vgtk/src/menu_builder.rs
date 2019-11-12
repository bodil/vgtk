use gio::{Menu, MenuItem};

/// Makes a `gtk::Menu` for you.
pub struct MenuBuilder {
    menu: Menu,
}

/// Construct a `MenuBuilder`.
pub fn menu() -> MenuBuilder {
    MenuBuilder { menu: Menu::new() }
}

impl MenuBuilder {
    /// Add a `MenuItem` to this menu.
    pub fn item(self, label: &str, action: &str) -> Self {
        let item = MenuItem::new(Some(label), Some(action));
        self.menu.append_item(&item);
        self
    }

    /// Add a section to this menu.
    pub fn section(self, section: MenuBuilder) -> Self {
        let section = section.build();
        self.menu.append_section(None, &section);
        self
    }

    /// Add a section with a label to this menu.
    pub fn section_label(self, label: &str, section: MenuBuilder) -> Self {
        let section = section.build();
        self.menu.append_section(Some(label), &section);
        self
    }

    /// Add a submenu to this menu.
    pub fn sub(self, label: &str, submenu: MenuBuilder) -> Self {
        let submenu = submenu.build();
        self.menu.append_submenu(Some(label), &submenu);
        self
    }

    /// Finalise the `MenuBuilder` and get your `Menu`.
    pub fn build(self) -> Menu {
        self.menu
    }
}
