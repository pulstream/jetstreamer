use jetstreamer::JetstreamerRunner;
use jetstreamer_plugin::plugins::mint_tracking::MintTrackingPlugin;
use solana_pubkey::Pubkey;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get mint address from command line arguments
    let args: Vec<String> = env::args().collect();

    // if args.len() < 2 {
    //     eprintln!("Usage: {} <mint_address> [slot_range]", args[0]);
    //     eprintln!(
    //         "Example: {} EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v 358560000:367631999",
    //         args[0]
    //     );
    //     eprintln!("Common mint addresses:");
    //     eprintln!("  USDC: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    //     eprintln!("  USDT: Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
    //     eprintln!("  SOL:  So11111111111111111111111111111111111111112");
    //     std::process::exit(1);
    // }

    let mint_address = "6P5PncNBnuSAkseoMzhbJRZT8dhzD1o1THKp3Pxgpump"
        .parse::<Pubkey>()
        .map_err(|e| format!("Invalid mint address '{}': {}", args[1], e))?;

    println!("Tracking mint address: {}", mint_address);

    JetstreamerRunner::default()
        .with_log_level("info")
        .parse_cli_args()?
        .with_plugin(Box::new(MintTrackingPlugin::new(mint_address)))
        .run()
        .map_err(|err| -> Box<dyn std::error::Error> { Box::new(err) })?;
    Ok(())
}
