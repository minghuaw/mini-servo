//! Impl a simple winit app to visualize the webpage

enum AppState {
    Initial,
    Running,
}

pub struct App {
    state: AppState
}