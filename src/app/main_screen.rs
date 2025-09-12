mod no_docker_service_screen;

use iced::{
    Length,
    widget::{center, text},
};
use iced_aw::Spinner;

use crate::{
    app::{AppElement, main_screen::no_docker_service_screen::NoDockerServiceScreen},
    controller::{docker::DockerController, state::StateController},
};

pub struct MainScreen;

impl MainScreen {
    pub fn view<'a>(
        &'a self,
        state: &'a StateController,
        docker: &'a DockerController,
    ) -> AppElement<'a> {
        let state_module = state.as_ref().unwrap();

        let Some(service) = &state_module.service else {
            return NoDockerServiceScreen.view(state, docker);
        };

        if !state_module.service_exists_db {
            return center(
                Spinner::new()
                    .width(Length::Fixed(50.0))
                    .height(Length::Fixed(50.0)),
            )
            .into();
        }

        center(text(format!("{} is ALIVE!", service.container_name))).into()
    }
}
