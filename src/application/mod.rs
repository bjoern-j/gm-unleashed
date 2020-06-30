use imgui::*;
use super::{ Fonts, FontStyle };

mod ui_tools;

use ui_tools::{ Button, TextField, markdown };

mod campaign;

use campaign::{ Campaign, EntityContent };

trait ApplicationState {
    fn build_gui(self : Box<Self>, ui : &Ui) -> Box<dyn ApplicationState>;
}

trait ApplicationSubstate {
    fn build_gui(self : Box<Self>, ui : &Ui, fonts : &Fonts) -> Box<dyn ApplicationSubstate>;    
    fn persist(&mut self, campaign : &mut Campaign);
    fn expired(&self) -> bool;
}

struct EmptyState;
impl ApplicationState for EmptyState {
    fn build_gui(self : Box<Self>, _ui : &Ui) -> Box<dyn ApplicationState> { self }
}

struct InitialState {
    title : ImString,
    create_button : Button,
    fonts : Fonts,
}

impl ApplicationState for InitialState {
    fn build_gui(mut self : Box<Self>, ui : &Ui) -> Box<dyn ApplicationState> {
        let title = &self.title;
        let create_button = &mut self.create_button;
        Window::new(&title).build(
            ui,
            || { create_button.build_gui(ui); }
        );
        if create_button.pressed() {
            Box::new(CreateCampaignState::new(self.fonts))
        } else {
            self
        }
    }
}

impl InitialState {
    const LABEL_CREATE : &'static str = "Create new campaign"; 
    pub fn new(fonts : Fonts) -> Self { InitialState{
        title : ImString::new(Application::MAIN_MENU_TITLE),
        create_button : Button::new(ImString::new(InitialState::LABEL_CREATE)),
        fonts,
    } }
}

struct CreateCampaignState {
    title : ImString,
    name_field : TextField,
    finish_button : Button,
    error_text : ImString,
    fonts : Fonts,
}

impl ApplicationState for CreateCampaignState {
    fn build_gui(mut self : Box<Self>, ui : &Ui) -> Box<dyn ApplicationState> {
        let title = &self.title;
        let name_field = &mut self.name_field;
        let finish_button = &mut self.finish_button;
        let error_text = &self.error_text;
        Window::new(title).size([200.0, 200.0], Condition::FirstUseEver).build(
            ui,
            || { 
                name_field.build_gui(ui); 
                finish_button.build_gui(ui);
                if !error_text.is_empty() {
                    ui.text(error_text);
                }
            }
        );        
        if finish_button.pressed() && !name_field.content().is_empty() {
            Box::new(EditCampaignState::new(name_field.content().to_owned(), self.fonts))
        } else if finish_button.pressed() {
            self.error_text = ImString::new(Application::NON_EMPTY_NAME_MESSAGE);
            self
        } else {
            self
        }
    }
}

impl CreateCampaignState {
    pub fn new(fonts : Fonts) -> Self {
        CreateCampaignState {
            title : ImString::new(Application::CREATE_CAMPAIGN_TITLE),
            name_field : TextField::new(ImString::new(Application::NAME_LABEL)),
            finish_button : Button::new(ImString::new(Application::FINISH_LABEL)),
            error_text : ImString::new(""),
            fonts,
        }
    }
}


struct EditCampaignState {
    title : ImString,
    name_label : ImString,
    entities_label : ImString,
    current_entity : i32,
    create_entity_button : Button,
    edit_entity_button : Button,
    substates : Vec<Box<dyn ApplicationSubstate>>,
    campaign : Campaign,
    fonts : Fonts,
}

impl ApplicationState for EditCampaignState {
    fn build_gui(mut self : Box<Self>, ui : &Ui) -> Box<dyn ApplicationState> {
        let title = &self.title;
        let name_label = &self.name_label;
        let entities_label = &self.entities_label;
        let entity_names : Vec<ImString> = self.campaign.entities().keys().map(|name| { ImString::new(name) }).collect();
        let entity_names : Vec<&ImStr> = entity_names.iter().map(|name| { name.as_ref() }).collect();
        let current_entity = &mut self.current_entity;
        let create_entity_button = &mut self.create_entity_button;
        let edit_entity_button = &mut self.edit_entity_button;
        let fonts = &mut self.fonts;
        Window::new(title).size([300.0, 600.0], Condition::FirstUseEver).build(
            ui,
            || { 
                ui.text(name_label);
                ui.list_box(
                    entities_label, 
                    current_entity, 
                    &entity_names[..], 
                    10
                );
                create_entity_button.build_gui(ui);
                edit_entity_button.build_gui(ui);
            }
        );       
        if create_entity_button.pressed() {
            self.substates.push(Box::new(CreateEntityState::new(self.substates.len())));
        }  
        if edit_entity_button.pressed() && *current_entity != -1 {
            self.substates.push(Box::new(EditEntityState::new(entity_names[*current_entity as usize].to_owned(), &mut self.campaign)));
        }
        let mut new_substates = Vec::new();
        for substate in self.substates {
            let mut new_substate = substate.build_gui(ui, fonts);
            new_substate.persist(&mut self.campaign);
            if !new_substate.expired() {
                new_substates.push(new_substate);
            }
        }
        self.substates = new_substates;
        self
    }
}

