use std::collections::HashMap;

use bollard::{
    Docker,
    query_parameters::{InspectContainerOptions, ListContainersOptionsBuilder},
    secret::{ContainerInspectResponse, ContainerSummary, DeviceMapping, Port, RestartPolicy},
};
use color_eyre::Result;
use derive_more::AsRef;
use tokio::task::JoinSet;

use crate::{
    app::{AppMsg, AppRenderer, AppTheme},
    controller::{Controller, ControllerModule, state::DockerServiceState},
};

pub type DockerController = Controller<DockerModule>;

#[derive(Debug)]
pub struct DockerModule {
    client: Docker,

    pub containers: Vec<ContainerData>,
}

impl ControllerModule for DockerModule {
    const NAME: &str = "Docker";

    type Init = ();

    async fn init_impl(_: Self::Init) -> Result<Self> {
        let client = Docker::connect_with_defaults()?;

        let containers = client
            .list_containers(Some(
                ListContainersOptionsBuilder::new()
                    .all(true)
                    .filters(&HashMap::from_iter([("ancestor", vec!["dockurr/windows"])]))
                    .build(),
            ))
            .await?
            .into_iter()
            .fold(JoinSet::new(), |mut join_set, summary| {
                let client = client.clone();

                join_set.spawn(async move {
                    let specs = client
                        .inspect_container(&summary.name(), Option::<InspectContainerOptions>::None)
                        .await?;

                    Result::Ok(ContainerData { summary, specs })
                });
                join_set
            })
            .join_all()
            .await
            .into_iter()
            .flat_map(|r| {
                r.inspect_err(|err: &color_eyre::Report| {
                    tracing::error!("Failed to load container information: {err}")
                })
            })
            .collect();

        Ok(Self { client, containers })
    }
}

macro_rules! column_fn {
    ($($name:ident),+) => {
        pastey::paste! {$(
            fn $name(&self) -> String;

            fn [< $name _column >]<'a>() -> iced::widget::table::Column<'a, 'a, &'a Self, AppMsg, AppTheme, AppRenderer> {
                iced::widget::table::column(
                    iced::widget::text(stringify!([< $name:camel >])),
                    |container: &Self| iced::widget::text(container.$name())
                )
            }
        )+}
    };
}

pub trait ContainerSummaryExt {
    fn name(&self) -> String;
}

impl ContainerSummaryExt for ContainerSummary {
    fn name(&self) -> String {
        self.names
            .as_ref()
            .and_then(|n| n.first().map(|x| x.trim_start_matches("/").into()))
            .unwrap_or_else(|| "Unknown".into())
    }
}

#[derive(Clone, Debug, AsRef)]
pub struct ContainerData {
    #[as_ref]
    summary: ContainerSummary,
    #[as_ref]
    specs: ContainerInspectResponse,
}

pub trait DockerContainerExt {
    column_fn! { name, image }

    fn env(&self) -> serde_json::Map<String, serde_json::Value>;
    fn devices(&self) -> Vec<DeviceMapping>;
    fn cap_add(&self) -> Vec<String>;
    fn ports(&self) -> Vec<Port>;
    fn volumes(&self) -> Vec<String>;
    fn restart(&self) -> RestartPolicy;

    fn into_service(self) -> DockerServiceState;
}

impl<C> DockerContainerExt for C
where
    C: AsRef<ContainerSummary>,
    C: AsRef<ContainerInspectResponse>,
{
    fn name(&self) -> String {
        AsRef::<ContainerSummary>::as_ref(&self).name()
    }

    fn image(&self) -> String {
        AsRef::<ContainerSummary>::as_ref(&self)
            .image
            .clone()
            .unwrap_or_else(|| "Unknown".into())
    }

    fn env(&self) -> serde_json::Map<String, serde_json::Value> {
        serde_json::Map::from_iter(
            AsRef::<ContainerInspectResponse>::as_ref(&self)
                .config
                .iter()
                .flat_map(|c| c.env.iter())
                .flat_map(|x| {
                    x.iter().map(|x| {
                        let mut split = x.split("=");

                        Some((
                            split.next()?.into(),
                            serde_json::from_str(split.next()?).ok()?,
                        ))
                    })
                })
                .flatten(),
        )
    }

    fn devices(&self) -> Vec<DeviceMapping> {
        AsRef::<ContainerInspectResponse>::as_ref(&self)
            .host_config
            .iter()
            .flat_map(|x| x.devices.iter())
            .flat_map(|x| x.iter().cloned())
            .collect()
    }

    fn cap_add(&self) -> Vec<String> {
        AsRef::<ContainerInspectResponse>::as_ref(&self)
            .host_config
            .as_ref()
            .and_then(|x| x.cap_add.clone())
            .unwrap_or_default()
    }

    fn ports(&self) -> Vec<Port> {
        AsRef::<ContainerSummary>::as_ref(&self)
            .ports
            .clone()
            .unwrap_or_default()
    }

    fn volumes(&self) -> Vec<String> {
        AsRef::<ContainerInspectResponse>::as_ref(&self)
            .config
            .as_ref()
            .and_then(|x| x.volumes.as_ref())
            .map(|x| x.keys().cloned().collect())
            .unwrap_or_default()
    }

    fn restart(&self) -> RestartPolicy {
        AsRef::<ContainerInspectResponse>::as_ref(&self)
            .host_config
            .as_ref()
            .and_then(|x| x.restart_policy.clone())
            .unwrap_or_else(|| serde_json::from_str("always").unwrap())
    }

    fn into_service(self) -> DockerServiceState {
        DockerServiceState {
            image: self.image(),
            container_name: self.name(),
            environment: self.env(),
            devices: self.devices(),
            cap_add: self.cap_add(),
            ports: self.ports(),
            volumes: self.volumes(),
            restart: self.restart(),
            // TODO: figure this one out
            // stop_grace_period: (),
            ..Default::default()
        }
    }
}
