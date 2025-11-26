pub mod detective;
pub mod mister_x;

pub trait Character {
    type Action;
    type ActionType;

    fn station_id(&self) -> u8;

    fn action_types(&self) -> Vec<Self::ActionType>;

    /// trim number of actions so they are no longer than "target"
    fn trim_actions(&mut self, target: usize);

    fn add_action(&mut self, action: Self::Action);

    fn actions(&self) -> &Vec<Self::Action>;
}
