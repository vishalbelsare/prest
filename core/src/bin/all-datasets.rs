extern crate prest;

use prest::alt_set::{AltSet};
use prest::rpc_common::{Subject,ChoiceRow};
use prest::precomputed::Precomputed;
use prest::estimation;
use std::iter::FromIterator;

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

    let precomputed = {
        let mut p = Precomputed::new(None);
        p.precompute(4).unwrap();
        p
    };

    let models = {
        use prest::model::*;
        use prest::model::Model::*;

        let pp = PreorderParams{strict: Some(true), total: Some(false)};

        [
            PreorderMaximization(pp),
            Unattractiveness(pp),
            UndominatedChoice{strict: true},
            PartiallyDominantChoice{fc: false},
            Overload(pp),
            SequentialDomination{strict: true},
        ]
    };

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

        let response = estimation::run_one(
            &precomputed, true, &subject, &models
        ).unwrap();

        for instance in &response.best_instances {
            println!(
                "{},{},\"{:?}\"",
                code,
                instance.entropy,
                instance.model,
            );
        }
    }
}
