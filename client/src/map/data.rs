use std::time::Duration;

use futures_timer::Delay;
use game::data::{Connection, Station};
use gpui::Context;

#[derive(Debug, Clone, Default)]
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
    pub fn init(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, app| {
            let station_task = app.spawn(async |_| {
                loop {
                    match ureq::get("http://localhost:8081/map/stations")
                        .call()
                        .and_then(|mut response| response.body_mut().read_json::<Vec<Station>>())
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
                    match ureq::get("http://localhost:8081/map/connections")
                        .call()
                        .and_then(|mut response| response.body_mut().read_json::<Vec<Connection>>())
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

            this.update(app, |data, app| {
                data.stations = stations;
                data.connections = connections;
                app.notify()
            })
            .unwrap();
        })
        .detach();
    }
}
