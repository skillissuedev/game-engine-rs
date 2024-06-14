use glutin::surface::GlSurface;
use raw_window_handle::HasRawWindowHandle;
use crate::{
    game::game_main,
    managers::{
        self,
        assets::get_full_asset_path,
        input, navigation,
        networking,
        physics,
        render::{self, ShadowTextures},
        sound::{self, set_listener_transform},
        systems::{self, SystemValue},
    },
};
use egui_glium::egui_winit::egui::{self, FontData, FontDefinitions, FontFamily, Id, Window};
use glam::Vec2;
use glium::{glutin::{context::NotCurrentGlContext, display::{GetGlDisplay, GlDisplay}}, Display};
use once_cell::sync::Lazy;
use winit::{dpi::PhysicalSize, event::{Event, WindowEvent}, event_loop::{EventLoop, EventLoopBuilder}, window::{CursorGrabMode, WindowBuilder}};
use std::{
    collections::HashMap, fs, num::NonZeroU32, time::{Duration, Instant}
};

static mut DEBUG_MODE: DebugMode = DebugMode::None;
static mut DELTA_TIME: Duration = Duration::new(0, 0);
static FONT: Lazy<Vec<u8>> =
    Lazy::new(|| fs::read(get_full_asset_path("fonts/JetBrainsMono-Regular.ttf")).unwrap());
static mut SYSTEM_GLOBALS: Lazy<HashMap<String, Vec<SystemValue>>> = Lazy::new(|| HashMap::new());
static mut SCREEN_RESOLUTION: Vec2 = Vec2::new(1280.0, 720.0);

pub fn start_game_with_render(debug_mode: DebugMode) {
    unsafe { DEBUG_MODE = debug_mode }
    /*let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("projectbaldej")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_transparent(false);
    let cb = ContextBuilder::new().with_multisampling(4).with_srgb(true);//.with_vsync(true);
    let display = Display::new(wb, cb, &event_loop).expect("failed to create glium display");
    let mut egui_glium = egui_glium::EguiGlium::new(&display, &event_loop);*/
    /*let event_loop = EventLoop::builder().build().expect("Failed to create an event loop");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);*/
    /*let event_loop: EventLoop<_> = EventLoopBuilder::new()
        .build()
        .expect("Event loop building failed");
    let (window, display) = SimpleWindowBuilder::new()
        .with_title("projectbaldej")
        .with_inner_size(2560, 1080)
        .build(&event_loop);
    window.set_resizable(true);
    window.set_transparent(false);*/
    let event_loop: EventLoop<_> = EventLoopBuilder::new()
        .build()
        .expect("Event loop building failed");
    let (window, display) = new_window(&event_loop);

    let mut egui_glium = egui_glium::EguiGlium::new(egui::ViewportId(Id::new(0)), &display, &window, &event_loop);

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
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "JetBrains Mono".into());
    egui_glium.egui_ctx().set_fonts(fonts);
    let mut ui_state = managers::ui::UiState::default();

    let mut frames_count: usize = 0;
    let mut now = std::time::Instant::now();
    let mut last_frame = std::time::Instant::now();

    sound::init().unwrap();
    render::init(&display);

    navigation::update();
    game_main::start();

    let mut win_w = window.inner_size().width;
    let mut win_h = window.inner_size().height;

    let shadow_textures = ShadowTextures::new(&display, 4096, 4096);

    event_loop.run(move |ev, window_target| {
        match ev {
            Event::AboutToWait => {
                window.request_redraw();
            },
            Event::DeviceEvent { device_id: _, event } => input::reg_device_event(&event),
            Event::WindowEvent { window_id: _, event } => {
                let event_response = egui_glium.on_event(&window, &event);
                if event_response.consumed == false {
                    input::reg_event(&event);
                    match event {
                        WindowEvent::RedrawRequested => {
                            let time_since_last_frame = last_frame.elapsed();
                            last_frame = Instant::now();
                            update_game(time_since_last_frame);

                            if input::is_mouse_locked() {
                                let _ = window.set_cursor_grab(CursorGrabMode::Locked);
                            } else {
                                let _ = window.set_cursor_grab(CursorGrabMode::None);
                            }

                            egui_glium.run(&window, |ctx| {
                                match get_debug_mode() {
                                    DebugMode::None => (),
                                    _ => {
                                        Window::new("inspector").show(ctx, |ui| {
                                            managers::ui::draw_inspector(ui, &fps, &mut ui_state);
                                        });
                                    }
                                }

                                systems::ui_render(ctx);
                            });

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
                        },
                        WindowEvent::CloseRequested => {
                            window_target.exit();
                            networking::disconnect();
                            return;
                        }
                        WindowEvent::Resized(size) => {
                            win_w = size.width;
                            win_h = size.height;
                            display.resize(size.into());
                            //window.request_inner_size(size);

                            unsafe {
                                SCREEN_RESOLUTION = Vec2::new(win_w as f32, win_h as f32);
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
            window.set_title(&format!("projectbaldej: {fps} fps"));
            frames_count = 0;
            now = Instant::now();
        }
    }).unwrap();
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
    render::update();
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
pub fn get_resolution() -> Vec2 {
    unsafe { SCREEN_RESOLUTION }
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

pub fn set_global_system_value(key: &str, value: Vec<SystemValue>) {
    unsafe {
        if let Some(hashmap_val) = SYSTEM_GLOBALS.get_mut(key) {
            *hashmap_val = value;
        } else {
            SYSTEM_GLOBALS.insert(key.into(), value);
        }
    }
}

pub fn get_global_system_value(key: &str) -> Option<Vec<SystemValue>> {
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

// Glium's SimpleWindowBuilder's build function with a few changes 
// https://github.com/glium/glium/blob/master/src/backend/glutin/mod.rs#L351
fn new_window<T>(event_loop: &winit::event_loop::EventLoop<T>) -> (winit::window::Window, Display<glutin::surface::WindowSurface>) {
    // First we start by opening a new Window
    let window_builder = WindowBuilder::new()
        .with_title("projectbaldej")
        .with_inner_size(PhysicalSize::new(1280, 720));
    let display_builder =
        glutin_winit::DisplayBuilder::new().with_window_builder(Some(window_builder));
    let config_template_builder = glutin::config::ConfigTemplateBuilder::new()
        .with_multisampling(4)
        //.with_swap_interval(Some(0), Some(0))
        .with_single_buffering(true);
    let (window, gl_config) = display_builder
        .build(&event_loop, config_template_builder, |mut configs| {
            // Just use the first configuration since we don't have any special preferences here
            configs.next().unwrap()
        })
    .unwrap();
    let window = window.unwrap();

    // Now we get the window size to use as the initial size of the Surface
    let (width, height): (u32, u32) = window.inner_size().into();
    let attrs =
        glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
        .build(
            window.raw_window_handle(),
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

    // Finally we can create a Surface, use it to make a PossiblyCurrentContext and create the glium Display
    let surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };
    let context_attributes = glutin::context::ContextAttributesBuilder::new()
        .build(Some(window.raw_window_handle()));
    let current_context = Some(unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &context_attributes)
            .expect("failed to create context")
    })
    .unwrap()
        .make_current(&surface)
        .unwrap();
    surface.set_swap_interval(&current_context, glutin::surface::SwapInterval::DontWait).unwrap();
    let display = Display::from_context_surface(current_context, surface).unwrap();

    (window, display)
}
