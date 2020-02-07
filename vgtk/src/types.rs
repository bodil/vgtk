//! Useful types for GTK extensions.

/// Specifies the position of a widget in a [`Grid`][Grid].
///
/// The primary use of this struct is to fetch the current
/// position of a widget in the grid via
/// [`GridExtHelpers::get_child_position`][get_child_position].
///
/// [Grid]: ../../gtk/struct.Grid.html
/// [get_child_position]: ../ext/trait.GridExtHelpers.html#tymethod.get_child_position
#[derive(Debug, Clone, PartialEq)]
pub struct GridPosition {
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
/// ```rust,no_run
/// # use vgtk::{gtk, VNode, types::GridPosition};
/// # use vgtk::ext::*;
/// # use vgtk::lib::gtk::*;
/// # fn build() -> VNode<()> { gtk! {
/// <Grid row_spacing=10 column_spacing=10>
///   <Label label="Label1:" halign=Align::End />
///   <Entry Grid::position=GridPosition::default().with_left(1) hexpand=true />
/// </Grid>
/// # }}
/// ```
impl GridPosition {
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

impl Default for GridPosition {
    fn default() -> Self {
        Self {
            left: 0,
            top: 0,
            width: 1,
            height: 1,
        }
    }
}
