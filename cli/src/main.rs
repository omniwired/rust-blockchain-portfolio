use anyhow::Result;
use clap::{Parser, Subcommand};
use ibc_light_client::{LightClient, TendermintHeader, ValidatorSet, Validator};
use zk_circuit::ZkProver;
use ark_std::rand::SeedableRng;

#[derive(Parser)]
#[command(name = "ibc-mini")]
#[command(about = "IBC Mini: Light client + ZK proof demo")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a ZK proof for header verification
    Prove {
        /// Block height to prove
        #[arg(long)]
        height: u64,
    },
    /// Run the full demo
    Demo,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prove { height } => {
            println!("🔍 Generating proof for height {}", height);
            generate_proof(height).await?;
        }
        Commands::Demo => {
            println!("🚀 Running IBC-Mini demo");
            run_demo().await?;
        }
    }

    Ok(())
}

async fn generate_proof(height: u64) -> Result<()> {
    // Create mock validator set (simplified for demo)
    let validators = vec![
        Validator {
            pub_key: vec![1; 32],
            voting_power: 100,
        },
        Validator {
            pub_key: vec![2; 32], 
            voting_power: 100,
        },
    ];
    
    let validator_set = ValidatorSet {
        validators,
        total_voting_power: 200,
    };

    // Create mock header with signatures for demo
    let header = TendermintHeader {
        height,
        time: "2024-01-01T00:00:00Z".to_string(),
        app_hash: [42; 32],
        validators_hash: [24; 32],
        commit_signatures: vec![
            ibc_light_client::CommitSignature {
                validator_address: [1; 20],
                signature: vec![0x42; 64], // Mock signature
            },
            ibc_light_client::CommitSignature {
                validator_address: [2; 20], 
                signature: vec![0x43; 64], // Mock signature
            },
        ],
    };

    // Verify with light client
    let light_client = LightClient::new(validator_set);
    let is_valid = light_client.verify_header(&header)?;
    
    if !is_valid {
        anyhow::bail!("Header verification failed");
    }
    
    let app_hash = light_client.extract_app_hash(&header);
    println!("✅ Light client verification passed");
    println!("📋 App hash: {:?}", hex::encode(app_hash));

    // Generate ZK proof
    println!("🔧 Setting up ZK circuit...");
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(42); // Use deterministic RNG for demo
    let prover = ZkProver::setup(&mut rng)?;
    
    println!("⚡ Generating proof...");
    let _proof = prover.prove([24; 32], app_hash, &mut rng)?;
    
    println!("🎉 Proof generated successfully!");
    println!("📄 Proof generated (serialization not shown for demo)");
    
    Ok(())
}

async fn run_demo() -> Result<()> {
    println!("Running full IBC-Mini demonstration:");
    println!("1. Mock Tendermint header verification");
    println!("2. ZK proof generation"); 
    println!("3. (Contract verification would happen here)");
    
    generate_proof(1000).await?;
    
    println!("\n🏆 Demo completed successfully!");
    println!("   In a real implementation:");
    println!("   - Header would be fetched from Osmosis RPC");
    println!("   - Ed25519 signatures would be verified");
    println!("   - Proof would be sent to CosmWasm contract");
    
    Ok(())
}