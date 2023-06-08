Run `cargo run --example mst --release`

Expected output `gas_cost = 576052` 

Remember to delete the output files in the output folder before running the example again.

To use params from Hermez setup download `hermez-raw-22` from https://github.com/han0110/halo2-kzg-srs and place it into a `ptau` folder. Then enable its usage from

```
    let ptau_path = format!("ptau/hermez-raw-{}", 22);
    let mut params_fs = File::open(ptau_path).expect("couldn't load params");
    let params = ParamsKZG::<Bn256>::read(&mut params_fs).expect("Failed to read params");

    // let params = gen_srs(22); to use random params instead of the ones from hermez
```