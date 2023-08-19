use framework::DebugMode;

mod assets;
mod components;
mod framework;
mod game;
mod managers;
mod math_utils;
mod object;

fn main() {
    #[cfg(debug_assertions)]
    framework::start_game(DebugMode::ShowFps);

    #[cfg(not(debug_assertions))]
    framework::start_game(DebugMode::ShowFps);
}
