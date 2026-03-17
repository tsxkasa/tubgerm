#[derive(Default, Debug)]
enum Panel {
    #[default]
    PLACEHOLDER,
}

#[derive(Default, Debug)]
pub struct MainView {
    focus: Panel,
}
