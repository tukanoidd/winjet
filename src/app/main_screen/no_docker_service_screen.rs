use std::sync::Arc;

use iced::{
    Length,
    widget::{button, center, column, container, horizontal_rule, rich_text, span, table, text},
};
use iced_fonts::{NERD_FONT, nerd};

use crate::{
    app::{AppElement, AppMsg},
    controller::{
        docker::{ContainerData, DockerContainerExt, DockerController},
        state::StateController,
    },
};

pub struct NoDockerServiceScreen;

impl NoDockerServiceScreen {
    pub fn view<'a>(
        &'a self,
        state: &'a StateController,
        docker: &'a DockerController,
    ) -> AppElement<'a> {
        let state_module = state.as_ref().unwrap();

        center(
            column![
                button(rich_text![
                    span::<(), _>(nerd::advanced_text::fa_plus().0)
                        .font(NERD_FONT)
                        .size(20.0),
                    span(" Create New Docker Service")
                ]),
                horizontal_rule(2),
                text("Choose One of The existing ones..."),
                center(
                    container(table(
                        [
                            table::column(text("Select"), |container: &ContainerData| button(
                                nerd::fa_check().size(20)
                            )
                            .on_press_maybe((!state_module.service_updating).then(|| {
                                AppMsg::CreateDockerServiceStateFromExisting(Arc::new(
                                    container.clone(),
                                ))
                            }))),
                            ContainerData::name_column(),
                            ContainerData::image_column(),
                        ],
                        docker.iter().flat_map(|d| d.containers.iter())
                    ))
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .padding(20)
                    .style(container::bordered_box),
                )
                .height(Length::Shrink)
            ]
            .max_width(800)
            .spacing(20),
        )
        .padding(20)
        .into()
    }
}
