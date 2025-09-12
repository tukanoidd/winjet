use iced::{
    Length,
    widget::{
        Space, button, center, column, container, horizontal_rule, rich_text, row, span, text,
    },
};
use iced_fonts::{NERD_FONT, nerd};

use crate::{
    app::{AppElement, AppMsg},
    controller::{docker::DockerController, kvm::KVMController, state::StateController},
};

pub struct SetupScreen;

impl SetupScreen {
    pub fn view<'a>(
        &'a self,
        state: &'a StateController,
        docker: &'a DockerController,
        kvm: &'a KVMController,
    ) -> AppElement<'a> {
        center(
            column![
                state.state_widget(),
                horizontal_rule(2),
                docker.state_widget(),
                horizontal_rule(2),
                kvm.state_widget(),
                horizontal_rule(2),
                Space::new(Length::Shrink, Length::Fixed(40.0)),
                row![
                    button(rich_text![
                        span::<(), _>(nerd::advanced_text::fa_rotate_right().0)
                            .font(NERD_FONT)
                            .size(20),
                        span(" Retry").size(20)
                    ])
                    .on_press_maybe(
                        (!state.loading && !docker.loading && !kvm.loading)
                            .then_some(AppMsg::RetryInit)
                    ),
                    button(text("Next").size(20)).on_press_maybe(
                        ([
                            (state.loading, state.is_some()),
                            (docker.loading, docker.is_some()),
                            (kvm.loading, kvm.is_some()),
                        ]
                        .into_iter()
                        .all(|(loading, exists)| !loading && exists))
                        .then_some(AppMsg::DoneSetup)
                    )
                ]
                .spacing(10)
            ]
            .spacing(10)
            .width(Length::Fixed(800.0)),
        )
        .style(container::bordered_box)
        .into()
    }
}
