use crate::{
    assets::{model_asset::ModelAsset, shader_asset::ShaderAsset, sound_asset::SoundAsset, texture_asset::TextureAsset},
    game::game_main,
    managers::{
        self, assets::{get_full_asset_path, AssetManager, ModelAssetId, SoundAssetId, TextureAssetId}, debugger, input::{self, InputManager}, navigation::NavigationManager, networking, physics::{self, BodyColliderType, CollisionGroups, PhysicsManager}, render::{RenderLayer, RenderManager}, saves::SavesManager, sound::set_listener_transform, systems::{self, SystemValue}, ui::UiManager
    },
    objects::{character_controller::CharacterController, empty_object::EmptyObject, instanced_model_object::InstancedModelObject, instanced_model_transform_holder::InstancedModelTransformHolder, master_instanced_model_object::MasterInstancedModelObject, model_object::ModelObject, nav_obstacle::NavObstacle, navmesh::NavigationGround, ray::Ray, sound_emitter::SoundEmitter, trigger::Trigger, Transform}, Args,
};
use egui_glium::egui_winit::egui::{self, FontData, FontDefinitions, FontFamily, Id, Window};
use ez_al::{EzAl, SoundSourceType};
use glam::{Vec2, Vec3};
use glium::{
    backend::glutin::SimpleWindowBuilder, glutin, Display
};
use once_cell::sync::Lazy;
use winit::{event::ElementState, keyboard::PhysicalKey, window::Fullscreen};
use std::{
    collections::HashMap, fs, num::NonZeroU32, time::{Duration, Instant}
};
use glium::winit::{
    event::{Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::KeyCode,
    window::CursorGrabMode,
};

pub static mut FRAMEWORK_POINTER: usize = 0;
static FONT: Lazy<Vec<u8>> =
    Lazy::new(|| fs::read(get_full_asset_path("fonts/JetBrainsMono-Regular.ttf")).unwrap());

pub fn start_game_with_render(args: Args, debug_mode: DebugMode) {
    let event_loop: EventLoop<_> = EventLoop::builder()
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
        ui: Some(UiManager::default())
    };
    framework.set_debug_mode(debug_mode);

    framework.navigation.update();
    unsafe {
        let ptr = &mut framework as *mut Framework;
        FRAMEWORK_POINTER = ptr as usize;
    };

    game_main::start(args, &mut framework);

    #[allow(deprecated)]
    event_loop
        .run(move |ev, window_target| {
            unsafe {
                let ptr = &mut framework as *mut Framework;
                FRAMEWORK_POINTER = ptr as usize;
            };

            match ev {
                Event::AboutToWait => window.request_redraw(),
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
                                    framework.ui.as_mut().unwrap().render(ctx, framework.debug_mode);
                                });

                                systems::render(&mut framework); // Don't mind me, "beautiful" Rust code going on here

                                {
                                    let render = framework.render.as_mut().unwrap();

                                    set_listener_transform(
                                        framework.al.as_ref().unwrap(),
                                        render.camera.translation,
                                        render.camera.front(),
                                    );

                                    render.render_scene(&framework.assets, &mut egui_glium);
                                }

                                frames_count += 1;
                                framework.input.update();
                            },
                            WindowEvent::CloseRequested => {
                                window_target.exit();
                                networking::disconnect();
                                return;
                            },
                            WindowEvent::Resized(size) => {
                                framework.render.as_mut()
                                    .expect("Failed to resize the Display\nFailed to get the render manager")
                                    .display.resize(size.into());
                                framework.resolution =
                                    Vec2::new(size.width as f32, size.height as f32);
                                framework.render.as_mut().expect("No RenderManager! - Framework (WindowEvent::Resized)")
                                    .resize(size.into());
                                let _ = window.request_inner_size(size);
                            },
                            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                                // fullscreen on f11
                                if let ElementState::Released = event.state {
                                    if let PhysicalKey::Code(KeyCode::F11) = event.physical_key {
                                        if let Some(_) = window.fullscreen() {
                                            window.set_fullscreen(None);
                                        } else {
                                            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                                        }
                                    }
                                }
                            },
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

pub fn start_game_without_render(args: Args) {
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
        ui: None
    };

    unsafe {
        let ptr = &mut framework as *mut Framework;
        FRAMEWORK_POINTER = ptr as usize;
    };

    game_main::start(args, &mut framework);

    let tickrate_tick = Duration::from_millis(16);
    let clock = chron::Clock::new(NonZeroU32::new(60).unwrap());

    for tick in clock {
        match tick {
            chron::clock::Tick::Update => {
                unsafe {
                    let ptr = &mut framework as *mut Framework;
                    FRAMEWORK_POINTER = ptr as usize;
                };
                update_game(&mut framework, tickrate_tick);
            }
            chron::clock::Tick::Render { interpolation: _ } => {}
        }
    }
}

