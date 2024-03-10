use octocrab::models::Author;

use super::{GetDetail, GetEdit, GetLabel};

impl GetLabel for Author {
    fn get_label(&self) -> String {
        self.login.to_string()
    }
}
impl GetEdit for Author {
    fn get_edit(&self) -> String {
        let id = self.login.to_owned();
        format!("[{id}](https://github.com/{id})")
    }
}
impl GetDetail for Author {
    fn get_detail(&self) -> String {
        self.get_label()
    }
}
