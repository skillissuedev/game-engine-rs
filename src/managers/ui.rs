use std::collections::HashMap;
use egui_glium::egui_winit::egui::{self, Button, Checkbox, ColorImage, ComboBox, Context, Image, Label, ProgressBar, RichText, Sense, Slider, TextEdit, TextureHandle, Ui, Visuals, Window};
use glam::{Vec2, Vec3};
use image::GenericImageView;
use crate::framework::{DebugMode, Framework};
use super::{assets::get_full_asset_path, debugger, physics::RenderColliderType, systems};

pub struct ImageToLoad {
    id: String,
    bytes: Vec<u8>,
    dimenstions: [u32; 2],
}

#[derive(Default)]
pub struct UiManager {
    windows: HashMap<String, UiManagerWindow>,
    images_to_load: Vec<ImageToLoad>,
    textures: HashMap<String, TextureHandle>,
    themes: HashMap<String, Visuals>,
}

#[derive(Clone, Debug)]
pub enum WidgetData {
    Button(String),
    Label(String, f32),
    Horizontal,
    Vertical,
    SinglelineTextEdit(String),
    MultilineTextEdit(String),
    Checkbox(bool, String),
    FloatSlider(f32, f32, f32),
    IntSlider(i32, i32, i32),
    ProgressBar(f32),
    Image(String),
}

impl Default for WidgetData {
    fn default() -> Self {
        Self::Label(String::new(), 14.0)
    }
}

#[derive(Clone, Default, Debug)]
pub struct WidgetState {
    pub left_clicked: bool,
    pub right_clicked: bool,
    pub double_clicked: bool,
    pub hovered: bool,
    pub dragged: bool,
    pub changed: bool,
}

#[derive(Debug)]
pub struct UiManagerWindow {
    position: Option<Vec2>,
    size: Option<Vec2>,
    widgets: Vec<Widget>,
    transparent: bool,
    show_title_bar: bool,
    show_close_button: bool,
    show_on_top: bool,
    theme: Option<String>
}

#[derive(Clone, Debug)]
pub struct Widget {
    id: String,
    size: Vec2,
    widget_data: WidgetData,
    children: Vec<Widget>,
    state: WidgetState,
    spacing: f32,
    theme: Option<String>
}

impl Default for Widget {
    fn default() -> Self {
        Widget {
            id: String::new(),
            size: Vec2::ZERO,
            widget_data: WidgetData::Label(String::new(), 14.0),
            children: Vec::new(),
            state: WidgetState::default(),
            spacing: 10.0,
            theme: None
        }
    }
}

impl UiManager {
    pub fn render(&mut self, ctx: &Context) {
        for image_to_load in &self.images_to_load {
            if self.textures.contains_key(&image_to_load.id) == false {
                println!("ui manager: loading a texture {}", &image_to_load.id);
                let dimenstions = image_to_load.dimenstions;
                let width = dimenstions[0] as usize;
                let height = dimenstions[1] as usize;
                let texture = ctx.load_texture(
                    image_to_load.id.clone(),
                    egui::ImageData::from(ColorImage::from_rgba_unmultiplied([width, height], &image_to_load.bytes)),
                    egui::TextureOptions::default()
                );
                self.textures.insert(image_to_load.id.clone(), texture);
            }
        }
        self.images_to_load.clear();

        for (id, manager_window) in self.windows.iter_mut() {
            let egui_id = egui::Id::new(id);
            let mut window = Window::new(id)
                .id(egui_id)
                .resizable(false);
            
            if manager_window.transparent {
                window = window
                    .frame(egui::Frame::none())
                    .title_bar(false)
                    .scroll(false)
            } else {
                window = window
                    .title_bar(manager_window.show_title_bar)
                    .collapsible(manager_window.show_close_button)
            }

            if manager_window.show_on_top {
                let layer_id = egui::LayerId::new(egui::Order::Middle, egui_id);
                ctx.move_to_top(layer_id);
            }

            if let Some(position) = manager_window.position {
                window = window.fixed_pos([position.x, position.y]);
            }

            window.show(ctx, |ui| {
                for widget in &mut manager_window.widgets {
                    Self::render_widget(&self.textures, ctx, ui, widget, &self.themes);
                }
            });
        }
    }

