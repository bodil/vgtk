//! Helper traits for adapting the GTK API to the [`gtk!`][gtk!] macro's mindset.
//!
//! It is generally a good idea to `use vgtk::ext::*;` wherever you're using the
//! [`gtk!`][gtk!] macro.
//!
//! [gtk!]: ../macro.gtk.html

#![allow(missing_docs)]

use gdk_pixbuf::Pixbuf;
use gio::{Action, ActionExt, ApplicationFlags};
use glib::{GString, IsA, Object, ObjectExt};
use gtk::{
    Application, ApplicationWindowExt, BoxExt, GridExt, GtkApplicationExt, GtkWindowExt,
    HeaderBarExt, ImageExt, LabelExt, NotebookExt, Widget, Window, WindowPosition, WindowType,
};

use colored::Colorize;
use log::trace;

use crate::types::GridPosition;

/// Helper trait for [`Application`][Application].
///
/// [Application]: ../../gtk/struct.Application.html
pub trait ApplicationHelpers: GtkApplicationExt {
    /// Construct a new [`Application`][Application] and panic if it fails.
    ///
    /// This is like [`Application::new`][new], but returns an [`Application`][Application] instead of
    /// an [`Option`][Option]`<`[`Application`][Application]`>`, so you can use it as a constructor in the [`gtk!`][gtk!]
    /// macro.
    ///
    /// [gtk!]: ../macro.gtk.html
    /// [Application]: ../../gtk/struct.Application.html
    /// [new]: ../../gtk/struct.Application.html#method.new
    /// [Option]: https://doc.rust-lang.org/std/option/enum.Option.html
    fn new_unwrap(application_id: Option<&str>, flags: ApplicationFlags) -> Application {
        Application::new(application_id, flags).expect("unable to create Application object")
    }

    fn get_child_accels<P: IsA<Action>>(&self, action: &P) -> Vec<GString> {
        self.get_accels_for_action(&format!(
            "app.{}",
            action
                .as_ref()
                .get_name()
                .expect("Action has no name")
                .as_str()
        ))
    }

    fn set_child_accels<P: IsA<Action>>(&self, action: &P, accels: &[&str]) {
        self.set_accels_for_action(
            &format!(
                "app.{}",
                action
                    .as_ref()
                    .get_name()
                    .expect("Action has no name")
                    .as_str()
            ),
            accels,
        )
    }
}

impl<A> ApplicationHelpers for A where A: GtkApplicationExt {}

/// Helper trait for [`ApplicationWindow`][ApplicationWindow].
///
/// [ApplicationWindow]: ../../gtk/struct.ApplicationWindow.html
pub trait ApplicationWindowHelpers: ApplicationWindowExt + GtkWindowExt + IsA<Window> {
    fn get_child_accels<P: IsA<Action>>(&self, action: &P) -> Vec<GString> {
        let app = self
            .get_application()
            .expect("ApplicationWindow has no Application!");
        app.get_accels_for_action(&format!(
            "win.{}",
            action
                .as_ref()
                .get_name()
                .expect("Action has no name")
                .as_str()
        ))
    }

    fn set_child_accels<P: IsA<Action>>(&self, action: &P, accels: &'static [&str]) {
        let name = format!(
            "win.{}",
            action
                .as_ref()
                .get_name()
                .expect("Action has no name")
                .as_str()
        );
        if let Some(app) = self.get_application() {
            app.set_accels_for_action(&name, accels);
        } else {
            use std::cell::Cell;
            use std::rc::Rc;

            let id = Rc::new(Cell::new(None));
            let inner_id = id.clone();
            id.set(Some(self.connect_property_application_notify(
                move |window: &Self| {
                    if let Some(app) = window.get_application() {
                        trace!(
                            "{} {} -> {}",
                            "Action:".bright_black(),
                            name.bright_cyan().bold(),
                            format!("{:?}", accels).bright_green().bold()
                        );
                        app.set_accels_for_action(&name, accels);
                        window.disconnect(inner_id.replace(None).unwrap());
                    }
                },
            )));
        }
    }
}

impl<A> ApplicationWindowHelpers for A where A: ApplicationWindowExt + GtkWindowExt + IsA<Window> {}

/// Helper trait for [`Window`][Window].
///
/// [Window]: ../../gtk/struct.Window.html
pub trait WindowExtHelpers: GtkWindowExt {
    fn get_default_height(&self) -> i32 {
        self.get_property_default_height()
    }

    fn set_default_height(&self, default_height: i32) {
        self.set_property_default_height(default_height)
    }

    fn get_default_width(&self) -> i32 {
        self.get_property_default_width()
    }

