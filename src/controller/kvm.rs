use color_eyre::Result;
use kvm_ioctls::Kvm;

use crate::controller::{Controller, ControllerModule};

pub type KVMController = Controller<KVMModule>;

#[derive(Debug)]
pub struct KVMModule {
    kvm: Kvm,
}

impl ControllerModule for KVMModule {
    const NAME: &str = "KVM";

    type Init = ();

    async fn init_impl(_: Self::Init) -> Result<Self>
    where
        Self: Sized,
    {
        let kvm = Kvm::new()?;

        Ok(Self { kvm })
    }
}