    fn render_widget(textures: &HashMap<String, TextureHandle>, ctx: &Context, ui: &mut Ui, widget: &mut Widget, themes: &HashMap<String, Visuals>) {
        let theme = match &widget.theme {
            Some(theme_id) => {
                match themes.get(theme_id) {
                    Some(visuals) => visuals.to_owned(),
                    None => Visuals::default(),
                }
            },
            None => Visuals::default(),
        };
        *ui.visuals_mut() = theme;

        let size = egui::Vec2::new(widget.size.x, widget.size.y);
        ui.spacing_mut().item_spacing.x = widget.spacing;
        ui.spacing_mut().item_spacing.y = widget.spacing;

        let egui_widget = match &mut widget.widget_data {
            WidgetData::Button(contents) => ui.add_sized(size, Button::new(contents.as_str()).sense(Sense::click_and_drag())),
            WidgetData::Label(contents, text_size) => 
                ui.add_sized(size, Label::new(RichText::new(contents.clone()).size(*text_size)).sense(Sense::click_and_drag())),
            WidgetData::Horizontal => {
                ui.horizontal(|ui| {
                    for widget in &mut widget.children {
                        Self::render_widget(textures, ctx, ui, widget, themes);
                    }
                }).response
            },
            WidgetData::Vertical => {
                ui.vertical(|ui| {
                    for widget in &mut widget.children {
                        Self::render_widget(textures, ctx, ui, widget, themes);
                    }
                }).response
            },
            WidgetData::SinglelineTextEdit(contents) => ui.add_sized(size, TextEdit::singleline(contents)),
            WidgetData::MultilineTextEdit(contents) => ui.add_sized(size, TextEdit::multiline(contents)),
            WidgetData::Checkbox(value, text) => ui.add_sized(size, Checkbox::new(value, text.as_str())),
            WidgetData::FloatSlider(value, min, max) => ui.add_sized(size, Slider::new(value, *min..=*max)),
            WidgetData::IntSlider(value, min, max) => ui.add_sized(size, Slider::new(value, *min..=*max)),
            WidgetData::ProgressBar(value) => ui.add_sized(size, ProgressBar::new(*value)),
            WidgetData::Image(image_path) => {
                let texture = textures.get(image_path);
                match texture {
                    Some(texture) => ui.add_sized(size, Image::new(texture).fit_to_exact_size(size).sense(Sense::click_and_drag())),
                    None => ui.add_sized(size, Label::new(format!("can't load the texture! id: {}", image_path)).sense(Sense::click_and_drag()))
                }
            },
        };

        let state = WidgetState {
            left_clicked: egui_widget.clicked(),
            right_clicked: egui_widget.secondary_clicked(),
            hovered: egui_widget.hovered(),
            dragged: egui_widget.dragged(),
            changed: egui_widget.changed(),
            double_clicked: egui_widget.double_clicked(),
        };

        widget.state = state;
    }

    fn remove_child(target_widget: &str, current_widget: &mut Widget) -> bool {
        let mut widget_idx = None;

        for (idx, widget) in current_widget.children.iter().enumerate() {
            let id = &widget.id;
            if id == target_widget {
                widget_idx = Some(idx);
                break
            } 
        }

        match widget_idx {
            Some(idx) => {
                current_widget.children.remove(idx);
                return true
            },
            None => (),
        }

        for widget in &mut current_widget.children {
            if Self::remove_child(target_widget, widget) == true {
                return true
            }
        }

        false
    }

    fn add_child(function_name: &str, target_widget: &str, current_widget: &mut Widget, widget_to_add: Widget, widget_to_add_id: &str) -> bool {
        for widget in &mut current_widget.children {
            let id = &widget.id;
            if id == target_widget {
                for i in &widget.children {
                    if i.id == widget_to_add_id {
                        debugger::error(
                            &format!(
                                "{} error!\nChild with id '{}' already exists in the children list of the widget with id '{}'", function_name, widget_to_add_id, id
                            )
                        );

                        return false
                    }
                }

                widget.children.push(widget_to_add);
                return true
            } 

            if Self::add_child(function_name, target_widget, widget, widget_to_add.clone(), widget_to_add_id) == true {
                return true
            }
        }

        false
    }
    // exists to write less in functions like add_button, add_label, ...
    fn add_widget(&mut self, function_name: &str, window_id: &str, widget_id: &str, widget: Widget, parent: Option<&str>) {
        match self.windows.get_mut(window_id) {
            Some(window) => {
                match parent {
                    Some(parent) => {
                        for child in &mut window.widgets {
                            let child_id = &child.id;
                            if child_id == parent {
                                for i in &widget.children {
                                    if i.id == widget_id {
                                        debugger::error(
                                            &format!(
                                                "{} error!\nChild with id '{}' already exists in the children list of the widget with id '{}'", 
                                                function_name, widget_id, child_id
                                            )
                                        );
                                        return
                                    }
                                }

                                child.children.push(widget.clone());
                                return
                            }

                            Self::add_child(function_name, parent, child, widget.clone(), widget_id);
                        }
                    },
                    None => {
                        for i in &window.widgets {
                            if i.id == widget_id {
                                debugger::error(
                                    &format!(
                                        "{} error!\nWidget with id '{}' already exists in the window with id '{}'", function_name, widget_id, window_id
                                    )
                                );
                                return
                            }
                        }

                        window.widgets.push(widget);
                    },
                }
            },
            None => {
                debugger::error(
                    &format!(
                        "{} error!\nFailed to get the window with id '{}' to add a widget with id '{}'", function_name, window_id, widget_id
                    )
                )
            },
        }
    }





