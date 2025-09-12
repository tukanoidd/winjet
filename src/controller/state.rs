use std::sync::Arc;

use bollard::secret::{DeviceMapping, Port, PortTypeEnum, RestartPolicy, RestartPolicyNameEnum};
use color_eyre::Result;
use directories::ProjectDirs;
use iced::futures::FutureExt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use smart_default::SmartDefault;
use surrealdb::{RecordId, Surreal, Uuid, engine::local::Db};
use surrealdb_extras::{SurrealExt, SurrealQuery, SurrealTable};

use crate::{
    app::{AppMsg, AppTask},
    controller::{Controller, ControllerModule},
    util::Arced,
};

pub type DB = Surreal<Db>;

pub type StateController = Controller<StateModule>;

#[derive(Debug)]
pub struct StateModule {
    db: DB,

    pub service: Option<DockerServiceState>,
    pub service_loading: bool,
    pub service_updating: bool,
    pub service_exists_db: bool,
}

impl ControllerModule for StateModule {
    const NAME: &str = "State";

    type Init = ProjectDirs;

    async fn init_impl(dirs: ProjectDirs) -> Result<Self> {
        let db_dir = dirs.data_local_dir().join("state");

        let db = DB::new(db_dir).await?;
        db.use_ns_db_checked("winjet", "state", vec![]).await?;

        Ok(Self {
            db,

            service: None,
            service_loading: false,
            service_updating: false,
            service_exists_db: false,
        })
    }
}

impl StateModule {
    pub fn try_load_service(&mut self) -> AppTask {
        self.service_loading = true;
        AppTask::perform(
            GetDockerService.execute(self.db.clone()).map(Arced::arced),
            AppMsg::LoadDockerServiceStateRes,
        )
    }

    pub fn check_set_service(&mut self, service: Arc<Result<Option<DockerServiceState>>>) {
        self.service_loading = false;
        self.service_updating = false;

        match Arc::into_inner(service).expect("Logic error!") {
            Ok(val) => {
                self.service = val;
                self.service_exists_db = self.service.is_some();
            }
            Err(err) => tracing::error!("Failed to load docker service: {err}"),
        }
    }

    pub fn update_service_db(&mut self) -> AppTask {
        match &self.service {
            Some(service) => {
                let exists = self.service_exists_db;
                let db = self.db.clone();
                let service = service.clone();

                self.service_updating = true;

                AppTask::perform(
                    async move {
                        match exists {
                            true => {
                                db.upsert::<Option<_>>(service.id.clone())
                                    .content(service)
                                    .await
                            }
                            false => db.create("container").content(service).await,
                        }
                        .map_err(color_eyre::Report::from)
                        .arced()
                    },
                    AppMsg::LoadDockerServiceStateRes,
                )
            }
            None => AppTask::none(),
        }
    }
}

#[derive(SmartDefault, Debug, Clone, Serialize, Deserialize, SurrealTable)]
#[table(db = container)]
#[serde(default)]
pub struct DockerServiceState {
    #[default(RecordId::from_table_key("container", Uuid::now_v7()))]
    pub id: RecordId,
    #[default = "dockurr/windows"]
    pub image: String,
    #[default = "windows"]
    pub container_name: String,
    #[default(Map::from_iter([
        ("VERSION".into(), "11".into())
    ]))]
    pub environment: Map<String, Value>,
    #[default(
        ["/dev/kvm", "/dev/net/tun"]
            .map(|x| DeviceMapping {
                path_on_host: Some(x.into()),
                ..Default::default()
            })
            .into_iter()
            .collect()
    )]
    pub devices: Vec<DeviceMapping>,
    #[default(vec!["NET_ADMIN".into()])]
    pub cap_add: Vec<String>,
    #[default(
        vec![
            Port {
                private_port: 8006,
                public_port: Some(8006),
                ..Default::default()
            },
            Port {
                private_port: 3389,
                public_port: Some(3389),
                typ: Some(PortTypeEnum::TCP),
                ..Default::default()
            },
            Port {
                private_port: 3389,
                public_port: Some(3389),
                typ: Some(PortTypeEnum::UDP),
                ..Default::default()
            },
        ]
    )]
    pub ports: Vec<Port>,
    #[default(vec![format!(
        "{}/windows:/storage",
        std::env::home_dir()
            .unwrap()
            .to_string_lossy()
    )])]
    pub volumes: Vec<String>,
    #[default(RestartPolicy {
        name: Some(RestartPolicyNameEnum::ALWAYS),
        ..Default::default()
    })]
    pub restart: RestartPolicy,
    #[default = "2m"]
    pub stop_grace_period: String,
}

#[derive(SurrealQuery)]
#[query(
    output = "Option<DockerServiceState>",
    error = "color_eyre::Report",
    sql = "SELECT * FROM ONLY container"
)]
struct GetDockerService;
