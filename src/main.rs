extern crate rustc_serialize;
extern crate markov;

use markov::Chain;

fn main() {
    let chain: Chain = Chain::from_file("chain.json").unwrap();

    let mut count: u32 = 0;
    for s in chain {
        println!("{}",s);
        count += 1;
        if count > 10 {
            break;
        }
    }

}
