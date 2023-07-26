use std::process::Command;

use crate::parser::CliTypeshare;

#[derive(Debug)]
pub struct Typeshare {}

impl TryFrom<CliTypeshare> for Typeshare {
    type Error = anyhow::Error;

    fn try_from(_typeshare: CliTypeshare) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl Typeshare {
    pub async fn exec(&self) -> anyhow::Result<()> {
        let output = Command::new("npm")
            .args(["run", "typeshare"])
            .current_dir("./services/ui")
            .output()?;
        println!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));

        println!("Saved to: ./services/ui/src/types/bencher.ts");

        Ok(())
    }
}