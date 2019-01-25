use glib::{MainContext, MainLoop as GMainLoop};
use std::cell::Cell;
use std::rc::Rc;

pub trait MainLoop: Clone {
    fn quit(&self, return_code: i32);
    fn run(&self) -> i32;
}

#[derive(Clone)]
pub struct GtkMainLoop {
    main_loop: GMainLoop,
    return_code: Rc<Cell<i32>>,
}

impl GtkMainLoop {
    #[must_use]
    pub fn new(context: MainContext) -> Self {
        GtkMainLoop {
            main_loop: GMainLoop::new(&context, false),
            return_code: Default::default(),
        }
    }
}

impl MainLoop for GtkMainLoop {
    fn quit(&self, return_code: i32) {
        self.return_code.set(return_code);
        self.main_loop.quit();
    }

    fn run(&self) -> i32 {
        self.main_loop.run();
        self.return_code.get()
    }
}
