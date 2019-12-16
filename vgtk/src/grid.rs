use glib::object::IsA;
use gtk::{Grid, GridExt, Widget};

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    left: i32,
    top: i32,
    width: i32,
    height: i32,
}

impl Position {
    pub fn with_left(mut self, left: i32) -> Self {
        self.left = left;
        self
    }

    pub fn with_top(mut self, top: i32) -> Self {
        self.top = top;
        self
    }

    pub fn with_width(mut self, width: i32) -> Self {
        self.width = width;
        self
    }

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

pub trait GridProps {
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

impl GridProps for Grid {
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