    // Public methods:
    pub fn add_theme(&mut self, theme_id: String, theme_json: String) {
        if self.themes.contains_key(&theme_id) {
            debugger::error(
                &format!("UI manager: Failed to add a theme '{}'! Theme with this id already exists", theme_id)
            );
            return
        }

        let theme: Result<Visuals, serde_json::Error> = serde_json::from_str(&theme_json);
        match theme {
            Ok(theme) => {
                self.themes.insert(theme_id, theme);
            },
            Err(err) => {
                debugger::error(
                    &format!("UI manager: Failed to add a theme '{}'! Failed to deserialize the JSON string! Err: {}", theme_id, err)
                );
            },
        }
    }

    pub fn show_title_bar(&mut self, window_id: &str, show: bool) {
        match self.windows.get_mut(window_id) {
            Some(window) => {
                window.show_title_bar = show;
            },
            None => {
                debugger::error(
                    &format!(
                        "show_title_button error!\nFailed to get the window with id '{}'", window_id
                    )
                );
            },
        }
    }

    pub fn show_close_button(&mut self, window_id: &str, show: bool) {
        match self.windows.get_mut(window_id) {
            Some(window) => {
                window.show_close_button = show;
            },
            None => {
                debugger::error(
                    &format!(
                        "show_close_button error!\nFailed to get the window with id '{}'", window_id
                    )
                );
            },
        }
    }

    pub fn remove_widget(&mut self, window_id: &str, widget_id: &str) {
        match self.windows.get_mut(window_id) {
            Some(window) => {
                let mut widget_idx = None;
                for (idx, widget) in window.widgets.iter().enumerate() {
                    if widget.id == widget_id {
                        widget_idx = Some(idx);
                    }
                }
                match widget_idx {
                    Some(idx) => {
                        window.widgets.remove(idx);
                        return
                    },
                    None => (),
                }

                // if true, something was removed
                for widget in &mut window.widgets {
                    if Self::remove_child(widget_id, widget) == true {
                        return
                    }
                }

                debugger::error(
                    &format!(
                        "remove_widget error!\nFailed to get the widget with id '{}' in the window with id '{}'", widget_id, window_id
                    )
                );
            },
            None => {
                debugger::error(
                    &format!(
                        "remove_widget error!\nFailed to get the window with id '{}' to get widget with id '{}'", window_id, widget_id
                    )
                );
            },
        }
    }


    pub fn set_window_position(&mut self, window_id: &str, position: Option<Vec2>) {
        match self.windows.get_mut(window_id) {
            Some(window) => {
                window.position = position;
            },
            None => {
                debugger::error(&format!("set_window_position error!\nFailed to get the window with id '{}'", window_id));
            },
        }

    }

    pub fn get_widget_state(&self, window_id: &str, widget_id: &str) -> Option<WidgetState> {
        match self.windows.get(window_id) {
            Some(window) => {
                for widget in &window.widgets {
                    if &widget.id == widget_id {
                        return Some(widget.state.clone())
                    }
                    
                    let children_state = self.get_children_widget_state(widget, widget_id);
                    if children_state.is_some() {
                        return children_state
                    }
                }

                debugger::error(
                    &format!(
                        "get_widget_state error!\nFailed to get the widget with id '{}' in the window with id '{}'", widget_id, window_id
                    )
                );
                None
            },
            None => {
                debugger::error(
                    &format!(
                        "get_widget_state error!\nFailed to get the window with id '{}' to get state of the widget with id '{}'", window_id, widget_id
                    )
                );
                None
            },
        }
    }

    pub fn get_children_widget_state(&self, widget: &Widget, widget_id: &str) -> Option<WidgetState> {
        for widget in &widget.children {
            if &widget.id == widget_id {
                return Some(widget.state.clone())
            }

            let children_state = self.get_children_widget_state(widget, widget_id);
            if children_state.is_some() {
                return children_state
            }
        }

        None
    }


    fn set_child_widget_theme(widget: &mut Widget, widget_id: &str, theme_id: Option<&str>) -> bool {
        if &widget.id == widget_id {
            let string_option_theme = match theme_id {
                Some(theme_id) => Some(theme_id.to_string()),
                None => None,
            };

            widget.theme = string_option_theme;
            return true
        }

        for widget in &mut widget.children {
            Self::set_child_widget_theme(widget, widget_id, theme_id);
        }
        return false
    }
        

