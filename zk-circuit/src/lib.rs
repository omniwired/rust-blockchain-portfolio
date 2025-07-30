use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
use ark_r1cs_std::fields::fp::FpVar;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::prelude::*;
use ark_snark::SNARK;
use ark_std::rand::{CryptoRng, RngCore};

#[derive(Clone)]
pub struct AddCircuit {
    // Public inputs
    pub a: Option<Fr>,
    pub c: Option<Fr>,

    // Private witness
    pub b: Option<Fr>,
}

impl ConstraintSynthesizer<Fr> for AddCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // witness allocation
        let a_var = FpVar::new_input(cs.clone(), || {
            self.a.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let b_var = FpVar::new_witness(cs.clone(), || {
            self.b.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let c_var = FpVar::new_input(cs.clone(), || {
            self.c.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // constraint: a + b = c
        let sum = a_var + b_var;
        c_var.enforce_equal(&sum)?;

        Ok(())
    }
}

pub struct ZkProver {
    proving_key: ProvingKey<Bn254>,
    verifying_key: VerifyingKey<Bn254>,
}

impl ZkProver {
    pub fn setup<R: RngCore + CryptoRng>(mut rng: R) -> anyhow::Result<Self> {
        // dummy circuit for setup
        let circuit = AddCircuit {
            a: None,
            b: None,
            c: None,
        };

        let (pk, vk) = Groth16::<Bn254>::circuit_specific_setup(circuit, &mut rng)
            .map_err(|e| anyhow::anyhow!("Setup failed: {:?}", e))?;

        Ok(ZkProver {
            proving_key: pk,
            verifying_key: vk,
        })
    }

    pub fn prove<R: RngCore + CryptoRng>(
        &self,
        a: Fr,
        b: Fr,
        mut rng: R,
    ) -> anyhow::Result<(Proof<Bn254>, Fr)> {
        // lol this is just addition but with extra steps
        let c = a + b;

        let circuit = AddCircuit {
            a: Some(a),
            b: Some(b),
            c: Some(c),
        };

        // this takes forever on debug builds btw
        let proof = Groth16::<Bn254>::prove(&self.proving_key, circuit, &mut rng)
            .map_err(|e| anyhow::anyhow!("Proving failed: {:?}", e))?;

        Ok((proof, c))
    }

    pub fn verifying_key(&self) -> &VerifyingKey<Bn254> {
        &self.verifying_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::rand::SeedableRng;
    use ark_std::UniformRand;

    #[test]
    fn test_zk_circuit_setup_and_prove() {
        let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(42);

        // Setup
        let prover_result = ZkProver::setup(&mut rng);
        assert!(prover_result.is_ok(), "Setup should succeed");
        let prover = prover_result.unwrap();

        // Prove
        let a = Fr::rand(&mut rng);
        let b = Fr::rand(&mut rng);
        let (proof, c) = prover.prove(a, b, &mut rng).unwrap();

        use ark_groth16::Groth16;
        use ark_snark::SNARK;

        let pvk = Groth16::process_vk(&prover.verifying_key).unwrap();
        let valid_proof = Groth16::verify_with_processed_vk(&pvk, &[a, c], &proof).unwrap();

        assert!(valid_proof, "Proof verification should succeed");
    }
}