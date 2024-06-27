use crate::{
    assets::{model_asset::ModelAsset, texture_asset::TextureAsset},
    game::game_main,
    managers::{
        self,
        assets::{get_full_asset_path, AssetManager},
        debugger,
        input::{self, InputManager},
        navigation::NavigationManager,
        networking,
        physics::{self, CollisionGroups, PhysicsManager},
        render::{CurrentCascade, RenderManager},
        saves::SavesManager,
        sound::set_listener_transform,
        systems::{self, SystemValue},
    },
    objects::character_controller::CharacterController,
};
use egui_glium::egui_winit::egui::{self, FontData, FontDefinitions, FontFamily, Id, Window};
use ez_al::EzAl;
use glam::Vec2;
use glium::{
    glutin::{
        context::NotCurrentGlContext,
        display::{GetGlDisplay, GlDisplay},
    },
    Display,
};
use glutin::surface::GlSurface;
use once_cell::sync::Lazy;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::{
    collections::HashMap,
    fs,
    num::NonZeroU32,
    time::{Duration, Instant},
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, MouseButton, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder},
    keyboard::KeyCode,
    window::{CursorGrabMode, WindowBuilder},
};

static FONT: Lazy<Vec<u8>> =
    Lazy::new(|| fs::read(get_full_asset_path("fonts/JetBrainsMono-Regular.ttf")).unwrap());

pub fn start_game_with_render(debug_mode: DebugMode) {
    let event_loop: EventLoop<_> = EventLoopBuilder::new()
        .build()
        .expect("Event loop building failed");
    let (window, display) = new_window(&event_loop);

    let mut egui_glium =
        egui_glium::EguiGlium::new(egui::ViewportId(Id::new(0)), &display, &window, &event_loop);

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

    let al = EzAl::new().ok();
    let mut framework = Framework {
        debug_mode,
        delta_time: Duration::default(),
        system_globals: HashMap::new(),
        resolution: Vec2::new(1280.0, 720.0),

        al,
        input: InputManager::default(),
        navigation: NavigationManager::default(),
        physics: PhysicsManager::default(),
        saves: SavesManager::default(),
        assets: AssetManager::default(),
        render: Some(RenderManager::new(display)),
    };
    framework.set_debug_mode(debug_mode);

    //render::init(&display);

    framework.navigation.update();
    game_main::start(&mut framework);

    event_loop
        .run(move |ev, window_target| {
            dbg!(window_target.raw_display_handle());
            match ev {
                Event::AboutToWait => {
                    window.request_redraw();
                }
                Event::DeviceEvent {
                    device_id: _,
                    event,
                } => framework.input.reg_device_event(&event),
                Event::WindowEvent {
                    window_id: _,
                    event,
                } => {
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
                                                managers::ui::draw_inspector(
                                                    &mut framework,
                                                    ui,
                                                    &fps,
                                                    &mut ui_state,
                                                );
                                            });
                                        }
                                    }

                                    systems::ui_render(ctx);
                                });

                                {
                                    let render = framework.render.as_mut().unwrap();

                                    set_listener_transform(
                                        framework.al.as_ref().unwrap(),
                                        render.get_camera_position(),
                                        render.get_camera_front(),
                                    );

                                    render.draw();
                                }

                                println!("render!");
                                systems::shadow_render(
                                    framework.render.as_mut().unwrap(),
                                    &framework.assets,
                                    &CurrentCascade::Closest,
                                );
                                systems::shadow_render(
                                    framework.render.as_mut().unwrap(),
                                    &framework.assets,
                                    &CurrentCascade::Furthest,
                                );
                                systems::render(&mut framework); // Don't mind me, beautiful Rust code going on here

                                {
                                    let render = framework.render.as_mut().unwrap();
                                    render.debug_draw();
                                    egui_glium
                                        .paint(&render.display, render.target.as_mut().unwrap());
                                    render.finish_render();
                                }

                                frames_count += 1;
                            }
                            WindowEvent::CloseRequested => {
                                window_target.exit();
                                networking::disconnect();
                                return;
                            }
                            WindowEvent::Resized(size) => {
                                framework.resolution =
                                    Vec2::new(size.width as f32, size.height as f32);
                                framework
                                    .render
                                    .as_mut()
                                    .unwrap()
                                    .display
                                    .resize(size.into());
                                let _ = window.request_inner_size(size);

                                framework.render.as_mut().unwrap().aspect_ratio =
                                    size.width as f32 / size.height as f32;
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
        })
        .unwrap();
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
        render: None,
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

    if let Some(render) = &mut framework.render {
        render.update();
    }
    framework.physics.update();
    networking::update(delta_time);
    framework.navigation.update();
    game_main::update(framework);
    systems::update(framework);
    // Render to the closest cascade
    systems::render(framework);
    // Render to the furthest cascade
    systems::render(framework);
    // Render normally
    systems::render(framework);
    framework.navigation.create_grids();
    if let Some(render) = &mut framework.render {
        render.finish_render();
    }

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

