use ark_std::{end_timer, start_timer};
use halo2_proofs::plonk::*;
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::{
    halo2curves::bn256::{Bn256, Fr as Fp},
    poly::kzg::commitment::ParamsKZG,
};
use snark_verifier_sdk::evm::{evm_verify, gen_evm_proof_shplonk, gen_evm_verifier_shplonk};
use snark_verifier_sdk::halo2::gen_srs;
use snark_verifier_sdk::CircuitExt;
use snark_verifier_sdk::SHPLONK;
use snark_verifier_sdk::{
    gen_pk,
    halo2::{aggregation::AggregationCircuit, gen_snark_shplonk},
    Snark,
};
use std::fs::File;
use std::path::Path;
use summa_solvency::circuits::utils::instantiate_circuit;
use summa_solvency::merkle_sum_tree::MerkleSumTree;

fn gen_application_snark(params: &ParamsKZG<Bn256>) -> Snark {
    let merkle_sum_tree = MerkleSumTree::new("entry_16.csv").unwrap();

    let user_index = 0;

    let mt_proof = merkle_sum_tree.generate_proof(user_index).unwrap();

    // assets_sum are defined as liabilities_sum + 1
    let assets_sum = merkle_sum_tree.root().balance + Fp::from(1u64); // assets_sum are defined as liabilities_sum + 1

    let circuit = instantiate_circuit(assets_sum, mt_proof);

    let pk = gen_pk(params, &circuit, Some(Path::new("./output/app.pk")));

    gen_snark_shplonk(params, &pk, circuit, None::<&str>)
}

fn main() {
    let params_app = gen_srs(9);
    let snarks = [(); 1].map(|_| gen_application_snark(&params_app));

    let ptau_path = format!("ptau/hermez-raw-{}", 22);
    let mut params_fs = File::open(ptau_path).expect("couldn't load params");
    let params = ParamsKZG::<Bn256>::read(&mut params_fs).expect("Failed to read params");

    // let params = gen_srs(22); to use random params instead of the ones from hermez

    let agg_circuit = AggregationCircuit::<SHPLONK>::new(&params, snarks);

    let start0 = start_timer!(|| "gen vk & pk");
    let pk = gen_pk(
        &params,
        &agg_circuit.without_witnesses(),
        Some(Path::new("./output/agg.pk")),
    );
    end_timer!(start0);

    std::fs::remove_file("./output/agg.snark").unwrap_or_default();
    let _snark = gen_snark_shplonk(
        &params,
        &pk,
        agg_circuit.clone(),
        Some(Path::new("./output/agg.snark")),
    );

    // do one more time to verify
    let num_instances = agg_circuit.num_instance();
    let instances = agg_circuit.instances();
    let proof_calldata = gen_evm_proof_shplonk(&params, &pk, agg_circuit, instances.clone());

    let deployment_code = gen_evm_verifier_shplonk::<AggregationCircuit<SHPLONK>>(
        &params,
        pk.get_vk(),
        num_instances,
        Some(Path::new("./output/standard_plonk.yul")),
    );
    evm_verify(deployment_code, instances, proof_calldata);
}
