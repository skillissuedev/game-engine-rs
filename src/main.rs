use framework::DebugMode;

mod assets;
mod math_utils;
mod managers;
mod framework;
mod game;
mod components;
mod object;

fn main() {
    #[cfg(debug_assertions)]
    framework::start_game(DebugMode::ShowFps);

    #[cfg(not(debug_assertions))]
    framework::start_game(DebugMode::ShowFps);
}
