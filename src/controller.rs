pub mod docker;
pub mod kvm;
pub mod state;

use std::sync::Arc;

use color_eyre::Result;
use derive_more::{Deref, DerefMut};
use iced::{
    Length,
    widget::{Space, row, text},
};
use iced_aw::spinner::Spinner;
use iced_fonts::nerd;
use smart_default::SmartDefault;

use crate::{
    app::{AppElement, AppMsg, AppTask},
    util::Arced,
};

#[derive(SmartDefault, Deref, DerefMut)]
pub struct Controller<Module>
where
    Module: ControllerModule,
{
    #[deref]
    #[deref_mut]
    module: Option<Module>,
    pub loading: bool,
}

impl<Module> Controller<Module>
where
    Module: ControllerModule,
{
    pub fn load(
        &mut self,
        input: Module::Init,
        to_app_msg: impl FnOnce(Arc<Result<Module>>) -> AppMsg + Send + 'static,
    ) -> AppTask {
        self.loading = true;
        AppTask::perform(async move { Module::init(input).await }, to_app_msg)
    }

    pub fn loaded(
        &mut self,
        res: Arc<Result<Module>>,
        on_success: impl FnOnce() -> AppTask,
    ) -> AppTask {
        self.loading = false;

        match Arc::into_inner(res)
            .expect("Logic error! Shouldn't have any refcount on this Arc at this point!")
        {
            Ok(val) => self.module = Some(val),
            Err(err) => {
                tracing::error!("Failed to load module {}: {err}", Module::NAME);
            }
        }

        on_success()
    }

    pub fn state_widget(&self) -> AppElement<'_> {
        let row =
            row![text(Module::NAME), Space::new(Length::Fill, Length::Shrink)].width(Length::Fill);

        let row = match self.module.is_some() {
            true => row.push(nerd::fa_check().style(text::success)),
            false => match self.loading {
                true => row.push(Spinner::new()),
                false => row.push(nerd::cod_error().style(text::danger)),
            },
        };

        row.into()
    }
}

pub trait ControllerModule: Send + Sync + 'static {
    const NAME: &str;

    type Init: Send + 'static;

    fn init(input: Self::Init) -> impl Future<Output = Arc<Result<Self>>> + Send
    where
        Self: Sized,
    {
        async move { Self::init_impl(input).await.arced() }
    }

    fn init_impl(input: Self::Init) -> impl Future<Output = Result<Self>> + Send
    where
        Self: Sized;
}