    pub fn set_widget_theme(&mut self, window_id: &str, widget_id: &str, theme: Option<&str>) {
        match self.windows.get_mut(window_id) {
            Some(window) => {
                for widget in &mut window.widgets {
                    if Self::set_child_widget_theme(widget, widget_id, theme) == true {
                        return
                    }
                }

                debugger::error(
                    &format!(
                        "set_widget_theme error!\nFailed to get the widget with id '{}' in the window with id '{}'", widget_id, window_id
                    )
                );
            },
            None => {
                debugger::error(
                    &format!(
                        "set_widget_theme error!\nFailed to get the window with id '{}' to get widget with id '{}'", window_id, widget_id
                    )
                );
            },
        }
    }



    fn set_child_widget_spacing(widget: &mut Widget, widget_id: &str, spacing: f32) -> bool {
        if &widget.id == widget_id {
            widget.spacing = spacing;
            return true
        }

        for widget in &mut widget.children {
            Self::set_child_widget_spacing(widget, widget_id, spacing);
        }
        return false
    }
        

    pub fn set_widget_spacing(&mut self, window_id: &str, widget_id: &str, spacing: f32) {
        match self.windows.get_mut(window_id) {
            Some(window) => {
                for widget in &mut window.widgets {
                    if Self::set_child_widget_spacing(widget, widget_id, spacing) == true {
                        return
                    }
                }

                debugger::error(
                    &format!(
                        "set_widget_spacing error!\nFailed to get the widget with id '{}' in the window with id '{}'", widget_id, window_id
                    )
                );
            },
            None => {
                debugger::error(
                    &format!(
                        "set_widget_spacing error!\nFailed to get the window with id '{}' to get widget with id '{}'", window_id, widget_id
                    )
                );
            },
        }
    }



    pub fn get_widget_text(&self, window_id: &str, widget_id: &str) -> Option<String> {
        match self.windows.get(window_id) {
            Some(window) => {
                for widget in &window.widgets {
                    if &widget.id == widget_id {
                        return match widget.widget_data.clone() {
                            WidgetData::Button(contents) => Some(contents),
                            WidgetData::Label(contents, _) => Some(contents),
                            WidgetData::SinglelineTextEdit(contents) => Some(contents),
                            WidgetData::MultilineTextEdit(contents) => Some(contents),
                            WidgetData::Checkbox(_, label) => Some(label),
                            _ => {
                                debugger::error(
                                    &format!(
                                        "get_widget_text error!\nWidget with id '{}' doesn't contain any text.", widget_id
                                    )
                                );
                                None
                            }
                        }
                    }
                }

                debugger::error(
                    &format!(
                        "get_widget_text error!\nFailed to get the widget with id '{}' in the window with id '{}'", widget_id, window_id
                    )
                );
                None
            },
            None => {
                debugger::error(
                    &format!(
                        "get_widget_text error!\nFailed to get the window with id '{}' to get widget with id '{}'", window_id, widget_id
                    )
                );
                None
            },
        }
    }

    pub fn get_widget_numeric_value(&self, window_id: &str, widget_id: &str) -> Option<f32> {
        match self.windows.get(window_id) {
            Some(window) => {
                for widget in &window.widgets {
                    if &widget.id == widget_id {
                        return match widget.widget_data.clone() {
                            WidgetData::IntSlider(value, _, _) => Some(value as f32),
                            WidgetData::FloatSlider(value, _, _) => Some(value),
                            WidgetData::ProgressBar(value) => Some(value),
                            _ => {
                                debugger::error(
                                    &format!(
                                        "get_widget_numeric_value error!\nWidget with id '{}' doesn't contain any num values.", widget_id
                                    )
                                );
                                None
                            }
                        }
                    }
                }

                debugger::error(
                    &format!(
                        "get_widget_numeric_value error!\nFailed to get the widget with id '{}' in the window with id '{}'", widget_id, window_id
                    )
                );
                None
            },
            None => {
                debugger::error(
                    &format!(
                        "get_widget_numeric_value error!\nFailed to get the window with id '{}' to get widget with id '{}'", window_id, widget_id
                    )
                );
                None
            },
        }
    }

