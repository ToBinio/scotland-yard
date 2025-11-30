use crate::{
    character::{ActionTypeTrait, Character},
    data::StationType,
    event::DetectiveActionType,
};

pub struct Action {
    pub station: u8,
    pub action_type: DetectiveActionType,
}

impl ActionTypeTrait for DetectiveActionType {
    fn matches(&self, station_type: &StationType) -> bool {
        match station_type {
            StationType::Taxi => matches!(self, DetectiveActionType::Taxi),
            StationType::Bus => matches!(self, DetectiveActionType::Bus),
            StationType::Underground => matches!(self, DetectiveActionType::Underground),
            StationType::Water => false,
        }
    }
}

pub struct Detective {
    color: String,
    start_station_id: u8,
    actions: Vec<Action>,
}

impl Character for Detective {
    type Action = Action;

    type ActionType = DetectiveActionType;

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

    fn can_do_action(&self, action: &Self::ActionType) -> bool {
        match action {
            DetectiveActionType::Taxi => self.taxi() > 0,
            DetectiveActionType::Bus => self.bus() > 0,
            DetectiveActionType::Underground => self.underground() > 0,
        }
    }
}

impl Detective {
    pub fn new(station_id: u8, color: String) -> Self {
        Self {
            color,
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
            .filter(|step| matches!(step.action_type, DetectiveActionType::Taxi))
            .count() as u8;

        //todo: revert to 10
        50 - count
    }

    pub fn bus(&self) -> u8 {
        let count = self
            .actions
            .iter()
            .filter(|step| matches!(step.action_type, DetectiveActionType::Bus))
            .count() as u8;

        8 - count
    }

    pub fn underground(&self) -> u8 {
        let count = self
            .actions
            .iter()
            .filter(|step| matches!(step.action_type, DetectiveActionType::Underground))
            .count() as u8;

        4 - count
    }
}
