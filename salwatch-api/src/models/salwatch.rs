#[derive(Debug)]
pub struct ChangeEvent {
    pub repo: String,
    pub title: String,
    pub author: String,
    pub pr_url: String,
}