    pub fn get_widget_bool_value(&self, window_id: &str, widget_id: &str) -> Option<bool> {
        match self.windows.get(window_id) {
            Some(window) => {
                for widget in &window.widgets {
                    if &widget.id == widget_id {
                        return match widget.widget_data.clone() {
                            WidgetData::Checkbox(checked, _) => Some(checked),
                            _ => {
                                debugger::error(
                                    &format!(
                                        "get_widget_bool_value error!\nWidget with id '{}' doesn't contain any bool values.", widget_id
                                    )
                                );
                                None
                            }
                        }
                    }
                }

                debugger::error(
                    &format!(
                        "get_widget_bool_value error!\nFailed to get the widget with id '{}' in the window with id '{}'", widget_id, window_id
                    )
                );
                None
            },
            None => {
                debugger::error(
                    &format!(
                        "get_widget_bool_value error!\nFailed to get the window with id '{}' to get widget with id '{}'", window_id, widget_id
                    )
                );
                None
            },
        }
    }






    pub fn is_widget_left_clicked(&self, window_id: &str, widget_id: &str) -> bool {
        match self.get_widget_state(window_id, widget_id) {
            Some(value) => value.left_clicked,
            None => false,
        }
    }

    pub fn is_widget_right_clicked(&self, window_id: &str, widget_id: &str) -> bool {
        match self.get_widget_state(window_id, widget_id) {
            Some(value) => value.right_clicked,
            None => false,
        }
    }

    pub fn is_widget_double_clicked(&self, window_id: &str, widget_id: &str) -> bool {
        match self.get_widget_state(window_id, widget_id) {
            Some(value) => value.double_clicked,
            None => false,
        }
    }

    pub fn is_widget_hovered(&self, window_id: &str, widget_id: &str) -> bool {
        match self.get_widget_state(window_id, widget_id) {
            Some(value) => value.hovered,
            None => false,
        }
    }

    pub fn is_widget_dragged(&self, window_id: &str, widget_id: &str) -> bool {
        match self.get_widget_state(window_id, widget_id) {
            Some(value) => value.dragged,
            None => false,
        }
    }

    pub fn is_widget_changed(&self, window_id: &str, widget_id: &str) -> bool {
        match self.get_widget_state(window_id, widget_id) {
            Some(value) => value.changed,
            None => false,
        }
    }







    pub fn new_window(&mut self, id: &str, transparent: bool) {
        if self.windows.contains_key(id) == true {
            debugger::error(&format!("new_window error!\nWindow with id '{}' already exists!", id));
            return;
        }
        self.windows.insert(id.into(), UiManagerWindow {
            position: None,
            size: None,
            widgets: Vec::new(),
            transparent,
            show_title_bar: true,
            show_close_button: true,
            show_on_top: false,
            theme: None
        });
    }

    pub fn set_window_on_top(&mut self, window_id: &str, show_on_top: bool) {
        match self.windows.get_mut(window_id) {
            Some(window) => window.show_on_top = show_on_top,
            None => {
                debugger::error(
                    &format!(
                        "set_window_on_top error!\nFailed to get the window with id '{}'", window_id
                    )
                )
            },
        }
    }

    pub fn remove_window(&mut self, id: &str) {
        self.windows.retain(|key, _| {
            if key == id {
                false
            } else {
                true
            }
        });
    }

    pub fn add_button(&mut self, window_id: &str, widget_id: &str, contents: &str, size: Vec2, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            size,
            widget_data: WidgetData::Button(contents.into()),
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_button", window_id, widget_id, widget, parent)
    }

    pub fn add_label(&mut self, window_id: &str, widget_id: &str, contents: &str, text_size: Option<f32>, size: Vec2, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            size,
            widget_data: WidgetData::Label(contents.into(), text_size.unwrap_or(14.0)),
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_label", window_id, widget_id, widget, parent)
    }

    pub fn add_horizontal(&mut self, window_id: &str, widget_id: &str, size: Vec2, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            size,
            widget_data: WidgetData::Horizontal,
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_horizontal", window_id, widget_id, widget, parent)
    }

    pub fn add_vertical(&mut self, window_id: &str, widget_id: &str, size: Vec2, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            size,
            widget_data: WidgetData::Vertical,
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_vertical", window_id, widget_id, widget, parent)
    }

    pub fn add_singleline_text_edit(&mut self, window_id: &str, widget_id: &str, contents: &str, size: Vec2, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            size,
            widget_data: WidgetData::SinglelineTextEdit(contents.into()),
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_singleline_text_edit", window_id, widget_id, widget, parent)
    }

    pub fn add_multiline_text_edit(&mut self, window_id: &str, widget_id: &str, contents: &str, size: Vec2, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            size,
            widget_data: WidgetData::MultilineTextEdit(contents.into()),
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_multiline_text_edit", window_id, widget_id, widget, parent)
    }

    pub fn add_checkbox(&mut self, window_id: &str, widget_id: &str, contents: bool, label: &str, size: Vec2, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            size,
            widget_data: WidgetData::Checkbox(contents, label.into()),
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_checkbox", window_id, widget_id, widget, parent)
    }

