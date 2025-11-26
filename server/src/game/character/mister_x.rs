use serde::{Deserialize, Serialize};

use crate::game::character::Character;

pub struct MoveData {
    pub station: u8,
    pub action_type: ActionType,
}

pub enum Action {
    Single(MoveData),
    Double(MoveData, MoveData),
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Taxi,
    Bus,
    Underground,
    Hidden,
}

pub struct MisterX {
    start_station_id: u8,
    actions: Vec<Action>,
}

impl Character for MisterX {
    type Action = Action;

    type ActionType = ActionType;

    fn station_id(&self) -> u8 {
        match self.actions.last() {
            Some(step) => match step {
                Action::Single(step) => step.station,
                Action::Double(_, step) => step.station,
            },
            None => self.start_station_id,
        }
    }

    fn action_types(&self) -> Vec<Self::ActionType> {
        self.actions
            .iter()
            .flat_map(|step| match step {
                Action::Single(step) => vec![step.action_type.clone()],
                Action::Double(step1, step2) => {
                    vec![step1.action_type.clone(), step2.action_type.clone()]
                }
            })
            .collect()
    }

    /// trim number of actions so they are no longer than "target"
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

impl MisterX {
    pub fn new(station_id: u8) -> Self {
        Self {
            start_station_id: station_id,
            actions: Vec::new(),
        }
    }

    /// Returns number of aviable hidden moves
    pub fn hidden(&self) -> u8 {
        let count = self
            .action_types()
            .into_iter()
            .filter(|step| step.eq(&ActionType::Hidden))
            .count() as u8;

        2 - count
    }

    /// Returns number of aviable double moves
    pub fn double_moves(&self) -> u8 {
        let count = self
            .actions
            .iter()
            .filter(|step| matches!(step, Action::Double(_, _)))
            .count() as u8;

        2 - count
    }
}
