use anyhow::Result;
use ark_bn254::Fr;
use ark_std::rand::SeedableRng;
use ark_std::UniformRand;
use clap::{Parser, Subcommand};
use ed25519_dalek::{Signer, SigningKey};
use ibc_light_client::{LightClient, TendermintHeader, Validator, ValidatorSet};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use zk_circuit::ZkProver;

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
    // Create mock validator set with real keys
    let mut csprng = OsRng;
    let sk1 = SigningKey::generate(&mut csprng);
    let vk1 = sk1.verifying_key();
    let sk2 = SigningKey::generate(&mut csprng);
    let vk2 = sk2.verifying_key();

    let validators = vec![
        Validator {
            pub_key: vk1.to_bytes().to_vec(),
            voting_power: 100,
        },
        Validator {
            pub_key: vk2.to_bytes().to_vec(),
            voting_power: 100,
        },
    ];

    let validator_set = ValidatorSet {
        validators: validators.clone(),
        total_voting_power: 200,
    };

    // Create mock header and sign it
    let mut header = TendermintHeader {
        height,
        time: "2024-01-01T00:00:00Z".to_string(),
        app_hash: [42; 32],
        validators_hash: [24; 32],
        commit_signatures: vec![],
    };

    let header_digest = header.digest();
    let sig1 = sk1.sign(&header_digest);
    let sig2 = sk2.sign(&header_digest);

    let mut hasher = Sha256::new();
    hasher.update(&validators[0].pub_key);
    let vk1_hash = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&validators[1].pub_key);
    let vk2_hash = hasher.finalize();

    header.commit_signatures = vec![
        ibc_light_client::CommitSignature {
            validator_address: vk1_hash[0..20].try_into().unwrap(),
            signature: sig1.to_bytes().to_vec(),
        },
        ibc_light_client::CommitSignature {
            validator_address: vk2_hash[0..20].try_into().unwrap(),
            signature: sig2.to_bytes().to_vec(),
        },
    ];

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
    let a = Fr::rand(&mut rng);
    let b = Fr::rand(&mut rng);
    let (_proof, c) = prover.prove(a, b, &mut rng)?;

    println!("🎉 Proof generated successfully!");
    println!("📄 Public input 'a': {:?}", a);
    println!("📄 Public output 'c': {:?}", c);

    Ok(())
}

async fn run_demo() -> Result<()> {
    println!("Running full IBC-Mini demonstration:");
    println!("1. Mock Tendermint header verification");
    println!("2. ZK proof generation");

    generate_proof(1000).await?;

    println!("\n🏆 Demo completed successfully!");
    println!("   In a real implementation:");
    println!("   - Header would be fetched from Osmosis RPC");
    println!("   - ZK proof would be verified on-chain");

    Ok(())
}