    pub fn add_float_slider(&mut self, window_id: &str, widget_id: &str, current_value: f32, min: f32, max: f32, size: Vec2, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            size,
            widget_data: WidgetData::FloatSlider(current_value, min, max),
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_float_slider", window_id, widget_id, widget, parent)
    }

    pub fn add_int_slider(&mut self, window_id: &str, widget_id: &str, current_value: i32, min: i32, max: i32, size: Vec2, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            size,
            widget_data: WidgetData::IntSlider(current_value, min, max),
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_int_slider", window_id, widget_id, widget, parent)
    }

    pub fn add_progress_bar(&mut self, window_id: &str, widget_id: &str, current_value: f32, size: Vec2, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            size,
            widget_data: WidgetData::ProgressBar(current_value),
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_progress_bar", window_id, widget_id, widget, parent)
    }

    pub fn add_image(&mut self, window_id: &str, widget_id: &str, image_path: &str, size: Vec2, parent: Option<&str>) {
        let image = image::open(get_full_asset_path(image_path));
        match image {
            Ok(image) => {
                let bytes = image.to_rgba8().into_raw();
                let dimenstions = image.dimensions().into();

                let widget = Widget {
                    id: widget_id.into(),
                    size,
                    widget_data: WidgetData::Image(image_path.into()),
                    children: Vec::new(),
                    ..Default::default()
                };

                self.images_to_load.push(ImageToLoad {
                    id: image_path.into(),
                    bytes,
                    dimenstions,
                });

                self.add_widget("add_image", window_id, widget_id, widget, parent)
            },
            Err(err) => {
                debugger::error(&format!("add_image error!\nFailed to load the image '{}', error: {}", image_path, err));

                let widget = Widget {
                    id: widget_id.into(),
                    size,
                    widget_data: WidgetData::Label(format!("failed to load image '{}'", image_path), 14.0),
                    children: Vec::new(),
                    ..Default::default()
                };

                self.add_widget("add_image", window_id, widget_id, widget, parent)
            },
        }
    }
}

