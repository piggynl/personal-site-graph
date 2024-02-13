use crate::page::Page;

pub struct Site {
    domain: String,
    pages: Vec<Page>,
}

impl Site {
    pub fn new(domain: String) -> Self {
        Site {
            domain,
            pages: Vec::new(),
        }
    }
}
