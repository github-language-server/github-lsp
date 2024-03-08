use octocrab::models::{issues::Issue, IssueState};

use super::{GetDetail, GetEdit, GetLabel};

impl GetLabel for IssueState {
    fn get_label(&self) -> String {
        if let IssueState::Open = self {
            "Open".into()
        } else {
            "Closed".into()
        }
    }
}

impl GetLabel for Issue {
    fn get_label(&self) -> String {
        format!(
            "#{} [{}] {}",
            self.number,
            self.state.get_label(),
            self.title
        )
    }
}
impl GetEdit for Issue {
    fn get_edit(&self) -> String {
        let id = self.number;
        let title = &self.title;
        let url = self
            .url
            .to_string()
            .replace("api.", "")
            .replace("repos/", "");
        //TODO: cleanup & consider just printing the full URL and let GitHub format it
        format!("[#{id}: {title}]({url})")
    }
}
impl GetDetail for Issue {
    fn get_detail(&self) -> String {
        let title = self.title.to_string();
        format!(
            "# {} [{}] {}\n\n{}",
            self.number,
            self.state.get_label(),
            title,
            self.body.as_ref().unwrap_or(&title)
        )
    }
}