// inspector
pub fn draw_inspector(framework: &mut Framework, ui: &mut Ui, fps: &usize, ui_state: &mut UiState) {
    ui.label(format!("fps: {}", fps));
    ui.checkbox(&mut ui_state.full_debug_checkbox_val, "full debug");
    handle_full_debug_checkbox_value(framework, ui_state.full_debug_checkbox_val);

    ui.separator();

    ui.heading("systems:");
    for system in systems::get_systems_iter() {
        let system_id = system.system_id();

        ui.collapsing(system_id, |ui| {
            for object in system.objects_list() {
                let object_name = object.name();
                if ui.small_button(object_name).clicked() {
                    ui_state.selected_inspector_object = Some(SelectedInspectorObject {
                        current_selected_object_system: system_id.into(),
                        current_selected_object_name: object_name.into(),
                        //input_postition: None,
                        position: Vec3Inspector::default(),
                        rotation: Vec3Inspector::default(),
                        scale: Vec3Inspector::default(),
                        render_collider: None,
                        render_collider_size: Vec3Inspector {
                            input_value: Some(["".into(), "".into(), "".into()]),
                        },
                        cancel_collider: false,
                        new_group_name: String::new(),
                    });
                }
            }
        });
    }

    ui.separator();

    let selected_object = &mut ui_state.selected_inspector_object;
    if let Some(selected_object) = selected_object {
        match systems::get_system_mut_with_id(&selected_object.current_selected_object_system) {
            Some(system) => {
                match system.find_object_mut(&selected_object.current_selected_object_name) {
                    Some(object) => {
                        ui.heading(format!("object '{}'", object.name()));
                        ui.label(format!("type: {}", object.object_type()));
                        ui.separator();
                        ui.collapsing("children", |ui| {
                            for object in object.children_list() {
                                if ui.button(object.name()).clicked() {
                                    selected_object.current_selected_object_name =
                                        object.name().into();
                                    return;
                                }
                            }
                        });

                        ui.collapsing("groups", |ui| {
                            for group in object.groups_list().clone() {
                                ui.horizontal(|ui| {
                                    ui.label(group.as_raw());
                                    if ui.button("remove").clicked() {
                                        object.remove_from_group(group.as_raw());
                                    }
                                });
                            }
                            ui.horizontal(|ui| {
                                ui.label("group name:");
                                ui.text_edit_singleline(&mut selected_object.new_group_name);
                                if ui.button("add group").clicked() {
                                    object.add_to_group(&selected_object.new_group_name);
                                }
                            });
                        });

                        ui.label("local position:");
                        if let Some(pos) = draw_vec3_editor_inspector(
                            ui,
                            &mut selected_object.position,
                            &object.local_transform().position,
                            true,
                        ) {
                            object.set_position(framework, pos, true);
                        }
                        ui.label("local rotation:");
                        if let Some(rot) = draw_vec3_editor_inspector(
                            ui,
                            &mut selected_object.rotation,
                            &object.local_transform().rotation,
                            true,
                        ) {
                            object.set_rotation(framework, rot, true);
                        }
                        ui.label("local scale:");
                        if let Some(sc) = draw_vec3_editor_inspector(
                            ui,
                            &mut selected_object.scale,
                            &object.local_transform().scale,
                            true,
                        ) {
                            object.set_scale(sc);
                        }

                        // collider
                        if selected_object.cancel_collider {
                            selected_object.render_collider = None;
                            selected_object.cancel_collider = false;
                            return;
                        }
                        if let Some(collider) = &mut selected_object.render_collider {
                            ui.separator();
                            ui.label("render collider settings:");
                            ComboBox::from_label("collider type")
                                .selected_text(
                                    format!("{:?}", collider.collider_type).to_lowercase(),
                                )
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut collider.collider_type,
                                        InspectorRenderColliderType::Cuboid,
                                        "cuboid",
                                    );
                                    ui.selectable_value(
                                        &mut collider.collider_type,
                                        InspectorRenderColliderType::Ball,
                                        "ball",
                                    );
                                    ui.selectable_value(
                                        &mut collider.collider_type,
                                        InspectorRenderColliderType::Capsule,
                                        "capsule",
                                    );
                                    ui.selectable_value(
                                        &mut collider.collider_type,
                                        InspectorRenderColliderType::Cylinder,
                                        "cylinder",
                                    );
                                });
                            if let Some(ref mut input_value) =
                                &mut selected_object.render_collider_size.input_value
                            {
                                match &collider.collider_type {
                                    InspectorRenderColliderType::Ball => {
                                        ui.horizontal(|ui| {
                                            ui.label("radius:");
                                            ui.text_edit_singleline(&mut input_value[0]);
                                        });
                                    }
                                    InspectorRenderColliderType::Cuboid => {
                                        draw_vec3_editor_inspector(
                                            ui,
                                            &mut selected_object.render_collider_size,
                                            &Vec3::new(1.0, 1.0, 1.0),
                                            false,
                                        );
                                    }
                                    InspectorRenderColliderType::Capsule => {
                                        ui.horizontal(|ui| {
                                            ui.label("radius:");
                                            ui.text_edit_singleline(&mut input_value[0]);
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("height:");
                                            ui.text_edit_singleline(&mut input_value[1]);
                                        });
                                    }
                                    InspectorRenderColliderType::Cylinder => {
                                        ui.horizontal(|ui| {
                                            ui.label("radius:");
                                            ui.text_edit_singleline(&mut input_value[0]);
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("height:");
                                            ui.text_edit_singleline(&mut input_value[1]);
                                        });
                                    }
                                }
                            }
                            let render_collider_size = &mut selected_object.render_collider_size;
                            ui.horizontal(|ui| {
                                if ui.button("cancel").clicked() {
                                    selected_object.cancel_collider = true;
                                }

                                if ui.button("done").clicked() {
                                    let mut trigger = false;
                                    if object.object_type() == "Trigger" {
                                        trigger = true;
                                    };

                                    if let Some(ref input_value) = render_collider_size.input_value
                                    {
                                        let x = input_value[0].parse::<f32>();
                                        if let Ok(x) = x {
                                            let y = input_value[1].parse::<f32>();
                                            if let Ok(y) = y {
                                                let z = input_value[2].parse::<f32>();
                                                if let Ok(z) = z {
                                                    match collider.clone().collider_type {
                                                        InspectorRenderColliderType::Ball => {
                                                            object.build_object_rigid_body(
                                                                framework,
                                                                None,
                                                                Some(RenderColliderType::Ball(
                                                                    None, None, x, trigger,
                                                                )),
                                                                1.0,
                                                                None,
                                                                None,
                                                            );
                                                        }
                                                        InspectorRenderColliderType::Cuboid => {
                                                            object.build_object_rigid_body(
                                                                framework,
                                                                None,
                                                                Some(RenderColliderType::Cuboid(
                                                                    None, None, x, y, z, trigger,
                                                                )),
                                                                1.0,
                                                                None,
                                                                None,
                                                            );
                                                        }
                                                        InspectorRenderColliderType::Capsule => {
                                                            object.build_object_rigid_body(
                                                                framework,
                                                                None,
                                                                Some(RenderColliderType::Capsule(
                                                                    None, None, x, y, trigger,
                                                                )),
                                                                1.0,
                                                                None,
                                                                None,
                                                            );
                                                        }
                                                        InspectorRenderColliderType::Cylinder => {
                                                            object.build_object_rigid_body(
                                                                framework,
                                                                None,
                                                                Some(RenderColliderType::Cylinder(
                                                                    None, None, x, y, trigger,
                                                                )),
                                                                1.0,
                                                                None,
                                                                None,
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        } else {
                            if ui.button("render collider settings").clicked() {
                                match selected_object.render_collider {
                                    Some(_) => selected_object.render_collider = None,
                                    None => {
                                        selected_object.render_collider =
                                            Some(InspectorRenderCollider {
                                                collider_type: InspectorRenderColliderType::Cuboid,
                                            })
                                    }
                                }
                            }
                        }

                        ui.separator();
                        object.inspector_ui(framework, ui);
                    }
                    None => {
                        ui.heading(format!(
                            "Failed to get an object with name {} in the system with id {}",
                            selected_object.current_selected_object_name,
                            selected_object.current_selected_object_system
                        ));
                    }
                }
            }
            None => {
                ui.heading(format!(
                    "Failed to get a system with id {}",
                    selected_object.current_selected_object_system
                ));
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct UiState {
    full_debug_checkbox_val: bool,
    selected_inspector_object: Option<SelectedInspectorObject>,
}

#[derive(Default, Debug)]
pub struct SelectedInspectorObject {
    current_selected_object_system: String,
    current_selected_object_name: String,
    position: Vec3Inspector,
    rotation: Vec3Inspector,
    scale: Vec3Inspector,
    render_collider: Option<InspectorRenderCollider>,
    render_collider_size: Vec3Inspector,
    cancel_collider: bool,
    new_group_name: String, //input_postition: Option<[String; 3]>,
}

#[derive(Debug, Default)]
pub struct Vec3Inspector {
    input_value: Option<[String; 3]>,
}

#[derive(Debug, Clone)]
struct InspectorRenderCollider {
    collider_type: InspectorRenderColliderType,
}

#[derive(Debug, PartialEq, Clone)]
enum InspectorRenderColliderType {
    /// radius
    Ball, //(f32),
    /// half-size
    Cuboid, //(Vec3),
    /// radius, height
    Capsule, //(f32, f32),
    /// radius, height
    Cylinder, //(f32, f32),
}

fn handle_full_debug_checkbox_value(framework: &mut Framework, full_debug: bool) {
    if full_debug {
        framework.set_debug_mode(DebugMode::Full)
    } else {
        framework.set_debug_mode(DebugMode::ShowFps)
    }
}

pub fn draw_vec3_editor_inspector(
    ui: &mut Ui,
    vec3: &mut Vec3Inspector,
    object_val: &Vec3,
    show_default_buttons: bool,
) -> Option<Vec3> {
    let mut return_val: Option<Vec3> = None;
    ui.horizontal(|ui| {
        match vec3.input_value {
            Some(ref mut input_val) => {
                ui.label("x:");
                ui.add_sized(
                    egui::vec2(70.0, 20.0),
                    TextEdit::singleline(&mut input_val[0]),
                );
                ui.label("y:");
                ui.add_sized(
                    egui::vec2(70.0, 20.0),
                    TextEdit::singleline(&mut input_val[1]),
                );
                ui.label("z:");
                ui.add_sized(
                    egui::vec2(70.0, 20.0),
                    TextEdit::singleline(&mut input_val[2]),
                );

                if show_default_buttons {
                    if ui.button("done").clicked() {
                        let x = input_val[0].parse::<f32>();
                        if let Ok(x) = x {
                            let y = input_val[1].parse::<f32>();
                            if let Ok(y) = y {
                                let z = input_val[2].parse::<f32>();
                                if let Ok(z) = z {
                                    return_val = Some(Vec3::new(x, y, z));
                                    vec3.input_value = None;
                                    return;
                                }
                            }
                        }
                    };
                    if ui.button("cancel").clicked() {
                        vec3.input_value = None;
                    };
                }
            }
            None => {
                let val = object_val.clone();
                let mut val_array = [val.x.to_string(), val.y.to_string(), val.z.to_string()];
                ui.label("x:");
                ui.add_sized(
                    egui::vec2(70.0, 20.0),
                    TextEdit::singleline(&mut val_array[0]),
                );
                ui.label("y:");
                ui.add_sized(
                    egui::vec2(70.0, 20.0),
                    TextEdit::singleline(&mut val_array[1]),
                );
                ui.label("z:");
                ui.add_sized(
                    egui::vec2(70.0, 20.0),
                    TextEdit::singleline(&mut val_array[2]),
                );

                if show_default_buttons && ui.button("edit").clicked() {
                    vec3.input_value = Some(val_array.clone());
                };
            }
        }
        return_val = None;
    });

    return_val
}