impl EditCampaignState {
    pub fn new(name : ImString, fonts : Fonts) -> Self {
        EditCampaignState {
            title : ImString::new(Application::EDIT_CAMPAIGN_TITLE),
            name_label : ImString::new(format!("Campaign: {}", name.clone())),
            entities_label : ImString::new(Application::ENTITIES_LABEL),
            campaign : Campaign::new(name.to_string()),
            current_entity : -1,
            create_entity_button : Button::new(ImString::new(Application::CREATE_ENTITY_LABEL)),
            edit_entity_button : Button::new(ImString::new(Application::EDIT_ENTITY_LABEL)),
            substates : Vec::new(),
            fonts,
        }
    }
}

struct CreateEntityState {
    title : ImString,
    name_field : TextField,
    finish_button : Button,
    error_text : ImString,
    done : bool,
}

impl ApplicationSubstate for CreateEntityState {
    fn build_gui(mut self : Box<Self>, ui : &Ui, _fonts : &Fonts) -> Box<dyn ApplicationSubstate> {
        let title = &self.title;
        let name_field = &mut self.name_field;
        let finish_button = &mut self.finish_button;
        let error_text = &self.error_text;
        Window::new(title).size([200.0, 200.0], Condition::FirstUseEver).build(
            ui,
            || { 
                name_field.build_gui(ui);
                finish_button.build_gui(ui);
                if !error_text.is_empty() {
                    ui.text(error_text);
                }                
            }
        );  
        if finish_button.pressed() && !name_field.content().is_empty() {
            self.done = true;
        } else if finish_button.pressed() {
            self.error_text = ImString::new(Application::NON_EMPTY_NAME_MESSAGE);
        }
        self 
    }

    fn persist(&mut self, campaign : &mut Campaign) {
        if self.done {
            if let Err(_) = campaign.new_entity(self.name_field.content().to_string()) {
                self.done = false;
                self.error_text = ImString::new(Application::DUPLICATE_NAME_MESSAGE);
            }
        }
    }

    fn expired(&self) -> bool {
        self.done
    }
}

impl CreateEntityState {
    pub fn new(id : usize) -> Self {
        CreateEntityState {
            title : ImString::new(format!("{}##{}", Application::CREATE_ENTITY_LABEL, id)),
            name_field : TextField::new(ImString::new(Application::NAME_LABEL)),
            finish_button : Button::new(ImString::new(Application::FINISH_LABEL)),
            error_text : ImString::new(""),
            done : false,
        }
    }
}

struct EditEntityState {
    title : ImString,
    name : String,
    content : ImString,
    finish_button : Button,
    done : bool,
}

impl ApplicationSubstate for EditEntityState {
    fn build_gui(mut self : Box<Self>, ui : &Ui, fonts: &Fonts) -> Box<dyn ApplicationSubstate> {
        const TEXT_FIELD_SIZE : [f32; 2] = [200.0, 200.0];
        let title = &self.title;
        let content = &mut self.content;
        let finish_button = &mut self.finish_button;
        Window::new(title).size([800.0, 400.0], Condition::FirstUseEver).build(
            ui,
            || { 
                ui.input_text_multiline(&ImString::new(""), content, TEXT_FIELD_SIZE).resize_buffer(true).build();
                ui.same_line(220.0);
                markdown(ui, content.to_string(), fonts);
                ui.set_cursor_pos([TEXT_FIELD_SIZE[0], 0.0]);
                finish_button.build_gui(ui);
            }
        );  
        if self.finish_button.pressed() {
            self.done = true;
        }
        self
    }

    fn persist(&mut self, campaign : &mut Campaign) {
        if self.done {
            campaign.update_entity_content(&self.name, EntityContent{ text : self.content.to_string() }).unwrap();
        }
    }

    fn expired(&self) -> bool {
        self.done
    }
}

impl EditEntityState {
    pub fn new(name : ImString, campaign : &Campaign) -> Self {
        EditEntityState {
            title : ImString::new(format!("{}: {}", Application::EDIT_ENTITY_LABEL, name)),
            name : name.to_string(),
            content : ImString::new(campaign.entities().get(name.to_str()).unwrap().content().text.clone()),
            finish_button : Button::new(ImString::new(Application::FINISH_LABEL)),
            done : false,
        }
    }
}

pub struct Application {
    state : Box<dyn ApplicationState>,
}

impl Application {
    pub const MAIN_MENU_TITLE : &'static str = "Main menu";
    pub const CREATE_CAMPAIGN_TITLE : &'static str = "Create Campaign";
    pub const NAME_LABEL : &'static str = "Name";
    pub const FINISH_LABEL : &'static str = "Finish";
    pub const EDIT_CAMPAIGN_TITLE : &'static str = "Edit Campaign";
    pub const NON_EMPTY_NAME_MESSAGE : &'static str = "Name must not be empty";
    pub const ENTITIES_LABEL : &'static str = "Entities";
    pub const CREATE_ENTITY_LABEL : &'static str = "Create Entity";
    pub const DUPLICATE_NAME_MESSAGE : &'static str = "Duplicate names are not allowed";
    pub const EDIT_ENTITY_LABEL : &'static str = "Edit Entity";

    pub fn new(fonts : Fonts) -> Self {
        Application {
            state : Box::new(InitialState::new(fonts)),
        }
    }

    pub fn build_gui(&mut self, ui : &Ui) {
        let mut state : Box<dyn ApplicationState> = Box::new(EmptyState{});
        std::mem::swap(&mut self.state, &mut state);
        self.state = state.build_gui(ui);
    }
}