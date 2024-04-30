use crate::{
    game::game_main,
    managers::{
        self,
        assets::get_full_asset_path,
        input, navigation,
        networking::{self, NetworkingMode},
        physics,
        render::{self, ShadowTextures},
        sound::{self, set_listener_transform},
        systems::{self, SystemValue},
    },
};
use egui_glium::egui_winit::egui::{FontData, FontDefinitions, FontFamily, Window};
use glium::{
    backend::glutin,
    glutin::{
        dpi::PhysicalSize,
        event::WindowEvent,
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    Display,
};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap, fs, num::NonZeroU32, time::{Duration, Instant}
};

static mut DEBUG_MODE: DebugMode = DebugMode::None;
static mut DELTA_TIME: Duration = Duration::new(0, 0);
static FONT: Lazy<Vec<u8>> =
    Lazy::new(|| fs::read(get_full_asset_path("fonts/JetBrainsMono-Regular.ttf")).unwrap());
static mut SYSTEM_GLOBALS: Lazy<HashMap<String, SystemValue>> = Lazy::new(|| HashMap::new());

pub fn start_game_with_render(debug_mode: DebugMode) {
    unsafe { DEBUG_MODE = debug_mode }
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("projectbaldej")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_transparent(false);
    let cb = ContextBuilder::new().with_multisampling(8).with_srgb(true); //.with_vsync(true);
    let display = Display::new(wb, cb, &event_loop).expect("failed to create glium display");
    let mut egui_glium = egui_glium::EguiGlium::new(&display, &event_loop);

    let mut fps = 0;

    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("JetBrains Mono".into(), FontData::from_static(&FONT));
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "JetBrains Mono".into());
    egui_glium.egui_ctx.set_fonts(fonts);
    let mut ui_state = managers::ui::UiState::default();

    let mut frames_count: usize = 0;
    let mut now = std::time::Instant::now();
    let mut last_frame = std::time::Instant::now();

    sound::init().unwrap();
    render::init(&display);

    navigation::update();
    game_main::start();

    let mut win_w = display.gl_window().window().inner_size().width;
    let mut win_h = display.gl_window().window().inner_size().height;

    let shadow_textures = ShadowTextures::new(&display, 8192, 4096);

    let frame_time = Duration::from_millis(16);

    event_loop.run(move |ev, _, control_flow| {
        let time_since_last_frame = last_frame.elapsed();
        update_game(time_since_last_frame);
        egui_glium.run(&display, |ctx| match get_debug_mode() {
            DebugMode::None => (),
            _ => {
                Window::new("inspector").show(ctx, |ui| {
                    managers::ui::draw_inspector(ui, &fps, &mut ui_state);
                });
            }
        });

        if let NetworkingMode::Server(_) = networking::get_current_networking_mode() {
            if time_since_last_frame < frame_time {
                let wait_time = frame_time - time_since_last_frame;
                let wait_until = Instant::now() + wait_time;
                *control_flow = ControlFlow::WaitUntil(wait_until);
            }
            last_frame = Instant::now();
        }

        match ev {
            glium::glutin::event::Event::MainEventsCleared => {
                match networking::get_current_networking_mode() {
                    networking::NetworkingMode::Server(_) => (),
                    _ => {
                        set_listener_transform(
                            render::get_camera_position(),
                            render::get_camera_front(),
                        );

                        let mut target = display.draw();

                        render::draw(&display, &mut target, &shadow_textures);
                        //game_main::render();
                        render::debug_draw(&display, &mut target);
                        egui_glium.paint(&display, &mut target);

                        target.finish().unwrap();
                        frames_count += 1;
                        last_frame = Instant::now();
                    }
                }
            }
            glutin::glutin::event::Event::WindowEvent { event, .. } => {
                let event_response = egui_glium.on_event(&event);
                if event_response.consumed == false {
                    input::reg_event(&event);
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                            networking::disconnect();
                        }
                        WindowEvent::Resized(size) => {
                            win_w = size.width;
                            win_h = size.height;

                            unsafe {
                                render::ASPECT_RATIO = win_w as f32 / win_h as f32;
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }

        if let Some(new_fps) = get_fps(&now, &frames_count) {
            fps = new_fps;
            display
                .gl_window()
                .window()
                .set_title(&format!("projectbaldej: {fps} fps"));
            frames_count = 0;
            now = Instant::now();
        }
    });
}

pub fn start_game_without_render() {
    println!("starting game without render");
    game_main::start();

    let tickrate_tick = Duration::from_millis(16);
    let clock = chron::Clock::new(NonZeroU32::new(60).unwrap());

    for tick in clock {
        match tick {
            chron::clock::Tick::Update => {
                update_game(tickrate_tick);
            }
            chron::clock::Tick::Render { interpolation: _ } => {}
        }
    }
}

fn update_game(delta_time: Duration) {
    set_delta_time(delta_time);
    physics::update();
    networking::update(delta_time);
    navigation::update();
    game_main::update();
    systems::update();
    navigation::create_grids();
    input::update();
}

fn get_fps(now: &Instant, frames: &usize) -> Option<usize> {
    let one_second = std::time::Duration::new(1, 0);

    if now.elapsed() > one_second {
        return Some(frames.clone());
    }
    None
}

pub fn get_debug_mode() -> DebugMode {
    unsafe { DEBUG_MODE }
}

pub fn set_debug_mode(mode: DebugMode) {
    unsafe { DEBUG_MODE = mode }
}

fn set_audio_listener_transformations() {
    let camera_pos = render::get_camera_position();
    let camera_rot = render::get_camera_rotation();

    sound::set_listener_position(camera_pos);
    sound::set_listener_orientation(camera_rot);
}

fn set_delta_time(dt: Duration) {
    unsafe {
        DELTA_TIME = dt;
    }
}

pub fn set_global_system_value(key: &str, value: SystemValue) {
    unsafe {
        if let Some(hashmap_val) = SYSTEM_GLOBALS.get_mut(key) {
            *hashmap_val = value;
        } else {
            SYSTEM_GLOBALS.insert(key.into(), value);
        }
    }
}

pub fn get_global_system_value(key: &str) -> Option<SystemValue> {
    unsafe {
        match SYSTEM_GLOBALS.get(key) {
            Some(value) => Some(value.clone()),
            None => None
        }
    }
}

pub fn get_delta_time() -> Duration {
    unsafe { DELTA_TIME }
}

#[derive(Clone, Copy)]
pub enum DebugMode {
    None,
    ShowFps,
    Full,
}

pub struct GameSettings {
    master_volume: u8,
    shadowmap_size: u32,
}