    fn set_default_width(&self, default_width: i32) {
        self.set_property_default_width(default_width)
    }

    fn get_has_toplevel_focus(&self) -> bool {
        self.get_property_has_toplevel_focus()
    }

    fn get_is_active(&self) -> bool {
        self.get_property_is_active()
    }

    fn get_is_maximized(&self) -> bool {
        self.get_property_is_maximized()
    }

    fn get_type(&self) -> WindowType {
        self.get_property_type()
    }

    fn get_window_position(&self) -> WindowPosition {
        self.get_property_window_position()
    }

    fn set_window_position(&self, window_position: WindowPosition) {
        self.set_property_window_position(window_position)
    }
}

impl<A> WindowExtHelpers for A where A: GtkWindowExt {}

/// Helper trait for [`Box`][Box].
///
/// [Box]: ../../gtk/struct.Box.html
pub trait BoxExtHelpers: BoxExt {
    fn get_child_center_widget(&self, _child: &Object) -> bool {
        // Always compare true, it's all taken care of in add_child().
        true
    }

    fn set_child_center_widget(&self, _child: &Object, _center: bool) {
        // This is handled by add_child() rules. The setter is a no-op.
    }
}

impl<A> BoxExtHelpers for A where A: BoxExt {}

/// Helper trait for [`HeaderBar`][HeaderBar].
///
/// [HeaderBar]: ../../gtk/struct.HeaderBar.html
pub trait HeaderBarExtHelpers: HeaderBarExt {
    fn get_child_custom_title<P: IsA<Widget>>(&self, _child: &P) -> bool {
        // Always compare true, it's all taken care of in add_child().
        true
    }

    fn set_child_custom_title<P: IsA<Widget>>(&self, _child: &P, _center: bool) {
        // This is handled by add_child() rules. The setter is a no-op.
    }
}

impl<A> HeaderBarExtHelpers for A where A: HeaderBarExt {}

/// Helper trait for [`Image`][Image].
///
/// [Image]: ../../gtk/struct.Image.html
pub trait ImageExtHelpers: ImageExt {
    fn set_pixbuf(&self, pixbuf: Option<Pixbuf>) {
        self.set_from_pixbuf(pixbuf.as_ref());
    }
}

impl<A> ImageExtHelpers for A where A: ImageExt {}

/// Helper trait for [`Label`][Label].
///
/// [Label]: ../../gtk/struct.Label.html
pub trait LabelExtHelpers: LabelExt {
    fn get_markup(&self) -> GString {
        self.get_label()
    }
}

impl<A> LabelExtHelpers for A where A: LabelExt {}

/// Helper trait for [`Notebook`][Notebook].
///
/// [Notebook]: ../../gtk/struct.Notebook.html
pub trait NotebookExtHelpers: NotebookExt {
    fn set_child_action_widget_start<P: IsA<Widget>>(&self, _child: &P, _val: bool) {
        // This is handled by add_child() rules. The setter is a no-op.
    }
    fn get_child_action_widget_start<P: IsA<Widget>>(&self, _child: &P) -> bool {
        // Always compare true, it's all taken care of in add_child().
        true
    }
    fn set_child_action_widget_end<P: IsA<Widget>>(&self, _child: &P, _val: bool) {
        // This is handled by add_child() rules. The setter is a no-op.
    }
    fn get_child_action_widget_end<P: IsA<Widget>>(&self, _child: &P) -> bool {
        // Always compare true, it's all taken care of in add_child().
        true
    }
    fn set_child_tab_with_label<P: IsA<Widget>>(&self, _child: &P, _val: bool) {
        // This is handled by add_child() rules. The setter is a no-op.
    }
    fn get_child_tab_with_label<P: IsA<Widget>>(&self, _child: &P) -> bool {
        // Always compare true, it's all taken care of in add_child().
        true
    }
}

impl<A> NotebookExtHelpers for A where A: NotebookExt {}

