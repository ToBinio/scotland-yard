use serde::{Deserialize, Serialize};

use crate::game::character::Character;

pub struct Action {
    pub station: u8,
    pub action_type: ActionType,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Taxi,
    Bus,
    Underground,
}

pub struct Detective {
    color: String,
    start_station_id: u8,
    actions: Vec<Action>,
}

impl Character for Detective {
    type Action = Action;

    type ActionType = ActionType;

    fn station_id(&self) -> u8 {
        match self.actions.last() {
            Some(step) => step.station,
            None => self.start_station_id,
        }
    }

    fn action_types(&self) -> Vec<Self::ActionType> {
        self.actions
            .iter()
            .map(|step| step.action_type.clone())
            .collect()
    }

    fn trim_actions(&mut self, target: usize) {
        if self.actions.len() > target {
            self.actions.pop();
        }
    }

    fn add_action(&mut self, action: Self::Action) {
        self.actions.push(action);
    }

    fn actions(&self) -> &Vec<Self::Action> {
        &self.actions
    }
}

impl Detective {
    pub fn new(station_id: u8, color: &str) -> Self {
        Self {
            color: color.to_string(),
            start_station_id: station_id,
            actions: Vec::new(),
        }
    }

    pub fn color(&self) -> &str {
        &self.color
    }

    pub fn taxi(&self) -> u8 {
        let count = self
            .actions
            .iter()
            .filter(|step| matches!(step.action_type, ActionType::Taxi))
            .count() as u8;

        10 - count
    }

    pub fn bus(&self) -> u8 {
        let count = self
            .actions
            .iter()
            .filter(|step| matches!(step.action_type, ActionType::Bus))
            .count() as u8;

        8 - count
    }

    pub fn underground(&self) -> u8 {
        let count = self
            .actions
            .iter()
            .filter(|step| matches!(step.action_type, ActionType::Underground))
            .count() as u8;

        4 - count
    }
}
