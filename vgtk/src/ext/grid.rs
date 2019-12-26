//! Helper traits and structs for using the GTK Grid layout.
//!
//! This module enables using the GTK Grid for laying out widgets in a window.
//! For example, the following snippet specifies a layout that will render to
//! look something like this:
//!
//! ```text
//!    +--------------------------------------------------+
//!    |         +-------------------------------------+  |
//!    | Label1: | Text input                          |  |
//!    |         +-------------------------------------+  |
//!    |         +-------------------------------------+  |
//!    | Label2: | Another bit of text                 |  |
//!    |         +-------------------------------------+  |
//!    |  +--------------------------------------------+  |
//!    |  |                                            |  |
//!    |  |                                            |  |
//!    |  |                                            |  |
//!    |  |              More stuff here               |  |
//!    |  |                                            |  |
//!    |  |                                            |  |
//!    |  |                                            |  |
//!    |  |                                            |  |
//!    |  +--------------------------------------------+  |
//!    |                                   +-----------+  |
//!    |                                   | A Button  |  |
//!    |                                   +-----------+  |
//!    +--------------------------------------------------+
//! ```
//!
//! ```ignore
//! use vgtk::ext::grid::GridExtHelpers;
//!
//! fn build() -> VNode<Model> {
//!   gtk! {
//!     <Grid row_spacing=10 column_spacing=10>
//!
//!       // --- Row 0 ---
//!
//!       // Widgets are placed by default in the top-left corner. So this
//!       // label does not need any additional annotation.
//!       <Label label="Label1:" halign=Alilgn::End />
//!
//!       // This text entry is being moved to column 2. We don't specify the
//!       // the row because, again, by default it is placed in the first row
//!       // which is what we want.
//!       <Entry Grid::left=1 hexpand=true />
//!
//!       // --- Row 1 ---
//!
//!       // Leave the column at its default of 0 and set the row to 1.
//!       <Label label="Label2:" Grid::top=1 halign=Alilgn::End />
//!
//!       // Place this text entry in row 1 and column 1.
//!       <Entry Grid::left=1 Grid::top=1 hexpand=true />
//!
//!       // --- Row 2 ---
//!
//!       // We want the following widget to span the width of the grid and
//!       // also consume excess vertical space.
//!       <ScrolledWindow Grid::top=2 Grid::width=2 hexpand=true vexpand=true>
//!         <ListBox />
//!       </ScrolledWindow>
//!
//!       // --- Row 3 ---
//!
//!       // We leave the first row unoccupied and only have a button in the second
//!       // column.
//!       <Button label="A Button" Grid::left=1 Grid::top=3 hexpand=false halign=Align::End />
//!
//!     </Grid>
//!   }
//! }
//! ```
//!

use glib::object::IsA;
use gtk::{GridExt, Widget};

/// Specifies the position of a widget in the grid.
///
/// The primary use of this struct is to fetch the current
/// position of a widget in the grid via [`GridExtHelpers::get_child_position`].
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// The column in the grid for the widget.
    pub left: i32,

    /// The row in the grid for the widget.
    pub top: i32,

    /// The number of columns the widget will span in the grid.
    pub width: i32,

    /// The number of rows the widget will span in the grid.
    pub height: i32,
}

/// These builder methods allow for an alternate syntax to be used when building
/// grid layouts. A brief example:
///
/// ```ignore
/// <Grid row_spacing=10 column_spacing=10>
///   <Label label="Label1:" halign=Align::End />
///   <Entry Grid::position=Position::default().with_left(1) hexpand=true />
/// </Grid>
/// ```
impl Position {
    /// Specify the left/column position of the widget.
    pub fn with_left(mut self, left: i32) -> Self {
        self.left = left;
        self
    }

    /// Specify the top/row position of the widget.
    pub fn with_top(mut self, top: i32) -> Self {
        self.top = top;
        self
    }

    /// Specify the column span for the widget.
    pub fn with_width(mut self, width: i32) -> Self {
        self.width = width;
        self
    }

    /// Specify the row span for the widget.
    pub fn with_height(mut self, height: i32) -> Self {
        self.height = height;
        self
    }
}

impl Default for Position {
    fn default() -> Self {
        Position {
            left: 0,
            top: 0,
            width: 1,
            height: 1,
        }
    }
}

/// Helper trait for `Grid` layout.
pub trait GridExtHelpers: GridExt {
    fn set_child_position<P: IsA<Widget>>(&self, child: &P, position: Position);
    fn get_child_position<P: IsA<Widget>>(&self, child: &P) -> Position;

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
    fn set_child_position<P: IsA<Widget>>(&self, child: &P, position: Position) {
        self.set_cell_left_attach(child, position.left);
        self.set_cell_top_attach(child, position.top);
        self.set_cell_width(child, position.width);
        self.set_cell_height(child, position.height);
    }

    fn get_child_position<P: IsA<Widget>>(&self, child: &P) -> Position {
        Position {
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