/// Helper trait for [`Grid`][Grid] layout.
///
/// This helper enables using the GTK Grid for laying out widgets in a window.
/// For example, the following snippet specifies a layout that will render to
/// look something like this:
///
/// ```text
///    +--------------------------------------------------+
///    |         +-------------------------------------+  |
///    | Label1: | Text input                          |  |
///    |         +-------------------------------------+  |
///    |         +-------------------------------------+  |
///    | Label2: | Another bit of text                 |  |
///    |         +-------------------------------------+  |
///    |  +--------------------------------------------+  |
///    |  |                                            |  |
///    |  |                                            |  |
///    |  |                                            |  |
///    |  |              More stuff here               |  |
///    |  |                                            |  |
///    |  |                                            |  |
///    |  |                                            |  |
///    |  |                                            |  |
///    |  +--------------------------------------------+  |
///    |                                   +-----------+  |
///    |                                   | A Button  |  |
///    |                                   +-----------+  |
///    +--------------------------------------------------+
/// ```
///
/// ```rust,no_run
/// # #![recursion_limit="256"]
/// # use vgtk::{gtk, VNode};
/// # use vgtk::ext::*;
/// # use vgtk::lib::gtk::*;
/// # fn build() -> VNode<()> {
/// gtk! {
///     <Grid row_spacing=10 column_spacing=10>
///
///       // --- Row 0 ---
///
///       // Widgets are placed by default in the top left corner, so this
///       // label does not need any additional annotation.
///       <Label label="Label1:" halign=Align::End />
///
///       // This text entry is being moved to column 2. We don't specify
///       // the row because, again, by default it is placed in the first row
///       // which is what we want.
///       <Entry Grid::left=1 hexpand=true />
///
///       // --- Row 1 ---
///
///       // Leave the column at its default of 0 and set the row to 1.
///       <Label label="Label2:" Grid::top=1 halign=Align::End />
///
///       // Place this text entry in row 1 and column 1.
///       <Entry Grid::left=1 Grid::top=1 hexpand=true />
///
///       // --- Row 2 ---
///
///       // We want the following widget to span the width of the grid and
///       // also consume excess vertical space.
///       <ScrolledWindow Grid::top=2 Grid::width=2 hexpand=true vexpand=true>
///         <ListBox />
///       </ScrolledWindow>
///
///       // --- Row 3 ---
///
///       // We leave the first row unoccupied and only have a button in the second
///       // column.
///       <Button label="A Button" Grid::left=1 Grid::top=3 hexpand=false halign=Align::End />
///
///     </Grid>
/// }
/// # }
/// ```
///
///
/// [Grid]: ../../gtk/struct.Grid.html
pub trait GridExtHelpers: GridExt {
    fn set_child_position<P: IsA<Widget>>(&self, child: &P, position: GridPosition);
    fn get_child_position<P: IsA<Widget>>(&self, child: &P) -> GridPosition;

    fn set_child_left<P: IsA<Widget>>(&self, child: &P, left: i32);
    fn get_child_left<P: IsA<Widget>>(&self, child: &P) -> i32;

    fn set_child_top<P: IsA<Widget>>(&self, child: &P, top: i32);
    fn get_child_top<P: IsA<Widget>>(&self, child: &P) -> i32;

    fn set_child_width<P: IsA<Widget>>(&self, child: &P, width: i32);
    fn get_child_width<P: IsA<Widget>>(&self, child: &P) -> i32;

    fn set_child_height<P: IsA<Widget>>(&self, child: &P, height: i32);
    fn get_child_height<P: IsA<Widget>>(&self, child: &P) -> i32;
}

impl<G> GridExtHelpers for G
where
    G: GridExt,
{
    fn set_child_position<P: IsA<Widget>>(&self, child: &P, position: GridPosition) {
        self.set_cell_left_attach(child, position.left);
        self.set_cell_top_attach(child, position.top);
        self.set_cell_width(child, position.width);
        self.set_cell_height(child, position.height);
    }

    fn get_child_position<P: IsA<Widget>>(&self, child: &P) -> GridPosition {
        GridPosition {
            left: self.get_cell_left_attach(child),
            top: self.get_cell_top_attach(child),
            width: self.get_cell_width(child),
            height: self.get_cell_height(child),
        }
    }

    fn set_child_left<P: IsA<Widget>>(&self, child: &P, left: i32) {
        self.set_cell_left_attach(child, left);
    }

    fn get_child_left<P: IsA<Widget>>(&self, child: &P) -> i32 {
        self.get_cell_left_attach(child)
    }

    fn set_child_top<P: IsA<Widget>>(&self, child: &P, top: i32) {
        self.set_cell_top_attach(child, top);
    }

    fn get_child_top<P: IsA<Widget>>(&self, child: &P) -> i32 {
        self.get_cell_top_attach(child)
    }

    fn set_child_width<P: IsA<Widget>>(&self, child: &P, width: i32) {
        self.set_cell_width(child, width);
    }

    fn get_child_width<P: IsA<Widget>>(&self, child: &P) -> i32 {
        self.get_cell_width(child)
    }

    fn set_child_height<P: IsA<Widget>>(&self, child: &P, height: i32) {
        self.set_cell_height(child, height);
    }

    fn get_child_height<P: IsA<Widget>>(&self, child: &P) -> i32 {
        self.get_cell_height(child)
    }
}
