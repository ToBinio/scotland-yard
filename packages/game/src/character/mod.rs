use crate::data::StationType;

pub mod detective;
pub mod mister_x;

pub trait Character {
    type Action;
    type ActionType: ActionTypeTrait;

    fn start_station(&self) -> u8;

    fn station_id(&self) -> u8;

    fn can_do_action(&self, action: &Self::ActionType) -> bool;

    fn action_types(&self) -> Vec<Self::ActionType>;

    /// trim number of actions so they are no longer than "target"
    fn trim_actions(&mut self, target: usize);

    fn add_action(&mut self, action: Self::Action);

    fn actions(&self) -> &Vec<Self::Action>;
}

pub trait ActionTypeTrait {
    fn matches(&self, station_type: &StationType) -> bool;
}
