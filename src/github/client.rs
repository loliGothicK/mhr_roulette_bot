use once_cell::sync::OnceCell;
use octocrab::OctocrabBuilder;

pub static GITHUB_CLIENT: OnceCell<Client> = OnceCell::new();

#[derive(Debug)]
pub struct Client {
    client: octocrab::Octocrab,
}

impl Client {
    fn init<Builder>(builder: Builder) -> anyhow::Result<(), !> 
        where Builder: FnOnce(OctocrabBuilder) -> OctocrabBuilder
    {
        GITHUB_CLIENT.set(Client{ client: builder(OctocrabBuilder::new()).build().unwrap() }).unwrap();
        Ok(())
    }

    fn global(&self) -> &'static octocrab::Octocrab {
        &GITHUB_CLIENT.get().expect("GitHub client is not initialized").client
    }
}
