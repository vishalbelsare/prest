extern crate prest;
extern crate byteorder;

use prest::alt_set::{AltSet};
use prest::rpc_common::{Subject,ChoiceRow};
use std::iter::FromIterator;
use rand::Rng;

fn fac(n : u32) -> u32 {
    (1..(n+1)).product()
}

fn comb(n : u32, k : u32) -> u32 {
    fac(n) / (fac(k) * fac(n-k))
}

fn datasets_nfc(n : u32) -> u32 {
    (1..(n+1)).map(|i|
        (i+1).pow(comb(n, i) as u32)
    ).product()
}

fn main() {
    let alts = vec![
        String::from("a"),
        String::from("b"),
        String::from("c"),
        String::from("d")
    ];

    for code in 0..datasets_nfc(4) {
        let mut j = code;
        let subject = Subject {
            name: code.to_string(),
            alternatives: alts.clone(),
            choices: AltSet::powerset(4).map(
                |menu| {
                    let n = menu.size();
                    let k = j % (n + 1);
                    j = j / (n + 1);

                    let choice = if k == n {
                        AltSet::empty()
                    } else {
                        AltSet::singleton(
                            Vec::from_iter(menu.view())[k as usize]
                        )
                    };

                    ChoiceRow {
                        menu,
                        default: None,
                        choice,
                    }
                }
            ).collect()
        };
    }
}
