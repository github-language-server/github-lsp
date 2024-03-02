use octocrab::models::Repository;

use super::{GetDetail, GetEdit, GetLabel};

impl GetLabel for Repository {
    fn get_label(&self) -> String {
        let owner = self
            .owner
            .as_ref()
            .expect("A repo must have an owner")
            .login
            .to_owned();
        format!("{}/{}", owner, self.name)
    }
}
impl GetEdit for Repository {
    fn get_edit(&self) -> String {
        let label = self.get_label();
        format!("[{label}](https://github.com/{label})")
    }
}
impl GetDetail for Repository {
    fn get_detail(&self) -> String {
        let description = self
            .description
            .as_ref()
            .unwrap_or(&"No description.".to_string())
            .to_owned();
        format!("{}\n{}", self.get_edit(), description)
    }
}
