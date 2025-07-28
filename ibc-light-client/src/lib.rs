use anyhow::Result;
use serde::{Deserialize, Serialize};

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
        // Simplified verification - just check we have some signatures
        // In real implementation this would verify ed25519 signatures
        let signature_count = header.commit_signatures.len();
        let required_signatures = (self.trusted_validators.validators.len() * 2 + 2) / 3;
        
        Ok(signature_count >= required_signatures)
    }

    pub fn extract_app_hash(&self, header: &TendermintHeader) -> [u8; 32] {
        header.app_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_validator_set() -> ValidatorSet {
        ValidatorSet {
            validators: vec![
                Validator {
                    pub_key: vec![1; 32],
                    voting_power: 100,
                },
                Validator {
                    pub_key: vec![2; 32],
                    voting_power: 100,
                },
                Validator {
                    pub_key: vec![3; 32],
                    voting_power: 100,
                },
            ],
            total_voting_power: 300,
        }
    }

    #[test]
    fn test_light_client_verification_success() {
        let validator_set = create_test_validator_set();
        let light_client = LightClient::new(validator_set);

        let header = TendermintHeader {
            height: 1000,
            time: "2024-01-01T00:00:00Z".to_string(),
            app_hash: [42; 32],
            validators_hash: [24; 32],
            commit_signatures: vec![
                CommitSignature {
                    validator_address: [1; 20],
                    signature: vec![0x42; 64],
                },
                CommitSignature {
                    validator_address: [2; 20],
                    signature: vec![0x43; 64],
                },
            ],
        };

        let result = light_client.verify_header(&header).unwrap();
        assert!(result, "Header verification should succeed with 2/3 signatures");
    }

    #[test]
    fn test_light_client_verification_failure() {
        let validator_set = create_test_validator_set();
        let light_client = LightClient::new(validator_set);

        let header = TendermintHeader {
            height: 1000,
            time: "2024-01-01T00:00:00Z".to_string(),
            app_hash: [42; 32],
            validators_hash: [24; 32],
            commit_signatures: vec![
                CommitSignature {
                    validator_address: [1; 20],
                    signature: vec![0x42; 64],
                },
            ],
        };

        let result = light_client.verify_header(&header).unwrap();
        assert!(!result, "Header verification should fail with insufficient signatures");
    }

    #[test]
    fn test_extract_app_hash() {
        let validator_set = create_test_validator_set();
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