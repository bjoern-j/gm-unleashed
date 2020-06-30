use std::collections::{ HashMap };

pub type Entities = HashMap<String, Entity>;

pub struct Campaign {
    name : String,
    entities : Entities,
}

pub struct EntityContent {
    pub text : String,
}

impl EntityContent {
    pub fn new() -> Self {
        EntityContent {
            text : String::new(),
        }
    }
}

impl Campaign {
    pub fn new(name : String) -> Self {
        Campaign {
            name,
            entities : Entities::new(),
        }
    }

    pub fn new_entity(&mut self, name : String) -> Result<(), NewEntityError> {
        if self.entities.contains_key(&name) {
            Err(NewEntityError::DuplicateName)
        } else {
            self.entities.insert(name.clone(), Entity::new(name));
            Ok(())
        }
    }

    pub fn update_entity_content(&mut self, name : &str, content : EntityContent) -> Result<(), UpdateEntityError> {
        match self.entities.get_mut(name) {
            Some(ent) => { 
                ent.content = content; 
                Ok(())
            }
            None => Err(UpdateEntityError::NoEntity)
        }
    }

    pub fn entities(&self) -> &Entities { &self.entities }
    pub fn name(&self) -> &str { &self.name }
}

pub struct Entity {
    name : String,
    content : EntityContent,
}

impl Entity {
    pub fn new(name : String) -> Self {
        Entity {
            name,
            content : EntityContent::new(),
        }
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn content(&self) -> &EntityContent { &self.content }
}

#[derive(PartialEq, Eq, Debug)]
pub enum NewEntityError {
    DuplicateName,
}

#[derive(PartialEq, Eq, Debug)]
pub enum UpdateEntityError {
    NoEntity,
}

#[cfg(test)]
mod campaign_tests {
    use super::*;
    #[test]
    fn name_is_stored() {
        assert_eq!(Campaign::new("C".to_string()).name(), "C");
    }
    #[test]
    fn create_entity_with_name() {
        let mut camp = Campaign::new("C".to_string());
        camp.new_entity("E".to_string()).unwrap();
        assert_eq!(camp.entities().get("E").unwrap().name(), "E");
    }
    #[test]
    fn cannot_create_entity_with_same_name() {
        let mut camp = Campaign::new("C".to_string());
        camp.new_entity("E".to_string()).unwrap();
        assert_eq!(camp.new_entity("E".to_string()), Err(NewEntityError::DuplicateName));
    }
    #[test]
    fn content_is_persisted() {
        let mut camp = Campaign::new("C".to_string());
        camp.new_entity("E".to_string()).unwrap();
        camp.update_entity_content("E", EntityContent{ text : "Hello world".to_string() }).unwrap();
        assert_eq!(camp.entities().get("E").unwrap().content().text, "Hello world");
    }
    #[test]
    fn cannot_update_nonexistent_entity() {
        let mut camp = Campaign::new("C".to_string());
        assert_eq!(camp.update_entity_content("E", EntityContent{ text : "".to_string() }), Err(UpdateEntityError::NoEntity));
    }
}