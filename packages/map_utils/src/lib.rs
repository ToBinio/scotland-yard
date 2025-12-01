use game::{
    data::{Connection, StationType},
    event::{DetectiveActionType, DetectiveTransportData, MisterXAbilityData, MisterXActionType},
};

//TODO: handle hidden on normal paths and double moves
pub fn all_valid_mister_x_moves(
    connections: &[Connection],
    station: u8,
    ability: &MisterXAbilityData,
) -> Vec<(u8, MisterXActionType)> {
    connections
        .iter()
        .filter(|c| c.from == station || c.to == station)
        .filter(|c| match c.mode {
            StationType::Taxi => true,
            StationType::Bus => true,
            StationType::Underground => true,
            StationType::Water => ability.hidden > 0,
        })
        .map(|c| {
            let action_type = match c.mode {
                StationType::Taxi => MisterXActionType::Taxi,
                StationType::Bus => MisterXActionType::Bus,
                StationType::Underground => MisterXActionType::Underground,
                StationType::Water => MisterXActionType::Hidden,
            };

            if c.from == station {
                (c.to, action_type)
            } else {
                (c.from, action_type)
            }
        })
        .collect()
}

pub fn all_valid_detective_moves(
    connections: &[Connection],
    station: u8,
    transport: &DetectiveTransportData,
) -> Vec<(u8, DetectiveActionType)> {
    connections
        .iter()
        .filter(|c| c.from == station || c.to == station)
        .filter(|c| match c.mode {
            StationType::Taxi => transport.taxi > 0,
            StationType::Bus => transport.bus > 0,
            StationType::Underground => transport.underground > 0,
            StationType::Water => false,
        })
        .map(|c| {
            let action_type = match c.mode {
                StationType::Taxi => DetectiveActionType::Taxi,
                StationType::Bus => DetectiveActionType::Bus,
                StationType::Underground => DetectiveActionType::Underground,
                StationType::Water => unreachable!(),
            };

            if c.from == station {
                (c.to, action_type)
            } else {
                (c.from, action_type)
            }
        })
        .collect()
}
