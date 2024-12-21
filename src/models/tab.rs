#[derive(PartialEq, Clone)]
pub enum Tab {
    General,
    Files,
    Peers,
}

pub struct TabView {
    pub tabs: [(Tab, String, bool); 3],
    pub selected: Tab,
}
