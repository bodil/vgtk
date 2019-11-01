use gio::{Menu, MenuItem};

pub struct MenuBuilder {
    menu: Menu,
}

pub fn menu() -> MenuBuilder {
    MenuBuilder { menu: Menu::new() }
}

impl MenuBuilder {
    pub fn item(self, label: &str, action: &str) -> Self {
        let item = MenuItem::new(Some(label), Some(action));
        self.menu.append_item(&item);
        self
    }

    pub fn section(self, section: MenuBuilder) -> Self {
        let section = section.menu;
        self.menu.append_section(None, &section);
        self
    }

    pub fn section_label(self, label: &str, section: MenuBuilder) -> Self {
        let section = section.menu;
        self.menu.append_section(Some(label), &section);
        self
    }

    pub fn sub(self, label: &str, submenu: MenuBuilder) -> Self {
        let submenu = submenu.menu;
        self.menu.append_submenu(Some(label), &submenu);
        self
    }

    pub fn build(self) -> Menu {
        self.menu
    }
}
