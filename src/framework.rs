use crate::{
    game::game_main,
    managers::{
        assets::get_full_asset_path,
        render,
        sound::{self, set_listener_transform}, systems,
    },
};
use conrod_core::{widget, widget_ids, Labelable, Positionable, Sizeable, Ui, Widget};
use conrod_glium::{
    glium_events_conversion::{handle_glium_event, WasEventHandled},
    Renderer,
};
use glium::{
    backend::glutin,
    glutin::{
        event::WindowEvent,
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    Display,
};
use std::{str::FromStr, time::Instant};

widget_ids!(struct Ids { text });

pub fn start_game(debug_mode: DebugMode) {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new();
    let cb = ContextBuilder::new().with_srgb(false);
    let mut display = Display::new(wb, cb, &event_loop).expect("failed to create glium display");

    let mut frames_count: usize = 0;
    let mut now = std::time::Instant::now();

    let mut ui = conrod_core::UiBuilder::new([2560 as f64, 1080 as f64]).build();
    ui.fonts
        .insert_from_file(get_full_asset_path("fonts/JetBrainsMono-Regular.ttf"))
        .expect("failed to add fonts");

    let ids = Ids::new(ui.widget_id_generator());
    let mut text = String::from_str("edit deez").unwrap();

    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();
    let mut renderer = Renderer::new(&display).unwrap();

    sound::init().unwrap();

    game_main::start();

    let mut redraw_ui = true;

    let mut win_w = 0;
    let mut win_h = 0;

    event_loop.run(move |ev, _, control_flow| {
        match ev {
            glium::glutin::event::Event::MainEventsCleared => {
                game_main::update();
                systems::update();

                set_listener_transform(render::get_camera_position(), render::get_camera_front());

                win_w = display.gl_window().window().inner_size().width;
                win_h = display.gl_window().window().inner_size().height;
                unsafe {
                    render::ASPECT_RATIO = win_w as f32 / win_h as f32;
                }

                let mut target = display.draw();

                render::draw(&mut target);
                game_main::render();
                systems::render(&mut display, &mut target);

                if redraw_ui == true {
                    set_ui(&mut ui, &ids, &mut text);
                    ui.has_changed();
                    redraw_ui = false;
                }
                let primitives = ui.draw();
                renderer.fill(&display, primitives, &image_map);
                renderer.draw(&display, &mut target, &image_map).unwrap();

                target.finish().unwrap();
            }
            glutin::glutin::event::Event::WindowEvent { event, .. } => {
                match handle_glium_event(&mut ui, &event, display.gl_window().window()) {
                    WasEventHandled::Yes => redraw_ui = true,
                    WasEventHandled::No => (),
                }

                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                }
            }
            _ => (),
        }

        match debug_mode {
            DebugMode::None => (),
            _ => {
                let fps = get_fps(&now, &frames_count);
                if fps.is_some() {
                    let fps = fps.unwrap();
                    display
                        .gl_window()
                        .window()
                        .set_title(&format!("very cool window: {fps} fps"));
                    frames_count = 0;
                    now = Instant::now();
                }
            }
        }

        frames_count += 1;
    });
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

fn set_ui(ui: &mut Ui, ids: &Ids, text: &mut String) {
    let mut ui_cell = ui.set_widgets();
    widget::Button::new()
        .middle_of(ui_cell.window)
        .y_relative(100.0)
        .y_dimension(conrod_core::position::Dimension::Absolute(100.0))
        .x_dimension(conrod_core::position::Dimension::Absolute(300.0))
        .label("test button")
        .label_font_size(32)
        .press_color(conrod_core::Color::Rgba(1.0, 1.0, 1.0, 1.0))
        .set(ids.text, &mut ui_cell);

    /*for event in widget::TextBox::new(&text)
        .middle_of(ui_cell.window)
        .x_dimension(conrod_core::position::Dimension::Absolute(500.0))
        .y_dimension(conrod_core::position::Dimension::Absolute(50.0))
        .set(ids.text, &mut ui_cell) {

        match event {
            widget::text_box::Event::Enter => (),
            widget::text_box::Event::Update(string) => *text = string,
        }
    }*/
}

fn set_audio_listener_transformations() {
    let camera_pos = render::get_camera_position();
    let camera_rot = render::get_camera_rotation();

    sound::set_listener_position(camera_pos);
    sound::set_listener_orientation(camera_rot);
}
