#[derive(PartialEq, Clone)]
pub enum Tab {
    General,
    Files,
    Peers,
    Trackers,
}

pub struct TabView {
    pub tabs: [(Tab, String, bool); 4],
    pub selected: Tab,
}
