//! A doubly-linked list in 50 LOCs of stable and safe Rust.
// Backup-fork of https://play.rust-lang.org/?gist=c3db81ec94bf231b721ef483f58deb35
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::{Rc, Weak};

type Prop = Rc<RefCell<InnerProp>>;

#[derive(Debug)]
struct InnerProp {
    ctrl_idx: i32,
    parent: Option<Prop>,
    child: Option<Prop>,
    next_sibling: Option<Prop>,
}

impl InnerProp {
    pub fn new(ctrl_idx: i32) -> Prop {
        Rc::new(RefCell::new(Self {
            ctrl_idx: ctrl_idx,
            parent: None,
            child: None,
            next_sibling: None,
        }))
    }

    pub fn display_link(link: &Option<Prop>) -> String {
        match link {
            Some(p) => format!("{}", p.borrow().ctrl_idx),
            None => format!("None"),
        }
    }

    pub fn display(&self) -> String {
        format!(
            "Prop(id={}, parent={}, child={}, next_sib={})",
            self.ctrl_idx,
            InnerProp::display_link(&self.parent),
            InnerProp::display_link(&self.child),
            InnerProp::display_link(&self.next_sibling),
        )
    }
}

fn set_child_link(child: &Prop, parent: &Prop) {
    let mut child_ref = child.borrow_mut();
    child_ref.parent = Some(parent.clone());

    let mut x = parent.borrow_mut();
    x.child = Some(child.clone());
}

fn main() {
    let a = InnerProp::new(0);
    let b = InnerProp::new(1);
    let c = InnerProp::new(3);

    println!("a={}", a.borrow().display());
    println!("b={}", b.borrow().display());
    println!("c={}\n\n", c.borrow().display());

    set_child_link(&a, &b);
    set_child_link(&b, &c);

    println!("a={}", a.borrow().display());
    println!("b={}", b.borrow().display());
    println!("c={}\n\n", c.borrow().display());
}
