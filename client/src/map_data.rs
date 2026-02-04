use std::time::Duration;

use futures_timer::Delay;
use game::data::{Connection, Station};
use gpui::{App, AppContext, Entity};

#[derive(Debug, Default)]
pub struct MapData {
    stations: Vec<Station>,
    connections: Vec<Connection>,
}

impl MapData {
    pub fn stations(&self) -> &Vec<Station> {
        &self.stations
    }

    pub fn connections(&self) -> &Vec<Connection> {
        &self.connections
    }
}

impl MapData {
    pub fn new(cx: &mut App) -> Entity<Self> {
        let map_data = cx.new(|_| MapData::default());

        let data = map_data.clone();
        cx.spawn(async move |app| {
            let station_task = app.spawn(async |_| {
                loop {
                    match reqwest::blocking::get("http://localhost:8081/map/stations")
                        .and_then(|response| response.json::<Vec<Station>>())
                    {
                        Ok(stations) => return stations,
                        Err(err) => {
                            println!("Failed to fetch stations: {} - retrying in 1 second", err);
                            Delay::new(Duration::from_millis(1000)).await;
                        }
                    }
                }
            });

            let connection_task = app.spawn(async |_| {
                loop {
                    match reqwest::blocking::get("http://localhost:8081/map/connections")
                        .and_then(|response| response.json::<Vec<Connection>>())
                    {
                        Ok(connections) => return connections,
                        Err(err) => {
                            println!(
                                "Failed to fetch connections: {} - retrying in 1 second",
                                err
                            );
                            Delay::new(Duration::from_millis(1000)).await;
                        }
                    }
                }
            });

            let stations = station_task.await;
            let connections = connection_task.await;

            data.update(app, |data, app| {
                data.stations = stations;
                data.connections = connections;
                app.notify()
            })
            .unwrap();
        })
        .detach();

        map_data
    }
}