fn update_game(framework: &mut Framework, delta_time: Duration) {
    framework.delta_time = delta_time;
    framework.physics.update();
    networking::update(delta_time);
    framework.navigation.update();
    game_main::update(framework);
    systems::update(framework);
}

fn get_fps(now: &Instant, frames: &usize) -> Option<usize> {
    let one_second = std::time::Duration::new(1, 0);

    if now.elapsed() > one_second {
        return Some(frames.clone());
    }
    None
}

#[derive(Clone, Copy, Debug)]
pub enum DebugMode {
    None,
    ShowFps,
    Full,
}

// TODO: fix for Windows (?)
fn new_window<T>(
    event_loop: &glium::winit::event_loop::EventLoop<T>,
) -> (
    glium::winit::window::Window,
    Display<glutin::surface::WindowSurface>,
) {
    let (window, display) = SimpleWindowBuilder::new().build(event_loop);

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
    pub(crate) render: Option<RenderManager>,
    pub ui: Option<UiManager>,
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

    pub fn remove_global_system_value(&mut self, key: &str) {
        self.system_globals.remove(key);
    }

    // render
    pub fn set_camera_position(&mut self, pos: Vec3) {
        match self.render.as_mut() {
            Some(render) => render.set_camera_position(pos),
            None => debugger::error("Failed to call set_camera_position, render is None (probably running a server!)"),
        }
    }

    pub fn set_camera_rotation(&mut self, rot: Vec3) {
        match self.render.as_mut() {
            Some(render) => render.set_camera_rotation(rot),
            None => debugger::error("Failed to call set_camera_rotation, render is None (probably running a server!)"),
        }
    }

    pub fn set_camera_fov(&mut self, fov: f32) {
        todo!();
        /*match self.render.as_mut() {
            Some(render) => render.set_camera_fov(fov),
            None => debugger::error("Failed to call set_camera_fov, render is None (probably running a server!)"),
        }*/
    }

    pub fn set_light_direction(&mut self, dir: Vec3) {
        todo!();
        /*match self.render.as_mut() {
            Some(render) => render.set_light_direction(dir),
            None => debugger::error("Failed to call set_light_direction, render is None (probably running a server!)"),
        }*/
    }

    pub fn get_light_direction(&self) -> Option<Vec3> {
        todo!();
        /*match self.render.as_ref() {
            Some(render) => Some(render.get_light_direction()),
            None => {
                debugger::error("Failed to call get_light_direction, render is None (probably running a server!)");
                None
            },
        }*/
    }

    pub fn get_camera_position(&self) -> Option<Vec3> {
        match self.render.as_ref() {
            Some(render) => Some(render.camera_position()),
            None => {
                debugger::error("Failed to call get_camera_position, render is None (probably running a server!)");
                None
            },
        }
    }

    pub fn get_camera_front(&mut self) -> Option<Vec3> {
        match self.render.as_mut() {
            Some(render) => Some(render.camera_front()),
            None => {
                debugger::error("Failed to call get_camera_front, render is None (probably running a server!)");
                None
            },
        }
    }

    pub fn get_camera_left(&self) -> Option<Vec3> {
        match self.render.as_ref() {
            Some(render) => Some(render.camera_left()),
            None => {
                debugger::error("Failed to call get_camera_left, render is None (probably running a server!)");
                None
            },
        }
    }

    pub fn get_camera_rotation(&self) -> Option<Vec3> {
        match self.render.as_ref() {
            Some(render) => Some(render.camera_rotation()),
            None => {
                debugger::error("Failed to call get_camera_rotation, render is None (probably running a server!)");
                None
            },
        }
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

    pub fn save_lazy_value(&mut self, value_name: &str, value: Vec<SystemValue>) {
        self.saves.save_lazy_value(value_name, value)
    }

    pub fn load_lazy_value(&mut self, value_name: &str) -> Option<Vec<SystemValue>> {
        self.saves.load_lazy_value(value_name)
    }

    // InputManager
    pub fn new_bind_keyboard(&mut self, bind_name: &str, keys: Vec<&str>) {
        let mut input_event_types = Vec::new();
        for key in keys {
            match serde_json::from_str::<KeyCode>(&format!("\"{}\"", key)) {
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
            match serde_json::from_str::<MouseButton>(&format!("\"{}\"", button)) {
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

    pub fn mouse_position(&self) -> Vec2 {
        self.input.mouse_position()
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

    // AssetManager
    pub fn preload_model_asset(&mut self, asset_id: String, gltf_path: &str) -> Result<(), ()> {
        ModelAsset::preload_model_asset_from_gltf(self, asset_id, gltf_path)
    }

    pub fn preload_sound_asset(&mut self, asset_id: String, wav_path: &str) -> Result<(), ()> {
        SoundAsset::preload_sound_asset_from_wav(self, asset_id, wav_path)
    }

    pub fn preload_texture_asset(&mut self, asset_id: String, texture_path: &str) -> Result<(), ()> {
        TextureAsset::preload_texture_asset(self, asset_id, texture_path)
    }

    pub fn get_model_asset(&self, asset_id: &str) -> Option<ModelAssetId> {
        self.assets.get_model_asset_id(asset_id)
    }

    pub fn get_texture_asset(&self, asset_id: &str) -> Option<TextureAssetId> {
        self.assets.get_texture_asset_id(asset_id)
    }

    pub fn get_sound_asset(&self, asset_id: &str) -> Option<SoundAssetId> {
        self.assets.get_sound_asset_id(asset_id)
    }

    // UI
    pub fn show_title_bar(&mut self, window_id: &str, show: bool) {
        match &mut self.ui {
            Some(ui) => ui.show_title_bar(window_id, show),
            None => {
                debugger::error("Framework error!\nCan't use UI (show_title_bar) while running server");
            },
        }
    }

    pub fn show_close_button(&mut self, window_id: &str, show: bool) {
        match &mut self.ui {
            Some(ui) => ui.show_close_button(window_id, show),
            None => {
                debugger::error("Framework error!\nCan't use UI (show_close_button) while running server");
            },
        }
    }

    pub fn is_widget_double_clicked(&mut self, window_id: &str, widget_id: &str) -> bool {
        match &mut self.ui {
            Some(ui) => ui.is_widget_double_clicked(window_id, widget_id),
            None => {
                debugger::error("Framework error!\nCan't use UI (is_widget_double_clicked) while running server");
                false
            },
        }
    }

    pub fn get_widget_numeric_value(&mut self, window_id: &str, widget_id: &str) -> Option<f32> {
        match &mut self.ui {
            Some(ui) => ui.get_widget_numeric_value(window_id, widget_id),
            None => {
                debugger::error("Framework error!\nCan't use UI (get_widget_numeric_value) while running server");
                None
            },
        }
    }

    pub fn add_singleline_text_edit(&mut self, window_id: &str, widget_id: &str, contents: &str, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_singleline_text_edit(window_id, widget_id, contents, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_singleline_text_edit) while running server");
            },
        }
    }

    pub fn is_widget_right_clicked(&mut self, window_id: &str, widget_id: &str) -> bool {
        match &mut self.ui {
            Some(ui) => ui.is_widget_right_clicked(window_id, widget_id),
            None => {
                debugger::error("Framework error!\nCan't use UI (is_widget_right_clicked) while running server");
                false
            },
        }
    }

    pub fn add_multiline_text_edit(&mut self, window_id: &str, widget_id: &str, contents: &str, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_multiline_text_edit(window_id, widget_id, contents, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_singleline_text_edit) while running server");
            },
        }
    }

    pub fn add_progress_bar(&mut self, window_id: &str, widget_id: &str, contents: f32, text: String, text_size: Option<f32>, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_progress_bar(window_id, widget_id, contents, text, text_size, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_progress_bar) while running server");
            },
        }
    }

    pub fn add_float_slider(&mut self, window_id: &str, widget_id: &str, value: f32, min: f32, max: f32, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_float_slider(window_id, widget_id, value, min, max, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_float_slider) while running server");
            },
        }
    }

    pub fn add_int_slider(&mut self, window_id: &str, widget_id: &str, value: i32, min: i32, max: i32, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_int_slider(window_id, widget_id, value, min, max, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_int_slider) while running server");
            },
        }
    }

    pub fn add_checkbox(&mut self, window_id: &str, widget_id: &str, value: bool, title: &str, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_checkbox(window_id, widget_id, value, title, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_checkbox) while running server");
            },
        }
    }

    pub fn add_horizontal_scroll(&mut self, window_id: &str, widget_id: &str, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_horizontal_scroll(window_id, widget_id, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_horizontal_scroll) while running server");
            },
        }
    }

    pub fn add_vertical_scroll(&mut self, window_id: &str, widget_id: &str, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_vertical_scroll(window_id, widget_id, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_vertical_scroll) while running server");
            },
        }
    }

    pub fn add_scroll(&mut self, window_id: &str, widget_id: &str, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_scroll(window_id, widget_id, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_scroll) while running server");
            },
        }
    }

    pub fn add_horizontal(&mut self, window_id: &str, widget_id: &str, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_horizontal(window_id, widget_id, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_horizontal) while running server");
            },
        }
    }

    pub fn add_vertical(&mut self, window_id: &str, widget_id: &str, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_vertical(window_id, widget_id, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_vertical) while running server");
            },
        }
    }

    pub fn add_button(&mut self, window_id: &str, widget_id: &str, contents: &str, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_button(window_id, widget_id, contents, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_button) while running server");
            },
        }
    }

    pub fn add_image(&mut self, window_id: &str, widget_id: &str, image_path: &str, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_image(window_id, widget_id, image_path, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_image) while running server");
            },
        }
    }

    pub fn add_label(&mut self, window_id: &str, widget_id: &str, contents: &str, text_size: Option<f32>, size: Vec2, parent: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.add_label(window_id, widget_id, contents, text_size, size, parent),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_label) while running server");
            },
        }
    }

    pub fn set_window_position(&mut self, window_id: &str, position: Option<Vec2>) {
        match &mut self.ui {
            Some(ui) => ui.set_window_position(window_id, position),
            None => {
                debugger::error("Framework error!\nCan't use UI (set_window_position) while running server");
            },
        }
    }

    pub fn set_window_size(&mut self, window_id: &str, size: Option<Vec2>) {
        match &mut self.ui {
            Some(ui) => ui.set_window_size(window_id, size),
            None => {
                debugger::error("Framework error!\nCan't use UI (set_window_size) while running server");
            },
        }
    }

    pub fn add_theme(&mut self, theme_id: String, theme_json: String) {
        match &mut self.ui {
            Some(ui) => ui.add_theme(theme_id, theme_json),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_theme) while running server");
            },
        }
    }

    pub fn add_window_theme(&mut self, theme_id: String, theme_json: String) {
        match &mut self.ui {
            Some(ui) => ui.add_window_theme(theme_id, theme_json),
            None => {
                debugger::error("Framework error!\nCan't use UI (add_window_theme) while running server");
            },
        }
    }

    pub fn set_window_theme(&mut self, window_id: &str, theme: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.set_window_theme(window_id, theme),
            None => {
                debugger::error("Framework error!\nCan't use UI (set_window_theme) while running server");
            },
        }
    }
    
    pub fn set_widget_theme(&mut self, window_id: &str, widget_id: &str, theme: Option<&str>) {
        match &mut self.ui {
            Some(ui) => ui.set_widget_theme(window_id, widget_id, theme),
            None => {
                debugger::error("Framework error!\nCan't use UI (set_widget_theme) while running server");
            },
        }
    }

    pub fn set_widget_spacing(&mut self, window_id: &str, widget_id: &str, spacing: f32) {
        match &mut self.ui {
            Some(ui) => ui.set_widget_spacing(window_id, widget_id, spacing),
            None => {
                debugger::error("Framework error!\nCan't use UI (set_widget_spacing) while running server");
            },
        }
    }

    pub fn new_window(&mut self, window_id: &str, transparency: bool) {
        match &mut self.ui {
            Some(ui) => ui.new_window(window_id, transparency),
            None => {
                debugger::error("Framework error!\nCan't use UI (new_window) while running server");
            },
        }
    }

    pub fn set_window_on_top(&mut self, window_id: &str, show_on_top: bool) {
        match &mut self.ui {
            Some(ui) => ui.set_window_on_top(window_id, show_on_top),
            None => {
                debugger::error("Framework error!\nCan't use UI (set_window_on_top) while running server");
            },
        }
    }

    pub fn remove_window(&mut self, window_id: &str) {
        match &mut self.ui {
            Some(ui) => ui.remove_window(window_id),
            None => {
                debugger::error("Framework error!\nCan't use UI (remove_window) while running server");
            },
        }
    }

    pub fn remove_widget(&mut self, window_id: &str, widget_id: &str) {
        match &mut self.ui {
            Some(ui) => ui.remove_widget(window_id, widget_id),
            None => {
                debugger::error("Framework error!\nCan't use UI (remove_widget) while running server");
            },
        }
    }

    pub fn is_widget_hovered(&mut self, window_id: &str, widget_id: &str) -> bool {
        match &mut self.ui {
            Some(ui) => ui.is_widget_hovered(window_id, widget_id),
            None => {
                debugger::error("Framework error!\nCan't use UI (is_widget_hovered) while running server");
                false
            },
        }
    }

    pub fn is_widget_left_clicked(&mut self, window_id: &str, widget_id: &str) -> bool {
        match &mut self.ui {
            Some(ui) => ui.is_widget_left_clicked(window_id, widget_id),
            None => {
                debugger::error("Framework error!\nCan't use UI (is_widget_left_clicked) while running server");
                false
            },
        }
    }

    pub fn is_widget_dragged(&mut self, window_id: &str, widget_id: &str) -> bool {
        match &mut self.ui {
            Some(ui) => ui.is_widget_dragged(window_id, widget_id),
            None => {
                debugger::error("Framework error!\nCan't use UI (is_widget_dragged) while running server");
                false
            },
        }
    }

    pub fn is_widget_changed(&mut self, window_id: &str, widget_id: &str) -> bool {
        match &mut self.ui {
            Some(ui) => ui.is_widget_changed(window_id, widget_id),
            None => {
                debugger::error("Framework error!\nCan't use UI (is_widget_changed) while running server");
                false
            },
        }
    }


    // new objects
    pub fn new_character_controller_object(
        &mut self,
        name: &str,
        shape: physics::BodyColliderType,
        membership_groups: Option<CollisionGroups>,
        mask: Option<CollisionGroups>,
    ) -> CharacterController {
        CharacterController::new(&mut self.physics, name, shape, membership_groups, mask)
    }

    pub fn new_empty_object(
        &mut self,
        name: &str,
    ) -> EmptyObject {
        EmptyObject::new(name)
    }

    pub fn new_instanced_model_object(
        &mut self,
        name: &str,
        instance: &str,
    ) -> InstancedModelObject {
        InstancedModelObject::new(name, instance.into())
    }

    pub fn new_master_instanced_model_object(
        &mut self,
        name: &str,
        model_asset_id: ModelAssetId,
        texture_asset_id: Option<TextureAssetId>,
        shader_asset: ShaderAsset,
        is_transparent: bool,
        layer: RenderLayer,
    ) -> MasterInstancedModelObject {
        MasterInstancedModelObject::new(name, model_asset_id, texture_asset_id, shader_asset, layer, is_transparent)
    }

    pub fn new_instanced_model_transform_holder(
        &mut self,
        name: &str,
        instance: &str,
        transforms: Vec<Transform>
    ) -> InstancedModelTransformHolder {
        InstancedModelTransformHolder::new(name, instance.into(), transforms)
    }

    pub fn new_model_object(
        &mut self,
        name: &str,
        model_asset_id: ModelAssetId,
        texture_asset_id: Option<TextureAssetId>,
        shader_asset: ShaderAsset,
        is_transparent: bool,
        layer: RenderLayer
    ) -> ModelObject {
        ModelObject::new(name, model_asset_id, texture_asset_id, shader_asset, layer, is_transparent)
    }

    pub fn new_nav_obstacle(&mut self, name: &str, size: Vec3) -> NavObstacle {
        NavObstacle::new(name, size)
    }

    pub fn new_navigation_ground(&mut self, name: &str, size: Vec3) -> NavigationGround {
        NavigationGround::new(name, Vec2::new(size.x, size.z))
    }

    pub fn new_ray(&mut self, name: &str, direction: Vec3, mask: Option<CollisionGroups>) -> Ray {
        Ray::new(name, direction, mask)
    }

    pub fn new_sound_emitter(&mut self, name: &str, asset_id: SoundAssetId, is_positional: bool) -> SoundEmitter {
        let emitter_type = match is_positional {
            true => SoundSourceType::Positional,
            false => SoundSourceType::Simple,
        };

        SoundEmitter::new(name, self, asset_id, emitter_type)
    }

    pub fn new_trigger(
        &mut self, 
        name: &str,
        membership_group: Option<CollisionGroups>,
        mask: Option<CollisionGroups>,
        collider: BodyColliderType,
    ) -> Trigger {
        Trigger::new(&mut self.physics, name, membership_group, mask, collider)
    }

    pub fn get_resolution(&self) -> Vec2 {
        self.resolution
    }
}
