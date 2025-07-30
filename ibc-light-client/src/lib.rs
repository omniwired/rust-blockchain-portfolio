use anyhow::{anyhow, Result};
use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSet {
    pub validators: Vec<Validator>,
    pub total_voting_power: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub pub_key: Vec<u8>,
    pub voting_power: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TendermintHeader {
    pub height: u64,
    pub time: String,
    pub app_hash: [u8; 32],
    pub validators_hash: [u8; 32],
    pub commit_signatures: Vec<CommitSignature>,
}

impl TendermintHeader {
    // Create a digest of the header for signing
    pub fn digest(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.height.to_be_bytes());
        hasher.update(self.time.as_bytes());
        hasher.update(self.app_hash);
        hasher.update(self.validators_hash);
        hasher.finalize().into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSignature {
    pub validator_address: [u8; 20],
    pub signature: Vec<u8>,
}

pub struct LightClient {
    trusted_validators: ValidatorSet,
}

impl LightClient {
    pub fn new(validators: ValidatorSet) -> Self {
        Self {
            trusted_validators: validators,
        }
    }

    pub fn verify_header(&self, header: &TendermintHeader) -> Result<bool> {
        let header_digest = header.digest();
        let mut signing_power = 0;

        for commit_sig in &header.commit_signatures {
            if let Some(validator) = self
                .trusted_validators
                .validators
                .iter()
                .find(|v| {
                    let mut hasher = Sha256::new();
                    hasher.update(&v.pub_key);
                    let hash = hasher.finalize();
                    let truncated_hash = &hash[0..20];
                    truncated_hash == commit_sig.validator_address
                })
            {
                let pub_key_bytes: [u8; 32] = validator.pub_key.clone().try_into().map_err(|_| anyhow!("Invalid public key length"))?;
                let signature_bytes: [u8; 64] = commit_sig.signature.clone().try_into().map_err(|_| anyhow!("Invalid signature length"))?;

                let public_key = VerifyingKey::from_bytes(&pub_key_bytes)?;
                let signature = Signature::from_bytes(&signature_bytes);

                if public_key.verify(&header_digest, &signature).is_ok() {
                    signing_power += validator.voting_power;
                }
            }
        }

        let required_power = (self.trusted_validators.total_voting_power * 2) / 3;
        Ok(signing_power > required_power)
    }

    pub fn extract_app_hash(&self, header: &TendermintHeader) -> [u8; 32] {
        header.app_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use sha2::{Digest, Sha256};

    fn create_test_keys() -> (SigningKey, VerifyingKey) {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        (signing_key, signing_key.verifying_key())
    }

    fn create_test_validator_set() -> (ValidatorSet, Vec<SigningKey>) {
        let (sk1, vk1) = create_test_keys();
        let (sk2, vk2) = create_test_keys();
        let (sk3, vk3) = create_test_keys();

        let validators = vec![
            Validator {
                pub_key: vk1.to_bytes().to_vec(),
                voting_power: 100,
            },
            Validator {
                pub_key: vk2.to_bytes().to_vec(),
                voting_power: 100,
            },
            Validator {
                pub_key: vk3.to_bytes().to_vec(),
                voting_power: 100,
            },
        ];

        (ValidatorSet {
            validators,
            total_voting_power: 300,
        }, vec![sk1, sk2, sk3])
    }

    #[test]
    fn test_light_client_verification_success() {
        let (validator_set, signing_keys) = create_test_validator_set();
        let light_client = LightClient::new(validator_set.clone());

        let mut header = TendermintHeader {
            height: 1000,
            time: "2024-01-01T00:00:00Z".to_string(),
            app_hash: [42; 32],
            validators_hash: [24; 32],
            commit_signatures: vec![],
        };

        let header_digest = header.digest();

        let sig1 = signing_keys[0].sign(&header_digest);
        let sig2 = signing_keys[1].sign(&header_digest);

        let mut hasher = Sha256::new();
        hasher.update(&validator_set.validators[0].pub_key);
        let vk1_hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(&validator_set.validators[1].pub_key);
        let vk2_hash = hasher.finalize();

        header.commit_signatures = vec![
            CommitSignature {
                validator_address: vk1_hash[0..20].try_into().unwrap(),
                signature: sig1.to_bytes().to_vec(),
            },
            CommitSignature {
                validator_address: vk2_hash[0..20].try_into().unwrap(),
                signature: sig2.to_bytes().to_vec(),
            },
        ];

        let result = light_client.verify_header(&header).unwrap();
        assert!(result, "Header verification should succeed with 2/3 voting power");
    }

    #[test]
    fn test_light_client_verification_failure() {
        let (validator_set, signing_keys) = create_test_validator_set();
        let light_client = LightClient::new(validator_set.clone());

        let mut header = TendermintHeader {
            height: 1000,
            time: "2024-01-01T00:00:00Z".to_string(),
            app_hash: [42; 32],
            validators_hash: [24; 32],
            commit_signatures: vec![],
        };

        let header_digest = header.digest();
        let sig1 = signing_keys[0].sign(&header_digest);

        let mut hasher = Sha256::new();
        hasher.update(&validator_set.validators[0].pub_key);
        let vk1_hash = hasher.finalize();

        header.commit_signatures = vec![
            CommitSignature {
                validator_address: vk1_hash[0..20].try_into().unwrap(),
                signature: sig1.to_bytes().to_vec(),
            },
        ];

        let result = light_client.verify_header(&header).unwrap();
        assert!(!result, "Header verification should fail with insufficient voting power");
    }

    #[test]
    fn test_extract_app_hash() {
        let (validator_set, _) = create_test_validator_set();
        let light_client = LightClient::new(validator_set);

        let expected_hash = [42; 32];
        let header = TendermintHeader {
            height: 1000,
            time: "2024-01-01T00:00:00Z".to_string(),
            app_hash: expected_hash,
            validators_hash: [24; 32],
            commit_signatures: vec![],
        };

        let app_hash = light_client.extract_app_hash(&header);
        assert_eq!(app_hash, expected_hash, "App hash should be extracted correctly");
    }
}