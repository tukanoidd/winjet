mod main_screen;
mod setup_screen;

use std::sync::Arc;

use color_eyre::Result;
use directories::ProjectDirs;

use crate::{
    app::{main_screen::MainScreen, setup_screen::SetupScreen},
    controller::{
        docker::{ContainerData, DockerContainerExt, DockerController, DockerModule},
        kvm::{KVMController, KVMModule},
        state::{DockerServiceState, StateController, StateModule},
    },
};

pub type AppTask = iced::Task<AppMsg>;
pub type AppSubscription = iced::Subscription<AppMsg>;

pub type AppTheme = iced::Theme;
pub type AppRenderer = iced::Renderer;
pub type AppElement<'a> = iced::Element<'a, AppMsg, AppTheme, AppRenderer>;

pub struct App {
    project_dirs: ProjectDirs,

    screen: AppScreen,

    state: StateController,
    docker: DockerController,
    kvm: KVMController,
}

impl App {
    pub fn new(dirs: ProjectDirs) -> (Self, AppTask) {
        let res = Self {
            project_dirs: dirs,

            screen: AppScreen::Setup(SetupScreen),

            state: StateController::default(),
            docker: DockerController::default(),
            kvm: KVMController::default(),
        };
        let task = AppTask::batch([
            AppTask::done(AppMsg::InitState),
            AppTask::done(AppMsg::InitDocker),
            AppTask::done(AppMsg::InitKVM),
        ]);

        (res, task)
    }

    pub fn update(&mut self, msg: AppMsg) -> AppTask {
        match msg {
            AppMsg::InitState => {
                return self
                    .state
                    .load(self.project_dirs.clone(), AppMsg::InitStateRes);
            }
            AppMsg::InitStateRes(res) => {
                return self
                    .state
                    .loaded(res, || AppTask::done(AppMsg::LoadDockerServiceState));
            }

            AppMsg::InitDocker => return self.docker.load((), AppMsg::InitDockerRes),
            AppMsg::InitDockerRes(res) => return self.docker.loaded(res, AppTask::none),

            AppMsg::InitKVM => return self.kvm.load((), AppMsg::InitKVMRes),
            AppMsg::InitKVMRes(res) => return self.kvm.loaded(res, AppTask::none),

            AppMsg::RetryInit => {
                let mut tasks = vec![];

                if self.state.is_none() {
                    tasks.push(AppTask::done(AppMsg::InitState));
                }

                if self.docker.is_none() {
                    tasks.push(AppTask::done(AppMsg::InitDocker));
                }

                if self.kvm.is_none() {
                    tasks.push(AppTask::done(AppMsg::InitKVM));
                }

                if !tasks.is_empty() {
                    return AppTask::batch(tasks);
                }
            }
            AppMsg::DoneSetup => {
                self.screen = AppScreen::Main(MainScreen);
            }

            AppMsg::CreateDockerServiceStateFromExisting(data) => {
                self.state.as_mut().unwrap().service =
                    Some(Arc::into_inner(data).unwrap().into_service());
                return AppTask::done(AppMsg::UpdateDockerServiceState);
            }
            AppMsg::CreatedDockerServiceState(state) => {
                self.state.as_mut().unwrap().service = Some(Arc::into_inner(state).unwrap());
                return AppTask::done(AppMsg::UpdateDockerServiceState);
            }

            AppMsg::LoadDockerServiceState => {
                return self.state.as_mut().unwrap().try_load_service();
            }
            AppMsg::LoadDockerServiceStateRes(res) => {
                self.state.as_mut().unwrap().check_set_service(res)
            }

            AppMsg::UpdateDockerServiceState => {
                return self.state.as_mut().unwrap().update_service_db();
            }
        }

        AppTask::none()
    }

    pub fn subscription(&self) -> AppSubscription {
        AppSubscription::none()
    }

    pub fn view(&self) -> AppElement<'_> {
        self.screen.view(&self.state, &self.docker, &self.kvm)
    }

    pub fn theme(&self) -> AppTheme {
        AppTheme::TokyoNight
    }
}

enum AppScreen {
    Setup(SetupScreen),
    Main(MainScreen),
}

impl AppScreen {
    fn view<'a>(
        &'a self,
        state: &'a StateController,
        docker: &'a DockerController,
        kvm: &'a KVMController,
    ) -> AppElement<'a> {
        match self {
            Self::Setup(setup_screen) => setup_screen.view(state, docker, kvm),
            Self::Main(main_screen) => main_screen.view(state, docker),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    InitState,
    InitStateRes(Arc<Result<StateModule>>),

    InitDocker,
    InitDockerRes(Arc<Result<DockerModule>>),

    InitKVM,
    InitKVMRes(Arc<Result<KVMModule>>),

    RetryInit,
    DoneSetup,

    LoadDockerServiceState,
    LoadDockerServiceStateRes(Arc<Result<Option<DockerServiceState>>>),

    CreateDockerServiceStateFromExisting(Arc<ContainerData>),
    CreatedDockerServiceState(Arc<DockerServiceState>),

    UpdateDockerServiceState,
}