// Glium's SimpleWindowBuilder's build function with a few changes
// https://github.com/glium/glium/blob/master/src/backend/glutin/mod.rs#L351
fn new_window<T>(
    event_loop: &winit::event_loop::EventLoop<T>,
) -> (
    winit::window::Window,
    Display<glutin::surface::WindowSurface>,
) {
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
    let attrs = glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
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
    let context_attributes =
        glutin::context::ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
    let current_context = Some(unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &context_attributes)
            .expect("failed to create context")
    })
    .unwrap()
    .make_current(&surface)
    .unwrap();
    surface
        .set_swap_interval(&current_context, glutin::surface::SwapInterval::DontWait)
        .unwrap();
    let display = Display::from_context_surface(current_context, surface).unwrap();

    (window, display)
}

pub struct Framework {
    pub debug_mode: DebugMode,
    pub delta_time: Duration,
    pub system_globals: HashMap<String, Vec<SystemValue>>,
    pub resolution: Vec2,

    pub al: Option<EzAl>,
    pub input: InputManager, // done
    pub navigation: NavigationManager,
    pub physics: PhysicsManager,
    pub saves: SavesManager, // done
    pub assets: AssetManager,
    pub render: Option<RenderManager>,
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
            None => None,
        }
    }

    pub fn new_character_controller_object(
        &mut self,
        name: &str,
        shape: physics::BodyColliderType,
        membership_groups: Option<CollisionGroups>,
        mask: Option<CollisionGroups>,
    ) -> CharacterController {
        CharacterController::new(&mut self.physics, name, shape, membership_groups, mask)
    }

    // SavesManager
    pub fn load_save(&mut self, save_name: &str) -> Result<(), ()> {
        match self.saves.load_save(save_name) {
            Ok(save_values) => {
                for (key, value) in save_values {
                    self.set_global_system_value(&key, value);
                }
            }
            Err(_) => {
                debugger::error("Framework error!\nFailed to load save.");
                return Err(());
            }
        }

        Ok(())
    }

    pub fn register_save_value(&mut self, system_value_name: &str) {
        self.saves.register_save_value(system_value_name)
    }

    pub fn unregister_save_value(&mut self, system_value_name: &str) {
        self.saves.unregister_save_value(system_value_name)
    }

    pub fn new_save(&mut self, save_name: &str) -> Result<(), std::io::Error> {
        self.saves.new_save(save_name, &self.system_globals)
    }

    pub fn save_game(&mut self) {
        self.saves.save_game(&self.system_globals)
    }

    // InputManager
    pub fn new_bind_keyboard(&mut self, bind_name: &str, keys: Vec<&str>) {
        let mut input_event_types = Vec::new();
        for key in keys {
            match serde_json::from_str::<KeyCode>(&key.replace("\"", "")) {
                Ok(key) => input_event_types.push(input::InputEventType::Key(key)),
                Err(error) => debugger::error(
                    &format!(
                        "Framework's new_bind_keyboard error!\nFailed to deserialize a keycode from a string\nSee all avaliable keycodes here: {}\nserde_json error: {}",
                        "https://docs.rs/winit/0.29.10/i686-pc-windows-msvc/winit/keyboard/enum.KeyCode.html",
                        error
                    )
                ),
            }
        }
        self.input.new_bind(bind_name, input_event_types);
    }

    pub fn new_bind_mouse(&mut self, bind_name: &str, buttons: Vec<&str>) {
        let mut input_event_types = Vec::new();
        for button in buttons {
            match serde_json::from_str::<MouseButton>(&button.replace("\"", "")) {
                Ok(key) => input_event_types.push(input::InputEventType::Mouse(key)),
                Err(error) => debugger::error(
                    &format!(
                        "Framework's new_bind_mouse error!\nFailed to deserialize a mouse button from a string\nSee all avaliable values here: {}\nserde_json error: {}",
                        "https://docs.rs/winit/0.29.10/i686-pc-windows-msvc/winit/event/enum.MouseButton.html",
                        error
                    )
                ),
            }
        }
        self.input.new_bind(bind_name, input_event_types);
    }

    pub fn is_bind_pressed(&self, bind_name: &str) -> bool {
        self.input.is_bind_pressed(bind_name)
    }

    pub fn is_bind_down(&self, bind_name: &str) -> bool {
        self.input.is_bind_down(bind_name)
    }

    pub fn is_bind_released(&self, bind_name: &str) -> bool {
        self.input.is_bind_released(bind_name)
    }

    pub fn mouse_position_from_center(&self) -> Vec2 {
        self.input.mouse_position_from_center()
    }

    pub fn mouse_delta(&self) -> Vec2 {
        self.input.mouse_delta()
    }

    pub fn is_mouse_locked(&self) -> bool {
        self.input.is_mouse_locked()
    }

    pub fn set_mouse_locked(&mut self, lock: bool) {
        self.input.set_mouse_locked(lock)
    }

    pub fn preload_model_asset(&mut self, asset_id: String, gltf_path: &str) -> Result<(), ()> {
        ModelAsset::preload_model_asset_from_gltf(self, &asset_id, gltf_path)
    }

    pub fn preload_texture_asset(
        &mut self,
        asset_id: String,
        texture_path: &str,
    ) -> Result<(), ()> {
        TextureAsset::preload_texture_asset(self, asset_id, texture_path)
    }
}
