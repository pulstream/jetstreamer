use jetstreamer::{JetstreamerRunner, parse_cli_args};
use jetstreamer_plugin::plugins::mint_tracking::MintTrackingPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_cli_args()?;

    let mut runner = JetstreamerRunner::default()
        .with_log_level("info")
        .parse_cli_args()?;

    if let Some(mint) = config.mint {
        runner = runner.with_plugin(Box::new(MintTrackingPlugin::new(mint)));
    }

    runner
        .run()
        .map_err(|err| -> Box<dyn std::error::Error> { Box::new(err) })?;

    Ok(())
}
