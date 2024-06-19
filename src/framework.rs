use ez_al::EzAl;
use glutin::surface::GlSurface;
use raw_window_handle::HasRawWindowHandle;
use crate::{
    game::game_main,
    managers::{
        self, assets::{get_full_asset_path, AssetManager}, input::InputManager, navigation::{self, NavigationManager}, networking, physics::{self, CollisionGroups, PhysicsManager}, render::{self, ShadowTextures}, saves::SavesManager, sound::{self, set_listener_transform}, systems::{self, SystemValue}
    }, objects::{camera_position::CameraPosition, character_controller::CharacterController},
};
use egui_glium::egui_winit::egui::{self, FontData, FontDefinitions, FontFamily, Id, Window};
use glam::Vec2;
use glium::{glutin::{context::NotCurrentGlContext, display::{GetGlDisplay, GlDisplay}}, Display};
use once_cell::sync::Lazy;
use winit::{dpi::PhysicalSize, event::{Event, WindowEvent}, event_loop::{EventLoop, EventLoopBuilder}, window::{CursorGrabMode, WindowBuilder}};
use std::{
    collections::HashMap, fs, num::NonZeroU32, time::{Duration, Instant}
};

static FONT: Lazy<Vec<u8>> =
    Lazy::new(|| fs::read(get_full_asset_path("fonts/JetBrainsMono-Regular.ttf")).unwrap());

pub fn start_game_with_render(debug_mode: DebugMode) {
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

    let al = EzAl::new().unwrap();
    let mut framework = Framework {
        debug_mode,
        delta_time: Duration::default(),
        system_globals: HashMap::new(),
        resolution: Vec2::new(1280.0, 720.0),

        al: Some(al),
        input: InputManager::default(),
        navigation: NavigationManager::default(),
        physics: PhysicsManager::default(),
        saves: SavesManager::default(),
        assets: AssetManager::default(),
    };
    framework.set_debug_mode(debug_mode);

    render::init(&display);

    framework.navigation.update();
    game_main::start(&mut framework);

    let mut win_w = window.inner_size().width;
    let mut win_h = window.inner_size().height;

    let shadow_textures = ShadowTextures::new(&display, 4096, 4096);

    event_loop.run(move |ev, window_target| {
        match ev {
            Event::AboutToWait => {
                window.request_redraw();
            },
            Event::DeviceEvent { device_id: _, event } => framework.input.reg_device_event(&event),
            Event::WindowEvent { window_id: _, event } => {
                let event_response = egui_glium.on_event(&window, &event);
                if event_response.consumed == false {
                    framework.input.reg_event(&event);

                    match event {
                        WindowEvent::RedrawRequested => {
                            let time_since_last_frame = last_frame.elapsed();
                            last_frame = Instant::now();
                            update_game(&mut framework, time_since_last_frame);

                            if framework.input.is_mouse_locked() {
                                let _ = window.set_cursor_grab(CursorGrabMode::Locked);
                            } else {
                                let _ = window.set_cursor_grab(CursorGrabMode::None);
                            }

                            egui_glium.run(&window, |ctx| {
                                match framework.debug_mode() {
                                    DebugMode::None => (),
                                    _ => {
                                        Window::new("inspector").show(ctx, |ui| {
                                            managers::ui::draw_inspector(&mut framework, ui, &fps, &mut ui_state);
                                        });
                                    }
                                }

                                systems::ui_render(ctx);
                            });

                            set_listener_transform(
                                &framework.al.as_ref().unwrap(),
                                render::get_camera_position(),
                                render::get_camera_front(),
                            );

                            let mut target = display.draw();

                            render::draw(&display, &mut target, &shadow_textures, &mut framework);
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

    let mut framework = Framework {
        debug_mode: DebugMode::None,
        delta_time: Duration::default(),
        system_globals: HashMap::new(),
        resolution: Vec2::new(0.0, 0.0),

        al: None,
        input: InputManager::default(),
        navigation: NavigationManager::default(),
        physics: PhysicsManager::default(),
        saves: SavesManager::default(),
        assets: AssetManager::default(),
    };


    game_main::start(&mut framework);

    let tickrate_tick = Duration::from_millis(16);
    let clock = chron::Clock::new(NonZeroU32::new(60).unwrap());

    for tick in clock {
        match tick {
            chron::clock::Tick::Update => {
                update_game(&mut framework, tickrate_tick);
            }
            chron::clock::Tick::Render { interpolation: _ } => {}
        }
    }
}

fn update_game(framework: &mut Framework, delta_time: Duration) {
    framework.delta_time = delta_time;

    render::update();
    framework.physics.update();
    networking::update(delta_time);
    framework.navigation.update();
    game_main::update(framework);
    systems::update(framework);
    framework.navigation.create_grids();

    framework.input.update();
}

fn get_fps(now: &Instant, frames: &usize) -> Option<usize> {
    let one_second = std::time::Duration::new(1, 0);

    if now.elapsed() > one_second {
        return Some(frames.clone());
    }
    None
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

pub struct Framework {
    pub debug_mode: DebugMode,
    pub delta_time: Duration,
    pub system_globals: HashMap<String, Vec<SystemValue>>,
    pub resolution: Vec2,

    pub al: Option<EzAl>,
    pub input: InputManager,
    pub navigation: NavigationManager,
    pub physics: PhysicsManager,
    pub saves: SavesManager,
    pub assets: AssetManager,
}

impl Framework {
    pub fn debug_mode(&self) -> DebugMode {
        self.debug_mode
    }

    pub fn set_debug_mode(&mut self, debug_mode: DebugMode) {
        self.debug_mode = debug_mode
    }

    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    pub fn new_camera_position_object(name: &str) -> CameraPosition {
        CameraPosition::new(name)
    }

    pub fn set_global_system_value(&mut self, key: &str, value: Vec<SystemValue>) {
        if let Some(hashmap_val) = self.system_globals.get_mut(key) {
            *hashmap_val = value;
        } else {
            self.system_globals.insert(key.into(), value);
        }
    }

    pub fn get_global_system_value(&self, key: &str) -> Option<Vec<SystemValue>> {
        match self.system_globals.get(key) {
            Some(value) => Some(value.clone()),
            None => None
        }
    }

    pub fn new_character_controller_object(
        &mut self,
        name: &str, 
        shape: physics::BodyColliderType, 
        membership_groups: Option<CollisionGroups>, 
        mask: Option<CollisionGroups>
    ) -> CharacterController {
        CharacterController::new(&mut self.physics, name, shape, membership_groups, mask)
    }
}
