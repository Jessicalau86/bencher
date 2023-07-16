use async_trait::async_trait;

use crate::{bencher::sub::SubCmd, parser::project::benchmark::CliBenchmark, CliError};

mod create;
mod list;
mod update;
mod view;

#[derive(Debug)]
pub enum Benchmark {
    List(list::List),
    Create(create::Create),
    View(view::View),
    Update(update::Update),
}

impl TryFrom<CliBenchmark> for Benchmark {
    type Error = CliError;

    fn try_from(benchmark: CliBenchmark) -> Result<Self, Self::Error> {
        Ok(match benchmark {
            CliBenchmark::List(list) => Self::List(list.try_into()?),
            CliBenchmark::Create(create) => Self::Create(create.try_into()?),
            CliBenchmark::View(view) => Self::View(view.try_into()?),
            CliBenchmark::Update(update) => Self::Update(update.try_into()?),
        })
    }
}

#[async_trait]
impl SubCmd for Benchmark {
    async fn exec(&self) -> Result<(), CliError> {
        match self {
            Self::List(list) => list.exec().await,
            Self::Create(create) => create.exec().await,
            Self::View(create) => create.exec().await,
            Self::Update(update) => update.exec().await,
        }
    }
}
